use std::fmt;

use crate::domain::DomainError;

/// A plaintext password, only ever held briefly in memory.
///
/// `Debug` is redacted so a password can never end up in a log line by
/// accident, and there is no `Display` at all.
#[derive(Clone)]
pub struct Password(String);

impl Password {
    pub const MIN_CHARS: usize = 12;
    /// Upper bound so a client cannot make us hash megabytes (a cheap DoS).
    pub const MAX_CHARS: usize = 128;

    /// For setting a password: enforces the length policy.
    ///
    /// Length is the only rule, following current NIST guidance: long
    /// passphrases beat composition rules like "one digit, one symbol".
    pub fn new(raw: String) -> Result<Self, DomainError> {
        let chars = raw.chars().count();
        if chars < Self::MIN_CHARS {
            return Err(DomainError::Validation {
                field: "password",
                reason: "must be at least 12 characters",
            });
        }
        if chars > Self::MAX_CHARS {
            return Err(DomainError::Validation {
                field: "password",
                reason: "must be at most 128 characters",
            });
        }
        Ok(Self(raw))
    }

    /// For checking a login attempt: only the upper bound applies, so the
    /// response never reveals the password policy.
    pub fn for_login(raw: &str) -> Self {
        Self(raw.chars().take(Self::MAX_CHARS + 1).collect())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_policy() {
        assert!(Password::new("short".into()).is_err());
        assert!(Password::new("x".repeat(11)).is_err());
        assert!(Password::new("x".repeat(12)).is_ok());
        assert!(Password::new("x".repeat(128)).is_ok());
        assert!(Password::new("x".repeat(129)).is_err());
        // Characters, not bytes: 12 two-byte characters are enough.
        assert!(Password::new("é".repeat(12)).is_ok());
    }

    #[test]
    fn login_input_is_capped_not_rejected() {
        assert_eq!(Password::for_login("abc").expose(), "abc");
        assert_eq!(Password::for_login(&"x".repeat(10_000)).expose().len(), 129);
    }

    #[test]
    fn debug_is_redacted() {
        let password = Password::new("correct horse battery".into()).unwrap();
        assert_eq!(format!("{password:?}"), "Password(<redacted>)");
    }
}
