use crate::domain::Timestamp;

/// Source of the current time for use cases.
///
/// Injected rather than calling `Utc::now()` directly, so tests can pin or
/// advance time and every decision stays reproducible. `Send + Sync`
/// supertraits make `Arc<dyn Clock>` shareable across Tokio worker threads.
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}
