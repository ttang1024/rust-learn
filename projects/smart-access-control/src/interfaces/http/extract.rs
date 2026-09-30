//! Extractor wrappers that turn Axum's plain-text rejections into our JSON
//! errors, plus pagination types shared by list endpoints.

use std::fmt::Display;

use axum::{
    extract::{FromRequestParts, Path, Query},
    http::request::Parts,
};
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};

use super::ApiError;
use crate::application::PageRequest;

/// `axum::extract::Path` with JSON errors, e.g. a malformed UUID -> 400.
pub struct ApiPath<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        Path::<T>::from_request_parts(parts, state)
            .await
            .map(|Path(value)| Self(value))
            .map_err(|rejection| {
                ApiError::new(rejection.status(), "invalid_path", rejection.body_text())
            })
    }
}

/// `axum::extract::Query` with JSON errors.
pub struct ApiQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(value)| Self(value))
            .map_err(|rejection| {
                ApiError::new(rejection.status(), "invalid_query", rejection.body_text())
            })
    }
}

/// `?limit=&offset=` on list endpoints.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl From<PageParams> for PageRequest {
    fn from(params: PageParams) -> Self {
        page_request(params.limit, params.offset)
    }
}

pub fn page_request(limit: Option<u32>, offset: Option<u32>) -> PageRequest {
    PageRequest::new(
        limit.unwrap_or(PageRequest::DEFAULT_LIMIT),
        offset.unwrap_or(0),
    )
}

/// A page of results. `limit` is the effective (clamped) limit.
#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub limit: u32,
    pub offset: u32,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, page: PageRequest) -> Self {
        Self {
            items,
            limit: page.limit(),
            offset: page.offset(),
        }
    }

    /// Converts each entity into its response type.
    pub fn of<'a, E>(entities: &'a [E], page: PageRequest) -> Self
    where
        T: From<&'a E>,
    {
        Self::new(entities.iter().map(T::from).collect(), page)
    }
}

/// Distinguishes an absent field from an explicit `null` in PATCH bodies.
///
/// Use with `#[serde(default, deserialize_with = "double_option")]`:
/// field missing -> `None` (via `default`), `null` -> `Some(None)`,
/// a value -> `Some(Some(value))`. Plain `Option<Option<T>>` would turn
/// both "missing" and `null` into `None`.
pub fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// `kind:id`, the subject format used in audit entries.
pub fn subject(kind: &str, id: impl Display) -> String {
    format!("{kind}:{id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Patch {
        #[serde(default, deserialize_with = "double_option")]
        description: Option<Option<String>>,
    }

    #[test]
    fn double_option_distinguishes_missing_null_and_value() {
        let parse = |json| serde_json::from_str::<Patch>(json).unwrap().description;
        assert_eq!(parse("{}"), None);
        assert_eq!(parse(r#"{"description": null}"#), Some(None));
        assert_eq!(parse(r#"{"description": "x"}"#), Some(Some("x".into())));
    }
}
