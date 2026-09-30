//! Argon2id password hashing, run off the async runtime.

use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher as _, PasswordVerifier as _},
};

use crate::{
    application::{BoxError, Password, PasswordHasher},
    domain::PasswordHash,
};

/// Argon2id with the library defaults (m = 19 MiB, t = 2, p = 1), which
/// match the OWASP minimum recommendation.
///
/// Each hash costs tens of milliseconds of CPU *on purpose*. Running that
/// inside an async task would block a Tokio worker thread and stall every
/// other request scheduled on it, so the work goes to Tokio's dedicated
/// blocking thread pool via `spawn_blocking`.
#[derive(Debug, Clone)]
pub struct Argon2PasswordHasher {
    /// Hash of a random password, verified against when an account does not
    /// exist so that "unknown user" costs the same time as "wrong password".
    dummy: Arc<str>,
}

impl Argon2PasswordHasher {
    /// Computes the dummy hash once (tens of milliseconds, at startup).
    pub fn new() -> Result<Self, BoxError> {
        let mut random = [0u8; 32];
        getrandom::fill(&mut random).map_err(|err| format!("getrandom failed: {err}"))?;
        Self::with_dummy_password(&random)
    }

    fn with_dummy_password(password: &[u8]) -> Result<Self, BoxError> {
        let dummy = Argon2::default().hash_password(password)?.to_string();
        Ok(Self {
            dummy: dummy.into(),
        })
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: &Password) -> Result<PasswordHash, BoxError> {
        // `spawn_blocking` needs a `'static` closure, so it cannot borrow
        // `password`; give it an owned copy instead.
        let password = password.expose().to_owned();
        let phc = tokio::task::spawn_blocking(move || {
            Argon2::default()
                .hash_password(password.as_bytes())
                .map(|hash| hash.to_string())
        })
        .await??;
        Ok(PasswordHash::new(phc))
    }

    async fn verify(&self, password: &Password, hash: Option<&PasswordHash>) -> bool {
        let is_real = hash.is_some();
        let phc: Arc<str> = match hash {
            Some(hash) => hash.as_str().into(),
            None => Arc::clone(&self.dummy),
        };
        let password = password.expose().to_owned();
        let matches = tokio::task::spawn_blocking(move || {
            Argon2::default()
                .verify_password(password.as_bytes(), &*phc)
                .is_ok()
        })
        .await
        .unwrap_or_else(|err| {
            tracing::error!(%err, "password verification task failed");
            false
        });
        // Even if someone guessed the dummy's random password, it must not log in.
        is_real && matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn password(raw: &str) -> Password {
        Password::new(raw.into()).unwrap()
    }

    #[tokio::test]
    async fn hash_and_verify() {
        let hasher = Argon2PasswordHasher::new().unwrap();
        let hash = hasher
            .hash(&password("correct horse battery"))
            .await
            .unwrap();

        assert!(hash.as_str().starts_with("$argon2id$v=19$"));
        assert!(
            hasher
                .verify(&password("correct horse battery"), Some(&hash))
                .await
        );
        assert!(
            !hasher
                .verify(&password("wrong horse battery"), Some(&hash))
                .await
        );
    }

    #[tokio::test]
    async fn same_password_gets_a_different_salt_each_time() {
        let hasher = Argon2PasswordHasher::new().unwrap();
        let a = hasher
            .hash(&password("correct horse battery"))
            .await
            .unwrap();
        let b = hasher
            .hash(&password("correct horse battery"))
            .await
            .unwrap();
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn missing_account_never_verifies() {
        let hasher = Argon2PasswordHasher::new().unwrap();
        assert!(!hasher.verify(&password("anything at all"), None).await);
    }

    #[tokio::test]
    async fn knowing_the_dummy_password_does_not_help() {
        // In production the dummy password is random and never known. Even
        // if it were, "no such account" must still fail.
        let hasher = Argon2PasswordHasher::with_dummy_password(b"known dummy password").unwrap();
        assert!(!hasher.verify(&password("known dummy password"), None).await);
    }

    #[tokio::test]
    async fn malformed_stored_hash_does_not_verify() {
        let hasher = Argon2PasswordHasher::new().unwrap();
        let garbage = PasswordHash::new("not a phc string".into());
        assert!(
            !hasher
                .verify(&password("anything at all"), Some(&garbage))
                .await
        );
    }
}
