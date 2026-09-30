//! `/auth/*` endpoints and the extractors that protect other routes.

use axum::{
    Json,
    extract::{FromRef, FromRequestParts, State},
    http::{HeaderMap, HeaderValue, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ApiError, ApiJson, AppState, client_ip::ClientIp};
use crate::application::{AccessClaims, ApplicationError, RetryAfter, TokenPair};

// ---- extractors ----

/// Any logged-in administrator. Add it as a handler argument and the route
/// requires a valid `Authorization: Bearer <access token>` header.
#[derive(Debug, Clone, Copy)]
pub struct AuthenticatedAdmin(pub AccessClaims);

/// Generic over the router state `S`: it works on any router whose state
/// can produce an `AppState` (`FromRef`), not only on the main router.
impl<S> FromRequestParts<S> for AuthenticatedAdmin
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let token = bearer_token(parts).ok_or_else(ApiError::unauthorized)?;
        AppState::from_ref(state)
            .auth
            .authenticate(token)
            .map(Self)
            .ok_or_else(ApiError::unauthorized)
    }
}

/// A logged-in administrator whose role may change configuration.
/// 401 without a valid token, 403 with a valid token but insufficient role.
#[derive(Debug, Clone, Copy)]
pub struct RequireAdmin(pub AccessClaims);

impl<S> FromRequestParts<S> for RequireAdmin
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let AuthenticatedAdmin(claims) =
            AuthenticatedAdmin::from_request_parts(parts, state).await?;
        if !claims.role.can_manage() {
            return Err(ApiError::forbidden());
        }
        Ok(Self(claims))
    }
}

/// Extracts the token from `Authorization: Bearer <token>`. The scheme name
/// is case-insensitive (RFC 7235).
fn bearer_token(parts: &Parts) -> Option<&str> {
    let value = parts.headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    (scheme.eq_ignore_ascii_case("bearer") && !token.trim().is_empty()).then(|| token.trim())
}

// ---- DTOs ----

/// No `Debug` derive: the struct carries a password.
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    /// How to deliver the refresh token; `body` if omitted.
    #[serde(default)]
    pub session: SessionMode,
}

/// Where the refresh token travels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    /// In the JSON body, for API clients that store it themselves.
    #[default]
    Body,
    /// Only in an `HttpOnly` cookie, for browsers: page scripts can never
    /// read it, so an XSS bug cannot steal a long-lived session.
    Cookie,
}

/// Refresh and logout take the token from the body, or else from the cookie.
#[derive(Deserialize)]
pub struct RefreshRequest {
    #[serde(default)]
    pub refresh_token: Option<String>,
}

/// OAuth 2.0-style token response (RFC 6749 §5.1).
#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    /// Seconds until the access token expires.
    pub expires_in: i64,
    /// Absent in cookie mode: the token is only in the `HttpOnly` cookie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Seconds until the refresh token expires.
    pub refresh_expires_in: i64,
}

/// Name of the refresh-token cookie.
pub const REFRESH_COOKIE: &str = "sac_refresh";

/// Builds the token response for `mode`: the refresh token goes either in
/// the body or in an `HttpOnly` cookie, never both.
fn token_response(pair: TokenPair, mode: SessionMode, cookie_secure: bool) -> Response {
    let refresh_expires_in = (pair.refresh_expires_at - pair.issued_at).num_seconds();
    let (refresh_token, cookie) = match mode {
        SessionMode::Body => (Some(pair.refresh_token), None),
        SessionMode::Cookie => (
            None,
            Some(refresh_cookie(
                &pair.refresh_token,
                refresh_expires_in,
                cookie_secure,
            )),
        ),
    };
    let body = TokenResponse {
        expires_in: (pair.access_expires_at - pair.issued_at).num_seconds(),
        refresh_expires_in,
        access_token: pair.access_token,
        token_type: "Bearer",
        refresh_token,
    };
    let mut response = no_store(Json(body));
    if let Some(cookie) = cookie {
        response.headers_mut().insert(header::SET_COOKIE, cookie);
    }
    response
}

/// `HttpOnly`: invisible to page scripts. `SameSite=Strict`: never sent on
/// cross-site requests (CSRF). `Path`: only sent to the auth endpoints.
/// An empty value with `Max-Age=0` deletes the cookie.
fn refresh_cookie(token: &str, max_age: i64, secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    let cookie = format!(
        "{REFRESH_COOKIE}={token}; HttpOnly{secure}; SameSite=Strict; Path=/api/v1/auth; Max-Age={max_age}"
    );
    // The token is hex and everything else is fixed ASCII.
    HeaderValue::from_str(&cookie).expect("cookie is valid header text")
}

