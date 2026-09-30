//! Administrator authentication: login, token refresh, logout.

use std::{fmt, sync::Arc};

use chrono::Duration;

use super::{AccessClaims, Password, PasswordHasher, TokenIssuer};
use crate::{
    application::{
        AdministratorRepository, ApplicationError, AuditLogRepository, Clock, ConsumeOutcome,
        RefreshTokenRecord, RefreshTokenRepository, RevocationReason,
    },
    domain::{
        Administrator, AdministratorId, AuditAction, AuditEntry, AuditSubject, Role, Timestamp,
        Username,
    },
};

/// Tokens handed to a client after login or refresh.
pub struct TokenPair {
    pub issued_at: Timestamp,
    pub access_token: String,
    pub access_expires_at: Timestamp,
    pub refresh_token: String,
    pub refresh_expires_at: Timestamp,
}

impl fmt::Debug for TokenPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenPair")
            .field("issued_at", &self.issued_at)
            .field("access_token", &"<redacted>")
            .field("access_expires_at", &self.access_expires_at)
            .field("refresh_token", &"<redacted>")
            .field("refresh_expires_at", &self.refresh_expires_at)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct CreateAdministrator {
    pub username: String,
    pub password: String,
    pub role: Role,
}

/// Four generic ports (storage + hashing) and two trait objects (tokens and
/// clock). The trait objects are needed outside this service too: the HTTP
/// layer verifies access tokens on every request.
pub struct AuthService<A, R, L, H> {
    admins: A,
    refresh_tokens: R,
    audit: L,
    hasher: H,
    tokens: Arc<dyn TokenIssuer>,
    clock: Arc<dyn Clock>,
    refresh_ttl: Duration,
}

