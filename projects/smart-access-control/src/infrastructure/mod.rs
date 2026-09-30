//! Infrastructure layer: concrete implementations of application traits
//! (PostgreSQL repositories, JWT, event broadcasting, device simulators)
//! plus process-level concerns such as logging and shutdown.

pub mod auth;
pub mod clock;
pub mod event_hub;
pub mod metrics;
pub mod postgres;
pub mod shutdown;
pub mod telemetry;
