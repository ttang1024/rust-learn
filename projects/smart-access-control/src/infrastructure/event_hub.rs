//! In-process fan-out of access events to live subscribers (WebSockets).
//!
//! # Delivery guarantees
//!
//! - **Best effort, at most once.** Events live only in memory. A subscriber
//!   that is not connected when an event is published never receives it.
//! - **Never blocks the publisher.** `broadcast::Sender::send` only writes to
//!   a fixed-size ring buffer; it never waits for subscribers.
//! - **Slow subscribers lose events, not the system.** Each subscriber
//!   reads at its own pace. If one falls more than `capacity` events behind,
//!   its oldest unread events are overwritten and its next `recv` reports
//!   `Lagged(n)`, so it knows exactly how many it missed.
//! - **Ordered.** Each subscriber sees events in publication order.
//! - **Single process.** Subscribers of another server instance are not
//!   reached; that would need a shared broker (e.g. Postgres LISTEN/NOTIFY).
//!
//! PostgreSQL remains the source of truth: every published event was stored
//! first, and clients can fill gaps via `GET /events?from=...`.

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::{application::EventPublisher, domain::AccessEvent};

#[derive(Debug, Clone)]
pub struct BroadcastEventHub {
    sender: broadcast::Sender<Arc<AccessEvent>>,
}

impl BroadcastEventHub {
    /// How many events a subscriber may fall behind before it starts losing them.
    pub const DEFAULT_CAPACITY: usize = 1024;

    pub fn new(capacity: usize) -> Self {
        // The initial receiver is dropped: subscribers come and go later.
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<AccessEvent>> {
        self.sender.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for BroadcastEventHub {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

impl EventPublisher for BroadcastEventHub {
    fn publish(&self, event: &AccessEvent) {
        // One allocation shared by every subscriber: the channel clones the
        // `Arc` (a reference-count increment), not the event.
        // `send` fails only when nobody is subscribed, which is fine.
        let _ = self.sender.send(Arc::new(event.clone()));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use tokio::sync::broadcast::error::RecvError;

    use super::*;
    use crate::{
        application::events::tests::event,
        domain::{AccessDecision, DoorId},
    };

    fn granted(minute: i64) -> AccessEvent {
        event(DoorId::generate(), AccessDecision::Granted, minute)
    }

    #[tokio::test]
    async fn every_subscriber_receives_every_event_in_order() {
        let hub = BroadcastEventHub::default();
        let (mut a, mut b) = (hub.subscribe(), hub.subscribe());
        let (first, second) = (granted(0), granted(1));

        hub.publish(&first);
        hub.publish(&second);

        for rx in [&mut a, &mut b] {
            assert_eq!(*rx.recv().await.unwrap(), first);
            assert_eq!(*rx.recv().await.unwrap(), second);
        }
    }

    #[test]
    fn publishing_without_subscribers_is_fine() {
        BroadcastEventHub::default().publish(&granted(0));
    }

    #[tokio::test]
    async fn a_slow_subscriber_is_told_how_many_events_it_missed() {
        let hub = BroadcastEventHub::new(4);
        let mut slow = hub.subscribe();

        for minute in 0..10 {
            hub.publish(&granted(minute));
        }

        // 10 published, room for 4: the 6 oldest were overwritten.
        assert!(matches!(slow.recv().await, Err(RecvError::Lagged(6))));
        // It then continues with the oldest event still buffered.
        assert_eq!(
            slow.recv().await.unwrap().occurred_at(),
            granted(6).occurred_at()
        );
    }

    #[test]
    fn a_subscriber_that_never_reads_cannot_block_publishing() {
        let hub = BroadcastEventHub::new(16);
        let _stuck = hub.subscribe();
        let events: Vec<AccessEvent> = (0..10_000).map(granted).collect();

        let started = Instant::now();
        for event in &events {
            hub.publish(event);
        }
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn dropped_subscribers_are_released() {
        let hub = BroadcastEventHub::default();
        let rx = hub.subscribe();
        assert_eq!(hub.subscriber_count(), 1);
        drop(rx);
        assert_eq!(hub.subscriber_count(), 0);
    }
}
