use thiserror::Error;

/// A business rule rejected an input or a state change.
///
/// All fields are `&'static str`: building an error never allocates, the type
/// is `Copy`, and every message is fixed text that is safe to show API clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("invalid {field}: {reason}")]
    Validation {
        field: &'static str,
        reason: &'static str,
    },

    #[error("cannot {action} {entity} with status {status}")]
    InvalidTransition {
        entity: &'static str,
        action: &'static str,
        status: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_readable() {
        let err = DomainError::Validation {
            field: "email",
            reason: "must not be empty",
        };
        assert_eq!(err.to_string(), "invalid email: must not be empty");

        let err = DomainError::InvalidTransition {
            entity: "card",
            action: "suspend",
            status: "revoked",
        };
        assert_eq!(err.to_string(), "cannot suspend card with status revoked");
    }
}
