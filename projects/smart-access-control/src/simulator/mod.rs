//! Simulated door controllers.
//!
//! **Software simulation only.** Nothing here talks to, or models the
//! protocol of, any real access control hardware.
//!
//! Each simulated controller is one Tokio task. It talks to the backend only
//! through a [`ControllerLink`], the same three operations a real device
//! would use (heartbeat, access request, disconnect), and is steered by
//! commands (swipe a card, lose the network, reconnect, stop).
//!
//! The simulator sits *outside* the system it exercises: it depends on the
//! application's types, never on its internals. The in-process link is
//! implemented in the interfaces layer; a network link (HTTP, MQTT) could
//! replace it without changing this module.

use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use thiserror::Error;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::MissedTickBehavior,
};

use crate::{
    application::{ApplicationError, HeartbeatReport},
    domain::{AccessEvent, DoorId, Timestamp},
    error_chain,
};

/// How a controller reaches the backend.
pub trait ControllerLink: Send + Sync + 'static {
    fn heartbeat(&self) -> impl Future<Output = Result<HeartbeatReport, ApplicationError>> + Send;

    fn request_access(
        &self,
        card_number: String,
        door_id: DoorId,
    ) -> impl Future<Output = Result<AccessEvent, ApplicationError>> + Send;

    fn disconnect(&self) -> impl Future<Output = Result<(), ApplicationError>> + Send;
}

#[derive(Debug, Error)]
pub enum SimulatorError {
    #[error("simulated controller {0} is not running")]
    NotRunning(String),

    #[error("simulated controller {0} is already running")]
    AlreadyRunning(String),

    /// The simulated network is down: the controller cannot reach the
    /// backend and denies access locally. Nothing is recorded centrally.
    #[error("simulated controller {0} has no network connection; access denied locally")]
    Offline(String),

    #[error("backend rejected the request")]
    Backend(#[from] ApplicationError),
}

/// What an observer can see about a running simulated controller.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SimulatedStatus {
    pub network_up: bool,
    pub heartbeats_sent: u64,
    pub last_heartbeat_at: Option<Timestamp>,
    /// Last failed call to the backend, if any (cleared by a success).
    pub last_error: Option<String>,
}

enum Command {
    Swipe {
        card_number: String,
        door_id: DoorId,
        reply: oneshot::Sender<Result<AccessEvent, SimulatorError>>,
    },
    /// The network drops without warning: the backend only notices through
    /// the missing heartbeats.
    Outage {
        reply: oneshot::Sender<()>,
    },
    /// A clean shutdown of the connection: the backend is told first.
    Disconnect {
        reply: oneshot::Sender<Result<(), SimulatorError>>,
    },
    Reconnect {
        reply: oneshot::Sender<Result<(), SimulatorError>>,
    },
    Stop,
}

struct Handle {
    commands: mpsc::Sender<Command>,
    status: Arc<Mutex<SimulatedStatus>>,
    task: JoinHandle<()>,
}

/// The set of running simulated controllers.
///
/// A `std::sync::Mutex` guards the registry: it is only held briefly to look
/// up or remove a handle, never across an `.await`.
#[derive(Default)]
pub struct Simulator {
    running: Mutex<HashMap<String, Handle>>,
}

impl Simulator {
    /// Starts a simulated controller that sends a heartbeat every
    /// `heartbeat_every` while its network is up (the first one at once).
    /// `now` stamps status updates for display.
    pub fn start<L: ControllerLink>(
        &self,
        id: &str,
        link: L,
        heartbeat_every: Duration,
        now: impl Fn() -> Timestamp + Send + 'static,
    ) -> Result<(), SimulatorError> {
        let mut running = self.lock();
        if running.contains_key(id) {
            return Err(SimulatorError::AlreadyRunning(id.to_owned()));
        }
        // Bounded: a flood of API calls queues up to 32 commands, then the
        // caller waits instead of memory growing without limit.
        let (commands, receiver) = mpsc::channel(32);
        let status = Arc::new(Mutex::new(SimulatedStatus {
            network_up: true,
            ..SimulatedStatus::default()
        }));
        let task = tokio::spawn(run(
            id.to_owned(),
            link,
            receiver,
            Arc::clone(&status),
            heartbeat_every,
            now,
        ));
        running.insert(
            id.to_owned(),
            Handle {
                commands,
                status,
                task,
            },
        );
        Ok(())
    }

