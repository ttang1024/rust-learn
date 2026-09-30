//! Interfaces layer: external entry points (REST API, WebSockets).
//!
//! Handlers stay thin: validate input, call an application use case, map the
//! result to a response. No business logic lives here.

pub mod background;
pub mod http;
