use std::error::Error as StdError;

use thiserror::Error;

use super::RepositoryError;
use crate::domain::DomainError;

/// Why a use case failed, in terms the interfaces layer can map to responses
/// (e.g. `NotFound` -> 404, `Conflict` -> 409, `Domain` -> 422/409,
/// `Unauthorized` -> 401, `Forbidden` -> 403, `Internal` -> 500 with a
/// generic body).
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("{entity} not found")]
    NotFound { entity: &'static str },

    #[error("{field} is already in use")]
    Conflict { field: &'static str },

    /// Invalid input or a forbidden state change. `transparent` forwards the
    /// domain error's own message.
    #[error(transparent)]
    Domain(#[from] DomainError),

    /// Authentication failed. Deliberately says nothing about *why* (unknown
    /// user, wrong password, disabled account, expired token ...).
    #[error("invalid credentials")]
    Unauthorized,

    /// Authenticated, but not allowed to act on this resource (e.g. a
    /// controller asking about another controller's door).
    #[error("not allowed")]
    Forbidden,

    /// Storage failed or returned unusable data. Details are kept in the
    /// source chain for logs and must not be shown to API clients.
    #[error("internal error")]
    Internal(#[source] Box<dyn StdError + Send + Sync>),
}

impl ApplicationError {
    /// Wraps any unexpected failure (storage, hashing, randomness, ...).
    pub fn internal(err: impl Into<Box<dyn StdError + Send + Sync>>) -> Self {
        Self::Internal(err.into())
    }
}

impl From<RepositoryError> for ApplicationError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound { entity } => Self::NotFound { entity },
            RepositoryError::Duplicate { field } => Self::Conflict { field },
            err @ (RepositoryError::InvalidData(_) | RepositoryError::Storage(_)) => {
                Self::internal(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_errors_map_to_application_errors() {
        let not_found = ApplicationError::from(RepositoryError::NotFound { entity: "user" });
        assert!(matches!(
            not_found,
            ApplicationError::NotFound { entity: "user" }
        ));

        let duplicate = ApplicationError::from(RepositoryError::Duplicate { field: "email" });
        assert!(matches!(
            duplicate,
            ApplicationError::Conflict { field: "email" }
        ));

        let storage = ApplicationError::from(RepositoryError::Storage("boom".into()));
        assert!(matches!(storage, ApplicationError::Internal(_)));
        // The client-facing message hides the cause.
        assert_eq!(storage.to_string(), "internal error");
    }
}
