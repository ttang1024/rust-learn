//! Background tasks: entry points into the application that are driven by
//! time rather than by requests.

use std::time::Duration;

use tokio::task::JoinHandle;

use crate::{error_chain, interfaces::http::AppState};

/// Periodically marks controllers that stopped sending heartbeats offline.
///
/// Checks three times per timeout period (at least once a second), so a
/// silent controller is detected at most a third of the timeout late. Stops
/// when the server shuts down.
pub fn spawn_liveness_monitor(state: AppState) -> JoinHandle<()> {
    let timeout = state
        .controllers
        .timeout()
        .to_std()
        .unwrap_or(Duration::from_secs(30));
    let period = (timeout / 3).max(Duration::from_secs(1));

    tokio::spawn(async move {
        let mut ticks = tokio::time::interval(period);
        // If a sweep runs long, do not fire a burst of catch-up ticks.
        ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = ticks.tick() => {
                    if let Err(err) = state.controllers.expire_stale().await {
                        // Keep running: the next tick retries.
                        tracing::error!(error = %error_chain(&err), "controller liveness check failed");
                    }
                }
                () = state.shutdown_requested() => break,
            }
        }
        tracing::debug!("liveness monitor stopped");
    })
}
