//! Serves the built dashboard (`STATIC_DIR`, normally `dashboard/dist`)
//! from the same origin as the API, so production needs no CORS and the
//! refresh-token cookie stays first-party.

use std::path::Path;

use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Response, header},
};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

/// The API's policy (`default-src 'none'`) would stop the dashboard from
/// running at all. Pages may load scripts, styles and data from this origin
/// only: no inline scripts, no third-party hosts, no framing.
const DASHBOARD_CSP: &str = "default-src 'self'; img-src 'self' data:; object-src 'none'; \
     base-uri 'none'; form-action 'self'; frame-ancestors 'none'";

/// Vite puts a content hash in every file name under `assets/`, so a
/// changed file gets a new name and the old one can be cached forever.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// `index.html` must be revalidated on every load, or browsers keep asking
/// for asset names from an old build.
const REVALIDATE: &str = "no-cache";

pub fn routes(dir: &Path) -> Router {
    let assets = Router::new()
        .fallback_service(ServeDir::new(dir.join("assets")))
        .layer(cache_control_on_success(IMMUTABLE));
    // Unknown paths get `index.html` (200): they are client-side routes
    // such as `/doors/…` that React Router resolves in the browser.
    let pages = Router::new()
        .fallback_service(ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html"))))
        .layer(cache_control_on_success(REVALIDATE));

    pages
        .nest("/assets", assets)
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(DASHBOARD_CSP),
        ))
}

/// Sets `Cache-Control` on successful responses only. Errors keep the
/// API-wide `no-store`, so a missing file is never cached for a year.
fn cache_control_on_success(
    value: &'static str,
) -> SetResponseHeaderLayer<impl Fn(&Response<Body>) -> Option<HeaderValue> + Clone> {
    SetResponseHeaderLayer::overriding(header::CACHE_CONTROL, move |response: &Response<Body>| {
        response
            .status()
            .is_success()
            .then(|| HeaderValue::from_static(value))
    })
}
