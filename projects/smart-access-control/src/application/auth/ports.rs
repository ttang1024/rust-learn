//! Ports for authentication: password hashing and token handling.

use std::{error::Error as StdError, fmt, future::Future};

use super::Password;
use crate::domain::{Administrator, AdministratorId, PasswordHash, Role, Timestamp};

pub type BoxError = Box<dyn StdError + Send + Sync>;

pub trait PasswordHasher: Send + Sync {
    fn hash(
        &self,
        password: &Password,
    ) -> impl Future<Output = Result<PasswordHash, BoxError>> + Send;

    /// Checks `password` against `hash`.
    ///
    /// With `hash = None` (no such account) an implementation must still do
    /// the same amount of work and return `false`, so response times do not
    /// reveal which usernames exist.
    fn verify(
        &self,
        password: &Password,
        hash: Option<&PasswordHash>,
    ) -> impl Future<Output = bool> + Send;
}

/// Creates and hashes high-entropy secrets (e.g. controller keys).
pub trait SecretGenerator: Send + Sync {
    /// A new, unguessable secret.
    fn generate(&self) -> Result<String, BoxError>;

    /// The form in which the secret is stored and looked up.
    fn hash(&self, secret: &str) -> String;
}

/// What a valid access token says about its bearer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessClaims {
    pub admin_id: AdministratorId,
    pub role: Role,
    pub expires_at: Timestamp,
}

pub struct IssuedAccessToken {
    pub token: String,
    pub expires_at: Timestamp,
}

impl fmt::Debug for IssuedAccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IssuedAccessToken")
            .field("token", &"<redacted>")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// Creates and checks tokens. Synchronous (no I/O), which also keeps it
/// usable as a trait object (`Arc<dyn TokenIssuer>`).
pub trait TokenIssuer: Send + Sync {
    fn issue_access_token(
        &self,
        admin: &Administrator,
        now: Timestamp,
    ) -> Result<IssuedAccessToken, BoxError>;

    /// `None` for any invalid token. The reason (bad signature, expired,
    /// malformed ...) is deliberately not exposed to callers.
    fn verify_access_token(&self, token: &str, now: Timestamp) -> Option<AccessClaims>;

    /// A new, unguessable refresh token.
    fn generate_refresh_token(&self) -> Result<String, BoxError>;

    /// The form in which refresh tokens are stored and looked up.
    fn hash_refresh_token(&self, token: &str) -> String;
}
