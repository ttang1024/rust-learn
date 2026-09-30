//! Door controllers: registration, credentials, liveness.
//!
//! A controller proves who it is with a secret key issued at registration.
//! Every call it makes counts as a sign of life; one that stays silent for
//! longer than the timeout is marked offline together with its doors, so
//! the decision engine stops granting access through them.

use std::{fmt, sync::Arc};

use chrono::Duration;

use super::{
    ApplicationError, Clock, ControllerRepository, DoorRepository, PageRequest, SecretGenerator,
};
use crate::domain::{Controller, ControllerId, DoorId, DoorStatus};

/// A controller plus its secret key. The key exists only in this value:
/// it is shown once and stored as a hash.
pub struct IssuedControllerKey {
    pub controller: Controller,
    pub key: String,
}

impl fmt::Debug for IssuedControllerKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IssuedControllerKey")
            .field("controller", &self.controller)
            .field("key", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartbeatReport {
    /// Doors of this controller that are online after the heartbeat
    /// (disabled doors stay disabled).
    pub doors_online: usize,
}

pub struct ControllerService<C, D> {
    controllers: C,
    doors: D,
    secrets: Arc<dyn SecretGenerator>,
    clock: Arc<dyn Clock>,
    timeout: Duration,
}

impl<C: ControllerRepository, D: DoorRepository> ControllerService<C, D> {
    pub fn new(
        controllers: C,
        doors: D,
        secrets: Arc<dyn SecretGenerator>,
        clock: Arc<dyn Clock>,
        timeout: Duration,
    ) -> Self {
        Self {
            controllers,
            doors,
            secrets,
            clock,
            timeout,
        }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub async fn register(&self, raw_id: &str) -> Result<IssuedControllerKey, ApplicationError> {
        let id = ControllerId::parse(raw_id)?;
        let key = self
            .secrets
            .generate()
            .map_err(ApplicationError::Internal)?;
        let controller = Controller::register(id, self.clock.now());
        self.controllers
            .insert(&controller, &self.secrets.hash(&key))
            .await?;
        Ok(IssuedControllerKey { controller, key })
    }

    /// Issues a new key; the old one stops working immediately.
    pub async fn rotate_key(&self, raw_id: &str) -> Result<IssuedControllerKey, ApplicationError> {
        let controller = self.get(raw_id).await?;
        let key = self
            .secrets
            .generate()
            .map_err(ApplicationError::Internal)?;
        self.controllers
            .set_key_hash(controller.id(), &self.secrets.hash(&key))
            .await?;
        Ok(IssuedControllerKey { controller, key })
    }

    pub async fn get(&self, raw_id: &str) -> Result<Controller, ApplicationError> {
        let id = ControllerId::parse(raw_id)?;
        self.controllers
            .find_by_id(&id)
            .await?
            .ok_or(ApplicationError::NotFound {
                entity: "controller",
            })
    }

    pub async fn list(&self, page: PageRequest) -> Result<Vec<Controller>, ApplicationError> {
        Ok(self.controllers.list(page).await?)
    }

    /// The controller owning `key`, or `Unauthorized`.
    pub async fn authenticate(&self, key: &str) -> Result<Controller, ApplicationError> {
        self.controllers
            .find_by_key_hash(&self.secrets.hash(key))
            .await?
            .ok_or(ApplicationError::Unauthorized)
    }

    /// The controller is alive: it and its (enabled) doors go online.
    pub async fn heartbeat(
        &self,
        controller: &Controller,
    ) -> Result<HeartbeatReport, ApplicationError> {
        let mut controller = controller.clone();
        controller.record_heartbeat(self.clock.now());
        self.controllers.update(&controller).await?;

        let mut doors_online = 0;
        for mut door in self.doors.list_by_controller(controller.id()).await? {
            match door.status() {
                DoorStatus::Online => doors_online += 1,
                DoorStatus::Offline => {
                    door.mark_online()?;
                    self.doors.update(&door).await?;
                    doors_online += 1;
                }
                DoorStatus::Disabled => {}
            }
        }
        Ok(HeartbeatReport { doors_online })
    }

    /// Clean disconnect: the controller and its doors go offline at once.
    pub async fn disconnect(&self, controller: &Controller) -> Result<(), ApplicationError> {
        let mut controller = controller.clone();
        controller.mark_offline();
        self.controllers.update(&controller).await?;
        self.take_doors_offline(controller.id()).await
    }

    /// Checks that `door_id` is one of this controller's doors, and counts
    /// the request as a heartbeat. Another controller's door, or a door that
    /// does not exist, is `Forbidden`: a controller learns nothing about
    /// doors it does not own.
    pub async fn authorize_request(
        &self,
        controller: &Controller,
        door_id: DoorId,
    ) -> Result<(), ApplicationError> {
        let owns_door = self
            .doors
            .find_by_id(door_id)
            .await?
            .is_some_and(|door| door.controller_id() == controller.id());
        if !owns_door {
            return Err(ApplicationError::Forbidden);
        }
        self.heartbeat(controller).await?;
        Ok(())
    }

    /// Marks controllers that missed the heartbeat timeout offline, with
    /// their doors. Returns the ids that went offline. Run periodically.
    pub async fn expire_stale(&self) -> Result<Vec<ControllerId>, ApplicationError> {
        let cutoff = self.clock.now() - self.timeout;
        let expired = self.controllers.expire_stale(cutoff).await?;
        metrics::counter!("controllers_expired_total").increment(expired.len() as u64);
        for id in &expired {
            tracing::warn!(controller_id = %id, "controller missed heartbeats; marked offline");
            self.take_doors_offline(id).await?;
        }
        Ok(expired)
    }

    async fn take_doors_offline(&self, id: &ControllerId) -> Result<(), ApplicationError> {
        for mut door in self.doors.list_by_controller(id).await? {
            if door.status() == DoorStatus::Online {
                door.mark_offline()?;
                self.doors.update(&door).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::{
        application::{
            CreateDoor, DoorService,
            fakes::{FakeSecrets, FixedClock, InMemoryControllers, InMemoryDoors},
        },
        domain::{ControllerStatus, Door, Timestamp},
    };

    fn start() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap()
    }

    struct Fixture {
        service: ControllerService<InMemoryControllers, InMemoryDoors>,
        doors: DoorService<InMemoryDoors>,
        clock: Arc<FixedClock>,
    }

    fn fixture() -> Fixture {
        let clock = FixedClock::at(start());
        let doors = InMemoryDoors::default();
        Fixture {
            service: ControllerService::new(
                InMemoryControllers::default(),
                doors.clone(),
                Arc::new(FakeSecrets::default()),
                clock.clone(),
                Duration::seconds(30),
            ),
            doors: DoorService::new(doors, clock.clone()),
            clock,
        }
    }

    impl Fixture {
        async fn door(&self, controller: &str) -> Door {
            self.doors
                .create(CreateDoor {
                    name: "Door".into(),
                    location: "Lab".into(),
                    controller_id: controller.into(),
                })
                .await
                .unwrap()
        }

        async fn status(&self, door: &Door) -> DoorStatus {
            self.doors.get(door.id()).await.unwrap().status()
        }
    }

    #[tokio::test]
    async fn register_issues_a_key_that_authenticates() {
        let f = fixture();
        let issued = f.service.register("ctrl-001").await.unwrap();

        assert_eq!(issued.controller.status(), ControllerStatus::Offline);
        let found = f.service.authenticate(&issued.key).await.unwrap();
        assert_eq!(found.id().as_str(), "ctrl-001");
        assert!(matches!(
            f.service.authenticate("wrong").await,
            Err(ApplicationError::Unauthorized)
        ));
        assert!(
            !format!("{issued:?}").contains(&issued.key),
            "key is redacted"
        );
    }

    #[tokio::test]
    async fn duplicate_or_invalid_ids_are_rejected() {
        let f = fixture();
        f.service.register("ctrl-001").await.unwrap();
        assert!(matches!(
            f.service.register("ctrl-001").await,
            Err(ApplicationError::Conflict { .. })
        ));
        assert!(matches!(
            f.service.register("not valid!").await,
            Err(ApplicationError::Domain(_))
        ));
    }

    #[tokio::test]
    async fn rotating_the_key_invalidates_the_old_one() {
        let f = fixture();
        let old = f.service.register("ctrl-001").await.unwrap();
        let new = f.service.rotate_key("ctrl-001").await.unwrap();

        assert_ne!(old.key, new.key);
        assert!(f.service.authenticate(&old.key).await.is_err());
        assert!(f.service.authenticate(&new.key).await.is_ok());
    }

    #[tokio::test]
    async fn heartbeat_brings_own_enabled_doors_online() {
        let f = fixture();
        let controller = f.service.register("ctrl-001").await.unwrap().controller;
        let (mine, disabled, other) = (
            f.door("ctrl-001").await,
            f.door("ctrl-001").await,
            f.door("ctrl-999").await,
        );
        f.doors.disable(disabled.id()).await.unwrap();

        let report = f.service.heartbeat(&controller).await.unwrap();

        assert_eq!(report.doors_online, 1);
        assert_eq!(f.status(&mine).await, DoorStatus::Online);
        assert_eq!(f.status(&disabled).await, DoorStatus::Disabled);
        assert_eq!(f.status(&other).await, DoorStatus::Offline);
        assert!(f.service.get("ctrl-001").await.unwrap().is_online());
    }

    #[tokio::test]
    async fn disconnect_takes_doors_offline_immediately() {
        let f = fixture();
        let controller = f.service.register("ctrl-001").await.unwrap().controller;
        let door = f.door("ctrl-001").await;
        f.service.heartbeat(&controller).await.unwrap();

        f.service.disconnect(&controller).await.unwrap();

        assert_eq!(f.status(&door).await, DoorStatus::Offline);
        assert!(!f.service.get("ctrl-001").await.unwrap().is_online());
    }

    #[tokio::test]
    async fn silent_controllers_expire_and_recover_on_reconnect() {
        let f = fixture();
        let controller = f.service.register("ctrl-001").await.unwrap().controller;
        let door = f.door("ctrl-001").await;
        f.service.heartbeat(&controller).await.unwrap();

        // 29 s of silence: still fine.
        f.clock.advance(Duration::seconds(29));
        assert!(f.service.expire_stale().await.unwrap().is_empty());
        assert_eq!(f.status(&door).await, DoorStatus::Online);

        // 30 s: expired, door offline.
        f.clock.advance(Duration::seconds(1));
        let expired = f.service.expire_stale().await.unwrap();
        assert_eq!(expired.len(), 1);
        assert_eq!(f.status(&door).await, DoorStatus::Offline);

        // Reconnect: the next heartbeat restores both.
        f.service.heartbeat(&controller).await.unwrap();
        assert_eq!(f.status(&door).await, DoorStatus::Online);
    }

    #[tokio::test]
    async fn requests_are_limited_to_own_doors_and_count_as_heartbeats() {
        let f = fixture();
        let controller = f.service.register("ctrl-001").await.unwrap().controller;
        let (mine, theirs) = (f.door("ctrl-001").await, f.door("ctrl-002").await);

        f.service
            .authorize_request(&controller, mine.id())
            .await
            .unwrap();
        assert_eq!(
            f.status(&mine).await,
            DoorStatus::Online,
            "request = sign of life"
        );

        for door in [theirs.id(), DoorId::generate()] {
            assert!(matches!(
                f.service.authorize_request(&controller, door).await,
                Err(ApplicationError::Forbidden)
            ));
        }
    }
}