/// The refresh token from the `Cookie` header, if present.
fn cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| {
            let (name, value) = pair.trim().split_once('=')?;
            (name == REFRESH_COOKIE && !value.is_empty()).then(|| value.to_owned())
        })
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub role: &'static str,
}

/// Token responses must never be cached by browsers or proxies (RFC 6749 §5.1).
fn no_store(response: impl IntoResponse) -> Response {
    let mut response = response.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

// ---- handlers ----

/// `POST /api/v1/auth/login`
///
/// Throttled per account name and per client address (see `LoginThrottle`):
/// once blocked, even a correct password gets `429` until the block ends.
pub async fn login(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    ApiJson(body): ApiJson<LoginRequest>,
) -> Result<Response, ApiError> {
    if let Err(RetryAfter(wait)) = state.login_throttle.check(&body.username, ip) {
        tracing::warn!(
            client_ip = ip.map(tracing::field::display),
            "login throttled"
        );
        metrics::counter!("auth_login_attempts_total", "outcome" => "throttled").increment(1);
        return Err(ApiError::too_many_requests(wait.num_seconds()));
    }
    match state.auth.login(&body.username, &body.password).await {
        Ok(pair) => {
            state.login_throttle.record_success(&body.username);
            metrics::counter!("auth_login_attempts_total", "outcome" => "success").increment(1);
            Ok(token_response(pair, body.session, state.cookie_secure))
        }
        Err(ApplicationError::Unauthorized) => {
            state.login_throttle.record_failure(&body.username, ip);
            metrics::counter!("auth_login_attempts_total", "outcome" => "failure").increment(1);
            Err(ApiError::unauthorized())
        }
        Err(other) => Err(other.into()),
    }
}

/// `POST /api/v1/auth/refresh`
///
/// A token from the cookie is answered in cookie mode (a new cookie), a
/// token from the body in body mode. The body must be JSON even when empty
/// (`{}`), so a cross-site HTML form cannot trigger this endpoint.
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    ApiJson(body): ApiJson<RefreshRequest>,
) -> Result<Response, ApiError> {
    let (token, mode) = match body.refresh_token {
        Some(token) => (token, SessionMode::Body),
        None => (
            cookie_token(&headers).ok_or_else(ApiError::unauthorized)?,
            SessionMode::Cookie,
        ),
    };
    let pair = state.auth.refresh(&token).await?;
    Ok(token_response(pair, mode, state.cookie_secure))
}

/// `POST /api/v1/auth/logout`: always 204, whether or not the token existed,
/// and always deletes the refresh cookie.
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    ApiJson(body): ApiJson<RefreshRequest>,
) -> Result<Response, ApiError> {
    if let Some(token) = body.refresh_token.or_else(|| cookie_token(&headers)) {
        state.auth.logout(&token).await?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        refresh_cookie("", 0, state.cookie_secure),
    );
    Ok(response)
}

/// `GET /api/v1/auth/me`: who the access token belongs to.
pub async fn me(AuthenticatedAdmin(claims): AuthenticatedAdmin) -> Json<MeResponse> {
    Json(MeResponse {
        id: claims.admin_id.as_uuid(),
        role: claims.role.as_str(),
    })
}

#[cfg(test)]
mod tests {
    use axum::http::Request;

    use super::*;

    fn parts_with(authorization: Option<&str>) -> Parts {
        let mut builder = Request::builder();
        if let Some(value) = authorization {
            builder = builder.header(header::AUTHORIZATION, value);
        }
        builder.body(()).unwrap().into_parts().0
    }

    #[test]
    fn bearer_token_parsing() {
        assert_eq!(bearer_token(&parts_with(Some("Bearer abc"))), Some("abc"));
        assert_eq!(bearer_token(&parts_with(Some("bearer  abc "))), Some("abc"));
        assert_eq!(bearer_token(&parts_with(Some("Basic abc"))), None);
        assert_eq!(bearer_token(&parts_with(Some("Bearer "))), None);
        assert_eq!(bearer_token(&parts_with(Some("Bearer"))), None);
        assert_eq!(bearer_token(&parts_with(None)), None);
    }
}