    pub fn is_running(&self, id: &str) -> bool {
        self.lock().contains_key(id)
    }

    /// Running controllers and their status, sorted by id.
    pub fn list(&self) -> Vec<(String, SimulatedStatus)> {
        let mut list: Vec<_> = self
            .lock()
            .iter()
            .map(|(id, handle)| (id.clone(), read(&handle.status)))
            .collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    }

    pub fn status(&self, id: &str) -> Result<SimulatedStatus, SimulatorError> {
        self.lock()
            .get(id)
            .map(|handle| read(&handle.status))
            .ok_or_else(|| SimulatorError::NotRunning(id.to_owned()))
    }

    /// Presents a card at one of the controller's doors.
    pub async fn swipe(
        &self,
        id: &str,
        card_number: String,
        door_id: DoorId,
    ) -> Result<AccessEvent, SimulatorError> {
        self.request(id, |reply| Command::Swipe {
            card_number,
            door_id,
            reply,
        })
        .await?
    }

    pub async fn outage(&self, id: &str) -> Result<(), SimulatorError> {
        self.request(id, |reply| Command::Outage { reply }).await
    }

    pub async fn disconnect(&self, id: &str) -> Result<(), SimulatorError> {
        self.request(id, |reply| Command::Disconnect { reply })
            .await?
    }

    pub async fn reconnect(&self, id: &str) -> Result<(), SimulatorError> {
        self.request(id, |reply| Command::Reconnect { reply })
            .await?
    }

    /// Stops a controller: it disconnects cleanly, then its task ends.
    pub async fn stop(&self, id: &str) -> Result<(), SimulatorError> {
        // Take the handle out first; the lock is released before awaiting.
        let handle = self
            .lock()
            .remove(id)
            .ok_or_else(|| SimulatorError::NotRunning(id.to_owned()))?;
        shut_down(handle).await;
        Ok(())
    }

    /// Stops every controller. Called on server shutdown, before the
    /// database pool closes, so their doors are marked offline cleanly.
    pub async fn stop_all(&self) {
        let handles: Vec<Handle> = self.lock().drain().map(|(_, handle)| handle).collect();
        for handle in handles {
            shut_down(handle).await;
        }
    }

    /// Sends a command built around a reply channel and waits for the reply.
    async fn request<T>(
        &self,
        id: &str,
        command: impl FnOnce(oneshot::Sender<T>) -> Command,
    ) -> Result<T, SimulatorError> {
        let (reply, response) = oneshot::channel();
        self.send(id, command(reply)).await?;
        response
            .await
            .map_err(|_| SimulatorError::NotRunning(id.to_owned()))
    }

    async fn send(&self, id: &str, command: Command) -> Result<(), SimulatorError> {
        // Clone the sender so the registry lock is not held while waiting
        // for room in the channel.
        let sender = self
            .lock()
            .get(id)
            .map(|handle| handle.commands.clone())
            .ok_or_else(|| SimulatorError::NotRunning(id.to_owned()))?;
        sender
            .send(command)
            .await
            .map_err(|_| SimulatorError::NotRunning(id.to_owned()))
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, Handle>> {
        lock(&self.running)
    }
}

async fn shut_down(handle: Handle) {
    let _ = handle.commands.send(Command::Stop).await;
    if tokio::time::timeout(Duration::from_secs(5), handle.task)
        .await
        .is_err()
    {
        tracing::warn!("simulated controller did not stop in time");
    }
}

/// Every guarded update here is a single insert, remove or field write: a
/// panic cannot leave the data half-updated, so recovering from poisoning is safe.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn read(status: &Mutex<SimulatedStatus>) -> SimulatedStatus {
    lock(status).clone()
}

fn update(status: &Mutex<SimulatedStatus>, change: impl FnOnce(&mut SimulatedStatus)) {
    change(&mut lock(status));
}

/// Sends one heartbeat and records the outcome in `status`.
///
/// A plain `async fn` rather than a closure: its future borrows `link`, and
/// an `async fn` expresses "the future lives as long as its reference
/// arguments" automatically. A closure returning such a future cannot state
/// that lifetime relationship in its signature, so the compiler rejects it.
async fn send_heartbeat<L: ControllerLink>(
    link: &L,
    status: &Mutex<SimulatedStatus>,
    at: Timestamp,
) -> Result<HeartbeatReport, ApplicationError> {
    let result = link.heartbeat().await;
    update(status, |s| match &result {
        Ok(_) => {
            s.heartbeats_sent += 1;
            s.last_heartbeat_at = Some(at);
            s.last_error = None;
        }
        Err(err) => s.last_error = Some(error_chain(err)),
    });
    result
}

