//! One JSON error shape for every failure:
//! `{"error": {"code": "not_found", "message": "user not found"}}`.

use std::borrow::Cow;

use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{application::ApplicationError, domain::DomainError, error_chain};

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: Cow<'static, str>,
    /// `WWW-Authenticate` value for 401 responses (RFC 7235).
    challenge: Option<&'static str>,
    /// Seconds for a `Retry-After` header (429 responses).
    retry_after: Option<i64>,
}

impl ApiError {
    pub fn new(
        status: StatusCode,
        code: &'static str,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            challenge: None,
            retry_after: None,
        }
    }

    /// 429: too many attempts; the client may retry after `seconds`.
    pub fn too_many_requests(seconds: i64) -> Self {
        Self {
            retry_after: Some(seconds.max(1)),
            ..Self::new(
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_requests",
                "too many attempts; try again later",
            )
        }
    }

    /// A 401 for the admin API, which uses bearer tokens (RFC 6750).
    pub fn unauthorized() -> Self {
        Self {
            challenge: Some("Bearer"),
            ..Self::new(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "invalid credentials",
            )
        }
    }

    pub fn forbidden() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "your role does not allow this action",
        )
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: ErrorDetail<'a>,
}

#[derive(Serialize)]
struct ErrorDetail<'a> {
    code: &'a str,
    message: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: &self.message,
            },
        });
        let mut response = (self.status, body).into_response();
        if let Some(seconds) = self.retry_after
            && let Ok(value) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }
        if let Some(challenge) = self.challenge {
            // Tell the client which authentication scheme to use.
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(challenge),
            );
        }
        response
    }
}

/// Maps use-case failures to HTTP. Internal details are logged, never sent.
impl From<ApplicationError> for ApiError {
    fn from(err: ApplicationError) -> Self {
        match err {
            ApplicationError::NotFound { entity } => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("{entity} not found"),
            ),
            ApplicationError::Conflict { field } => Self::new(
                StatusCode::CONFLICT,
                "conflict",
                format!("{field} is already in use"),
            ),
            ApplicationError::Domain(err @ DomainError::Validation { .. }) => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                err.to_string(),
            ),
            ApplicationError::Domain(err @ DomainError::InvalidTransition { .. }) => {
                Self::new(StatusCode::CONFLICT, "invalid_state", err.to_string())
            }
            ApplicationError::Unauthorized => Self::unauthorized(),
            ApplicationError::Forbidden => Self::forbidden(),
            ApplicationError::Internal(source) => {
                tracing::error!(error = %error_chain(source.as_ref()), "internal error");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal error",
                )
            }
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        ApplicationError::from(err).into()
    }
}

/// `axum::Json`, but malformed bodies produce our JSON error shape instead of
/// Axum's plain-text rejection (400 bad JSON, 415 wrong content type, 413
/// body too large, 422 wrong fields).
pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(json_rejection(rejection)),
        }
    }
}

fn json_rejection(rejection: JsonRejection) -> ApiError {
    let code = match rejection.status() {
        StatusCode::PAYLOAD_TOO_LARGE => "payload_too_large",
        StatusCode::UNSUPPORTED_MEDIA_TYPE => "unsupported_media_type",
        _ => "invalid_request",
    };
    ApiError::new(rejection.status(), code, rejection.body_text())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_errors_map_to_statuses() {
        let cases = [
            (
                ApplicationError::NotFound { entity: "user" },
                404,
                "not_found",
            ),
            (
                ApplicationError::Conflict { field: "email" },
                409,
                "conflict",
            ),
            (
                ApplicationError::Domain(DomainError::Validation {
                    field: "email",
                    reason: "must not be empty",
                }),
                422,
                "validation_failed",
            ),
            (
                ApplicationError::Domain(DomainError::InvalidTransition {
                    entity: "card",
                    action: "revoke",
                    status: "revoked",
                }),
                409,
                "invalid_state",
            ),
            (ApplicationError::Unauthorized, 401, "unauthorized"),
            (
                ApplicationError::internal("db exploded: password=hunter2"),
                500,
                "internal_error",
            ),
        ];
        for (err, status, code) in cases {
            let api = ApiError::from(err);
            assert_eq!(api.status.as_u16(), status);
            assert_eq!(api.code, code);
        }
    }

    #[test]
    fn internal_details_are_not_exposed() {
        let api = ApiError::from(ApplicationError::internal("db exploded: password=hunter2"));
        assert_eq!(api.message, "internal error");
    }
}
