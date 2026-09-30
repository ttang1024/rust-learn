//! Tracing (structured logging) setup.

use tracing_subscriber::EnvFilter;

const DEFAULT_FILTER: &str = "smart_access_control=debug,tower_http=debug";

/// Log output format, from `LOG_FORMAT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable, coloured (development).
    Pretty,
    /// One JSON object per line, with the current span's fields (such as
    /// `request_id`) included: for log collectors in production.
    Json,
}

impl LogFormat {
    /// `None` or an unknown value falls back to `Pretty`; the second value
    /// reports whether the input was unknown, so it can be logged once the
    /// subscriber exists.
    pub fn parse(raw: Option<&str>) -> (Self, bool) {
        match raw.map(str::trim) {
            None | Some("" | "pretty") => (Self::Pretty, false),
            Some("json") => (Self::Json, false),
            Some(_) => (Self::Pretty, true),
        }
    }
}

/// Installs the global tracing subscriber.
///
/// The filter comes from `RUST_LOG` when set, e.g.
/// `RUST_LOG=smart_access_control=trace,tower_http=debug`.
pub fn init(format: LogFormat) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    match format {
        LogFormat::Pretty => builder.init(),
        LogFormat::Json => builder
            .json()
            // Fields of the enclosing span (the request span carries
            // `request_id`) on every line, without the full span stack.
            .with_current_span(true)
            .with_span_list(false)
            .init(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_format_parsing() {
        assert_eq!(LogFormat::parse(None), (LogFormat::Pretty, false));
        assert_eq!(LogFormat::parse(Some("json")), (LogFormat::Json, false));
        assert_eq!(
            LogFormat::parse(Some(" pretty ")),
            (LogFormat::Pretty, false)
        );
        assert_eq!(LogFormat::parse(Some("xml")), (LogFormat::Pretty, true));
    }
}