/// The controller's main loop: heartbeats on a timer, commands as they come.
async fn run<L: ControllerLink>(
    id: String,
    link: L,
    mut commands: mpsc::Receiver<Command>,
    status: Arc<Mutex<SimulatedStatus>>,
    heartbeat_every: Duration,
    now: impl Fn() -> Timestamp + Send + 'static,
) {
    let mut network_up = true;
    let mut ticks = tokio::time::interval(heartbeat_every);
    ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = ticks.tick(), if network_up => {
                if let Err(err) = send_heartbeat(&link, &status, now()).await {
                    tracing::warn!(controller_id = %id, error = %error_chain(&err), "simulated heartbeat failed");
                }
            }
            command = commands.recv() => match command {
                Some(Command::Swipe { card_number, door_id, reply }) => {
                    let result = if network_up {
                        link.request_access(card_number, door_id).await.map_err(SimulatorError::from)
                    } else {
                        Err(SimulatorError::Offline(id.clone()))
                    };
                    let _ = reply.send(result);
                }
                Some(Command::Outage { reply }) => {
                    network_up = false;
                    update(&status, |s| s.network_up = false);
                    let _ = reply.send(());
                }
                Some(Command::Disconnect { reply }) => {
                    let result = if network_up {
                        link.disconnect().await.map_err(SimulatorError::from)
                    } else {
                        Ok(()) // already unreachable
                    };
                    network_up = false;
                    update(&status, |s| s.network_up = false);
                    let _ = reply.send(result);
                }
                Some(Command::Reconnect { reply }) => {
                    network_up = true;
                    update(&status, |s| s.network_up = true);
                    // Announce ourselves at once rather than at the next tick.
                    let result = send_heartbeat(&link, &status, now())
                        .await
                        .map(|_| ())
                        .map_err(SimulatorError::from);
                    ticks.reset();
                    let _ = reply.send(result);
                }
                Some(Command::Stop) | None => {
                    if network_up && let Err(err) = link.disconnect().await {
                        tracing::warn!(controller_id = %id, error = %error_chain(&err), "simulated disconnect failed");
                    }
                    break;
                }
            },
        }
    }
    tracing::debug!(controller_id = %id, "simulated controller stopped");
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::{application::events::tests::event, domain::AccessDecision};

    /// Records calls; fails them when `broken` is set.
    #[derive(Clone, Default)]
    struct FakeLink {
        calls: Arc<Mutex<Vec<&'static str>>>,
        broken: Arc<Mutex<bool>>,
    }

    impl FakeLink {
        fn calls(&self) -> Vec<&'static str> {
            self.calls.lock().unwrap().clone()
        }

        fn count(&self, call: &str) -> usize {
            self.calls().iter().filter(|c| **c == call).count()
        }

        fn record(&self, call: &'static str) -> Result<(), ApplicationError> {
            self.calls.lock().unwrap().push(call);
            if *self.broken.lock().unwrap() {
                return Err(ApplicationError::Unauthorized);
            }
            Ok(())
        }
    }

    impl ControllerLink for FakeLink {
        async fn heartbeat(&self) -> Result<HeartbeatReport, ApplicationError> {
            self.record("heartbeat")?;
            Ok(HeartbeatReport { doors_online: 1 })
        }

        async fn request_access(
            &self,
            _card_number: String,
            door_id: DoorId,
        ) -> Result<AccessEvent, ApplicationError> {
            self.record("request")?;
            Ok(event(door_id, AccessDecision::Granted, 0))
        }

        async fn disconnect(&self) -> Result<(), ApplicationError> {
            self.record("disconnect")
        }
    }

    const EVERY: Duration = Duration::from_secs(10);

    fn now() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap()
    }

    fn started() -> (Simulator, FakeLink) {
        let simulator = Simulator::default();
        let link = FakeLink::default();
        simulator.start("sim-1", link.clone(), EVERY, now).unwrap();
        (simulator, link)
    }

    /// Lets the controller task run until it is idle again.
    async fn settle() {
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
    }

    // `start_paused`: tokio's clock only moves when every task is idle or
    // when told to, so "30 seconds" of heartbeats run instantly and exactly.
    #[tokio::test(start_paused = true)]
    async fn heartbeats_are_periodic_while_connected() {
        let (_simulator, link) = started();
        settle().await;
        assert_eq!(link.count("heartbeat"), 1, "one heartbeat at start");

        // Let 30 s pass one period at a time, as real time would. (Jumping
        // 30 s at once would look like missed ticks, which
        // `MissedTickBehavior::Delay` deliberately does not replay in a burst.)
        for _ in 0..3 {
            tokio::time::advance(EVERY).await;
            settle().await;
        }
        assert_eq!(link.count("heartbeat"), 4);
    }

    #[tokio::test(start_paused = true)]
    async fn an_outage_silences_the_controller_and_denies_locally() {
        let (simulator, link) = started();
        settle().await;
        simulator.outage("sim-1").await.unwrap();

        tokio::time::advance(Duration::from_secs(60)).await;
        settle().await;
        assert_eq!(
            link.count("heartbeat"),
            1,
            "no heartbeats during the outage"
        );

        let swipe = simulator
            .swipe("sim-1", "CARD-1".into(), DoorId::generate())
            .await;
        assert!(matches!(swipe, Err(SimulatorError::Offline(_))));
        assert_eq!(link.count("request"), 0, "nothing reached the backend");
        assert!(!simulator.status("sim-1").unwrap().network_up);
    }

    #[tokio::test(start_paused = true)]
    async fn reconnect_heartbeats_immediately() {
        let (simulator, link) = started();
        settle().await;
        simulator.outage("sim-1").await.unwrap();

        simulator.reconnect("sim-1").await.unwrap();

        assert_eq!(link.count("heartbeat"), 2);
        let status = simulator.status("sim-1").unwrap();
        assert!(status.network_up);
        assert_eq!(status.heartbeats_sent, 2);
    }

    #[tokio::test(start_paused = true)]
    async fn swipes_go_through_the_link() {
        let (simulator, link) = started();
        let door = DoorId::generate();

        let event = simulator
            .swipe("sim-1", "CARD-1".into(), door)
            .await
            .unwrap();

        assert_eq!(event.door_id(), None, "fake event has no door reference");
        assert_eq!(link.count("request"), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn clean_disconnect_tells_the_backend() {
        let (simulator, link) = started();
        settle().await;

        simulator.disconnect("sim-1").await.unwrap();

        assert_eq!(link.count("disconnect"), 1);
        assert!(!simulator.status("sim-1").unwrap().network_up);
    }

    #[tokio::test(start_paused = true)]
    async fn stop_disconnects_and_ends_the_task() {
        let (simulator, link) = started();
        settle().await;

        simulator.stop("sim-1").await.unwrap();

        assert_eq!(link.calls().last(), Some(&"disconnect"));
        assert!(!simulator.is_running("sim-1"));
        assert!(matches!(
            simulator.stop("sim-1").await,
            Err(SimulatorError::NotRunning(_))
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn backend_errors_are_reported_not_fatal() {
        let (simulator, link) = started();
        settle().await;
        *link.broken.lock().unwrap() = true;

        tokio::time::advance(EVERY).await;
        settle().await;
        assert!(simulator.status("sim-1").unwrap().last_error.is_some());

        *link.broken.lock().unwrap() = false;
        tokio::time::advance(EVERY).await;
        settle().await;
        assert_eq!(
            simulator.status("sim-1").unwrap().last_error,
            None,
            "recovered"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn registry_rules() {
        let (simulator, _) = started();
        assert!(matches!(
            simulator.start("sim-1", FakeLink::default(), EVERY, now),
            Err(SimulatorError::AlreadyRunning(_))
        ));
        assert!(matches!(
            simulator
                .swipe("ghost", "CARD-1".into(), DoorId::generate())
                .await,
            Err(SimulatorError::NotRunning(_))
        ));

        simulator
            .start("sim-0", FakeLink::default(), EVERY, now)
            .unwrap();
        let ids: Vec<String> = simulator.list().into_iter().map(|(id, _)| id).collect();
        assert_eq!(ids, ["sim-0", "sim-1"]);

        simulator.stop_all().await;
        assert!(simulator.list().is_empty());
    }
}