impl<A, R, L, H> AuthService<A, R, L, H>
where
    A: AdministratorRepository,
    R: RefreshTokenRepository,
    L: AuditLogRepository,
    H: PasswordHasher,
{
    pub fn new(
        admins: A,
        refresh_tokens: R,
        audit: L,
        hasher: H,
        tokens: Arc<dyn TokenIssuer>,
        clock: Arc<dyn Clock>,
        refresh_ttl: Duration,
    ) -> Self {
        Self {
            admins,
            refresh_tokens,
            audit,
            hasher,
            tokens,
            clock,
            refresh_ttl,
        }
    }

    /// Creates an administrator account (used by the `create-admin` command).
    pub async fn create_administrator(
        &self,
        cmd: CreateAdministrator,
    ) -> Result<Administrator, ApplicationError> {
        let username = Username::parse(&cmd.username)?;
        let password = Password::new(cmd.password)?;
        let hash = self
            .hasher
            .hash(&password)
            .await
            .map_err(ApplicationError::Internal)?;

        let now = self.clock.now();
        let admin = Administrator::create(username, hash, cmd.role, now);
        self.admins.insert(&admin).await?;
        self.audit(
            None,
            AuditAction::AdministratorCreated,
            Some(admin.username()),
            now,
        )
        .await?;
        Ok(admin)
    }

    /// Exchanges a username and password for a token pair.
    ///
    /// Every failure is the same `Unauthorized`, and the password is always
    /// hashed (against a dummy if the account does not exist), so neither
    /// the response nor its timing reveals whether a username exists.
    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<TokenPair, ApplicationError> {
        let now = self.clock.now();
        let password = Password::for_login(password);
        let username = Username::parse(username).ok();

        let admin = match &username {
            Some(username) => self.admins.find_by_username(username).await?,
            None => None,
        };
        let password_ok = self
            .hasher
            .verify(&password, admin.as_ref().map(Administrator::password_hash))
            .await;

        match admin {
            Some(admin) if password_ok && admin.is_active() => {
                let pair = self.issue_pair(&admin, now).await?;
                self.audit(
                    Some(admin.id()),
                    AuditAction::LoginSucceeded,
                    Some(admin.username()),
                    now,
                )
                .await?;
                Ok(pair)
            }
            admin => {
                self.audit(
                    admin.as_ref().map(Administrator::id),
                    AuditAction::LoginFailed,
                    username.as_ref(),
                    now,
                )
                .await?;
                Err(ApplicationError::Unauthorized)
            }
        }
    }

    /// Exchanges a refresh token for a new pair ("rotation"): every refresh
    /// token works once. Presenting an already-rotated token again revokes
    /// all of the owner's sessions, because only a copy could do that.
    pub async fn refresh(&self, refresh_token: &str) -> Result<TokenPair, ApplicationError> {
        let now = self.clock.now();
        let hash = self.tokens.hash_refresh_token(refresh_token);

        let outcome = self
            .refresh_tokens
            .consume(&hash, RevocationReason::Rotated, now)
            .await?;
        let record = match outcome {
            ConsumeOutcome::Consumed(record) => record,
            ConsumeOutcome::Reused { admin_id } => {
                self.refresh_tokens.revoke_all(admin_id, now).await?;
                self.audit(Some(admin_id), AuditAction::RefreshTokenReused, None, now)
                    .await?;
                tracing::warn!(%admin_id, "refresh token reuse detected; all sessions revoked");
                metrics::counter!("auth_refresh_token_reuse_total").increment(1);
                return Err(ApplicationError::Unauthorized);
            }
            ConsumeOutcome::Revoked | ConsumeOutcome::Unknown => {
                return Err(ApplicationError::Unauthorized);
            }
        };

        if record.expires_at <= now {
            return Err(ApplicationError::Unauthorized);
        }
        // Disabling an administrator takes effect here, at the latest when
        // their current (short-lived) access token expires.
        let admin = self
            .admins
            .find_by_id(record.admin_id)
            .await?
            .filter(Administrator::is_active)
            .ok_or(ApplicationError::Unauthorized)?;

        self.issue_pair(&admin, now).await
    }

    /// Ends a session by invalidating its refresh token. Always succeeds, so
    /// the endpoint cannot be used to probe which tokens exist.
    ///
    /// The access token stays valid until it expires (stateless JWT); keeping
    /// its lifetime short bounds that window.
    pub async fn logout(&self, refresh_token: &str) -> Result<(), ApplicationError> {
        let now = self.clock.now();
        let hash = self.tokens.hash_refresh_token(refresh_token);
        let outcome = self
            .refresh_tokens
            .consume(&hash, RevocationReason::LoggedOut, now)
            .await?;
        if let ConsumeOutcome::Consumed(record) = outcome {
            self.audit(Some(record.admin_id), AuditAction::LoggedOut, None, now)
                .await?;
        }
        Ok(())
    }

    /// Checks a bearer access token. Called on every authenticated request,
    /// so it does no I/O.
    pub fn authenticate(&self, access_token: &str) -> Option<AccessClaims> {
        self.tokens
            .verify_access_token(access_token, self.clock.now())
    }

    async fn issue_pair(
        &self,
        admin: &Administrator,
        now: Timestamp,
    ) -> Result<TokenPair, ApplicationError> {
        let access = self
            .tokens
            .issue_access_token(admin, now)
            .map_err(ApplicationError::Internal)?;
        let refresh_token = self
            .tokens
            .generate_refresh_token()
            .map_err(ApplicationError::Internal)?;
        let record = RefreshTokenRecord {
            token_hash: self.tokens.hash_refresh_token(&refresh_token),
            admin_id: admin.id(),
            created_at: now,
            expires_at: now + self.refresh_ttl,
        };
        self.refresh_tokens.insert(&record).await?;

        Ok(TokenPair {
            issued_at: now,
            access_token: access.token,
            access_expires_at: access.expires_at,
            refresh_token,
            refresh_expires_at: record.expires_at,
        })
    }

    async fn audit(
        &self,
        actor: Option<AdministratorId>,
        action: AuditAction,
        subject: Option<&Username>,
        now: Timestamp,
    ) -> Result<(), ApplicationError> {
        // Usernames are at most 64 characters, well within the subject limit.
        let subject = subject
            .map(|username| AuditSubject::parse(username.as_str()))
            .transpose()?;
        let entry = AuditEntry::record(actor, action, subject, now);
        Ok(self.audit.append(&entry).await?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::{
        application::{
            PageRequest,
            fakes::{
                FakeHasher, FakeTokens, FixedClock, InMemoryAdmins, InMemoryAudit,
                InMemoryRefreshTokens,
            },
        },
        domain::DomainError,
    };

    const PASSWORD: &str = "correct horse battery staple";

    fn start() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap()
    }

    type Service = AuthService<InMemoryAdmins, InMemoryRefreshTokens, InMemoryAudit, FakeHasher>;

    struct Fixture {
        service: Service,
        admins: InMemoryAdmins,
        audit: InMemoryAudit,
        hasher: FakeHasher,
        clock: Arc<FixedClock>,
        ops: Administrator,
    }

    async fn fixture() -> Fixture {
        let (admins, audit, hasher) = (
            InMemoryAdmins::default(),
            InMemoryAudit::default(),
            FakeHasher::default(),
        );
        let clock = FixedClock::at(start());
        let service = AuthService::new(
            admins.clone(),
            InMemoryRefreshTokens::default(),
            audit.clone(),
            hasher.clone(),
            Arc::new(FakeTokens::default()),
            clock.clone(),
            Duration::days(7),
        );
        let ops = service
            .create_administrator(CreateAdministrator {
                username: "ops".into(),
                password: PASSWORD.into(),
                role: Role::Admin,
            })
            .await
            .unwrap();
        Fixture {
            service,
            admins,
            audit,
            hasher,
            clock,
            ops,
        }
    }

    impl Fixture {
        async fn actions(&self) -> Vec<AuditAction> {
            let mut entries = self
                .audit
                .list_recent(PageRequest::default())
                .await
                .unwrap();
            entries.reverse(); // oldest first reads better in assertions
            entries.iter().map(AuditEntry::action).collect()
        }
    }

    #[tokio::test]
    async fn create_administrator_hashes_and_audits() {
        let f = fixture().await;
        assert_ne!(f.ops.password_hash().as_str(), PASSWORD);
        assert_eq!(f.actions().await, [AuditAction::AdministratorCreated]);
    }

    #[tokio::test]
    async fn create_administrator_enforces_password_policy_and_unique_names() {
        let f = fixture().await;
        let err = f
            .service
            .create_administrator(CreateAdministrator {
                username: "weak".into(),
                password: "short".into(),
                role: Role::Viewer,
            })
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::Validation {
                field: "password",
                ..
            })
        ));

        let err = f
            .service
            .create_administrator(CreateAdministrator {
                username: "OPS".into(),
                password: PASSWORD.into(),
                role: Role::Viewer,
            })
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Conflict { field: "username" }
        ));
    }

    #[tokio::test]
    async fn login_issues_tokens_that_authenticate() {
        let f = fixture().await;

        let pair = f.service.login("OPS", PASSWORD).await.unwrap();

        let claims = f.service.authenticate(&pair.access_token).unwrap();
        assert_eq!(claims.admin_id, f.ops.id());
        assert_eq!(claims.role, Role::Admin);
        assert_eq!(pair.refresh_expires_at, start() + Duration::days(7));
        assert_eq!(
            f.actions().await,
            [
                AuditAction::AdministratorCreated,
                AuditAction::LoginSucceeded
            ]
        );
    }

    #[tokio::test]
    async fn wrong_password_is_unauthorized_and_audited() {
        let f = fixture().await;
        let err = f
            .service
            .login("ops", "wrong password!!")
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Unauthorized));
        assert_eq!(f.actions().await.last(), Some(&AuditAction::LoginFailed));
    }

    #[tokio::test]
    async fn unknown_user_still_costs_a_hash() {
        let f = fixture().await;
        let before = f.hasher.verify_calls();

        let err = f.service.login("nobody", PASSWORD).await.unwrap_err();

        assert!(matches!(err, ApplicationError::Unauthorized));
        assert_eq!(
            f.hasher.verify_calls(),
            before + 1,
            "dummy verification must run"
        );
        assert_eq!(f.hasher.last_verify_had_hash(), Some(false));
    }

    #[tokio::test]
    async fn disabled_administrator_cannot_log_in() {
        let f = fixture().await;
        f.admins.update_with(f.ops.id(), Administrator::disable);

        let err = f.service.login("ops", PASSWORD).await.unwrap_err();
        assert!(matches!(err, ApplicationError::Unauthorized));
    }

    #[tokio::test]
    async fn access_tokens_expire() {
        let f = fixture().await;
        let pair = f.service.login("ops", PASSWORD).await.unwrap();

        f.clock.advance(pair.access_expires_at - start());
        assert!(f.service.authenticate(&pair.access_token).is_none());
    }

    #[tokio::test]
    async fn refresh_rotates_tokens() {
        let f = fixture().await;
        let first = f.service.login("ops", PASSWORD).await.unwrap();

        let second = f.service.refresh(&first.refresh_token).await.unwrap();

        assert_ne!(second.refresh_token, first.refresh_token);
        assert!(f.service.authenticate(&second.access_token).is_some());
        // The next rotation works with the new token.
        assert!(f.service.refresh(&second.refresh_token).await.is_ok());
    }

    #[tokio::test]
    async fn refresh_token_reuse_revokes_every_session() {
        let f = fixture().await;
        let laptop = f.service.login("ops", PASSWORD).await.unwrap();
        let phone = f.service.login("ops", PASSWORD).await.unwrap();
        let rotated = f.service.refresh(&laptop.refresh_token).await.unwrap();

        // An attacker replays the laptop's old refresh token.
        let err = f.service.refresh(&laptop.refresh_token).await.unwrap_err();
        assert!(matches!(err, ApplicationError::Unauthorized));

        // Every session of this administrator is now dead.
        assert!(f.service.refresh(&rotated.refresh_token).await.is_err());
        assert!(f.service.refresh(&phone.refresh_token).await.is_err());
        assert_eq!(
            f.actions().await.last(),
            Some(&AuditAction::RefreshTokenReused)
        );
    }

    #[tokio::test]
    async fn expired_refresh_token_is_rejected() {
        let f = fixture().await;
        let pair = f.service.login("ops", PASSWORD).await.unwrap();
        f.clock.advance(Duration::days(7));

        assert!(matches!(
            f.service.refresh(&pair.refresh_token).await,
            Err(ApplicationError::Unauthorized)
        ));
    }

    #[tokio::test]
    async fn disabled_administrator_cannot_refresh() {
        let f = fixture().await;
        let pair = f.service.login("ops", PASSWORD).await.unwrap();
        f.admins.update_with(f.ops.id(), Administrator::disable);

        assert!(matches!(
            f.service.refresh(&pair.refresh_token).await,
            Err(ApplicationError::Unauthorized)
        ));
    }

    #[tokio::test]
    async fn unknown_refresh_token_is_rejected() {
        let f = fixture().await;
        assert!(matches!(
            f.service.refresh("made-up").await,
            Err(ApplicationError::Unauthorized)
        ));
    }

    #[tokio::test]
    async fn logout_invalidates_the_refresh_token() {
        let f = fixture().await;
        let pair = f.service.login("ops", PASSWORD).await.unwrap();

        f.service.logout(&pair.refresh_token).await.unwrap();

        assert!(f.service.refresh(&pair.refresh_token).await.is_err());
        // Logging out again, or with nonsense, is not an error.
        f.service.logout(&pair.refresh_token).await.unwrap();
        f.service.logout("nonsense").await.unwrap();
        assert!(f.actions().await.contains(&AuditAction::LoggedOut));
    }

    #[tokio::test]
    async fn retrying_a_logged_out_token_does_not_end_other_sessions() {
        let f = fixture().await;
        let laptop = f.service.login("ops", PASSWORD).await.unwrap();
        let phone = f.service.login("ops", PASSWORD).await.unwrap();
        f.service.logout(&laptop.refresh_token).await.unwrap();

        // A buggy client retries its logged-out token: rejected...
        assert!(f.service.refresh(&laptop.refresh_token).await.is_err());
        // ...but that is not treated as theft.
        assert!(f.service.refresh(&phone.refresh_token).await.is_ok());
        assert!(!f.actions().await.contains(&AuditAction::RefreshTokenReused));
    }
}
