//! Administrator, refresh-token and audit storage against real PostgreSQL.

mod common;

use chrono::{Duration, TimeZone, Utc};
use common::TestDb;
use smart_access_control::{
    application::{
        AdministratorRepository, AuditLogRepository, ConsumeOutcome, PageRequest,
        RefreshTokenRecord, RefreshTokenRepository, RepositoryError, RevocationReason,
    },
    domain::{
        Administrator, AdministratorId, AuditAction, AuditEntry, AuditSubject, PasswordHash, Role,
        Timestamp, Username,
    },
    infrastructure::postgres::{
        PgAdministratorRepository, PgAuditLogRepository, PgRefreshTokenRepository,
    },
};

fn now() -> Timestamp {
    Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap()
}

fn admin(username: &str) -> Administrator {
    Administrator::create(
        Username::parse(username).unwrap(),
        PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA".into()),
        Role::Admin,
        now(),
    )
}

async fn saved_admin(db: &TestDb, username: &str) -> Administrator {
    let admin = admin(username);
    PgAdministratorRepository::new(db.pool())
        .insert(&admin)
        .await
        .unwrap();
    admin
}

/// A syntactically valid (64 hex chars) token hash.
fn token_hash(n: u8) -> String {
    format!("{n:02x}").repeat(32)
}

fn record(admin_id: AdministratorId, n: u8) -> RefreshTokenRecord {
    RefreshTokenRecord {
        token_hash: token_hash(n),
        admin_id,
        created_at: now(),
        expires_at: now() + Duration::days(7),
    }
}

#[tokio::test]
async fn administrator_round_trip_and_unique_username() {
    let db = TestDb::new().await;
    let repo = PgAdministratorRepository::new(db.pool());
    let ops = admin("ops");
    repo.insert(&ops).await.unwrap();

    assert_eq!(repo.find_by_id(ops.id()).await.unwrap(), Some(ops.clone()));
    assert_eq!(
        repo.find_by_username(ops.username()).await.unwrap(),
        Some(ops)
    );

    let err = repo.insert(&admin("ops")).await.unwrap_err();
    assert!(
        matches!(err, RepositoryError::Duplicate { field: "username" }),
        "{err:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn database_refuses_anything_but_an_argon2id_hash() {
    let db = TestDb::new().await;
    let plaintext = Administrator::create(
        Username::parse("oops").unwrap(),
        PasswordHash::new("hunter2hunter2".into()),
        Role::Admin,
        now(),
    );
    let result = PgAdministratorRepository::new(db.pool())
        .insert(&plaintext)
        .await;
    assert!(result.is_err(), "a plaintext password must never be stored");
    db.cleanup().await;
}

#[tokio::test]
async fn consume_outcomes_depend_on_why_a_token_died() {
    let db = TestDb::new().await;
    let ops = saved_admin(&db, "ops").await;
    let repo = PgRefreshTokenRepository::new(db.pool());
    for n in 1..=2 {
        repo.insert(&record(ops.id(), n)).await.unwrap();
    }

    // Token 1: rotated, then presented again -> Reused (theft signal).
    let first = repo
        .consume(&token_hash(1), RevocationReason::Rotated, now())
        .await
        .unwrap();
    assert_eq!(first, ConsumeOutcome::Consumed(record(ops.id(), 1)));
    let again = repo
        .consume(&token_hash(1), RevocationReason::Rotated, now())
        .await
        .unwrap();
    assert_eq!(again, ConsumeOutcome::Reused { admin_id: ops.id() });

    // Token 2: logged out, then presented again -> Revoked (not theft).
    repo.consume(&token_hash(2), RevocationReason::LoggedOut, now())
        .await
        .unwrap();
    let again = repo
        .consume(&token_hash(2), RevocationReason::Rotated, now())
        .await
        .unwrap();
    assert_eq!(again, ConsumeOutcome::Revoked);

    let unknown = repo
        .consume(&token_hash(9), RevocationReason::Rotated, now())
        .await
        .unwrap();
    assert_eq!(unknown, ConsumeOutcome::Unknown);
    db.cleanup().await;
}

#[tokio::test]
async fn revoke_all_only_touches_that_administrator() {
    let db = TestDb::new().await;
    let (ops, other) = (
        saved_admin(&db, "ops").await,
        saved_admin(&db, "other").await,
    );
    let repo = PgRefreshTokenRepository::new(db.pool());
    repo.insert(&record(ops.id(), 1)).await.unwrap();
    repo.insert(&record(ops.id(), 2)).await.unwrap();
    repo.insert(&record(other.id(), 3)).await.unwrap();

    repo.revoke_all(ops.id(), now()).await.unwrap();

    for n in [1, 2] {
        let outcome = repo
            .consume(&token_hash(n), RevocationReason::Rotated, now())
            .await
            .unwrap();
        assert_eq!(outcome, ConsumeOutcome::Revoked);
    }
    let outcome = repo
        .consume(&token_hash(3), RevocationReason::Rotated, now())
        .await
        .unwrap();
    assert!(matches!(outcome, ConsumeOutcome::Consumed(_)));
    db.cleanup().await;
}

#[tokio::test]
async fn concurrent_refreshes_with_one_token_have_exactly_one_winner() {
    let db = TestDb::new().await;
    let ops = saved_admin(&db, "ops").await;
    let repo = PgRefreshTokenRepository::new(db.pool());
    repo.insert(&record(ops.id(), 1)).await.unwrap();

    // Ten tasks race to rotate the same token at the same moment.
    let attempts: Vec<_> = (0..10)
        .map(|_| {
            let repo = repo.clone();
            tokio::spawn(async move {
                repo.consume(&token_hash(1), RevocationReason::Rotated, now())
                    .await
                    .unwrap()
            })
        })
        .collect();
    let mut winners = 0;
    for attempt in attempts {
        match attempt.await.unwrap() {
            ConsumeOutcome::Consumed(_) => winners += 1,
            ConsumeOutcome::Reused { .. } => {}
            other => panic!("unexpected outcome {other:?}"),
        }
    }
    assert_eq!(winners, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn audit_log_round_trip_and_append_only() {
    let db = TestDb::new().await;
    let ops = saved_admin(&db, "ops").await;
    let repo = PgAuditLogRepository::new(db.pool());

    for (i, &action) in AuditAction::ALL.iter().enumerate() {
        let entry = AuditEntry::record(
            Some(ops.id()),
            action,
            Some(AuditSubject::parse("ops").unwrap()),
            now() + Duration::seconds(i as i64),
        );
        repo.append(&entry).await.unwrap();
    }
    let anonymous = AuditEntry::record(
        None,
        AuditAction::LoginFailed,
        None,
        now() + Duration::hours(1),
    );
    repo.append(&anonymous).await.unwrap();

    let recent = repo.list_recent(PageRequest::default()).await.unwrap();
    assert_eq!(recent.len(), AuditAction::ALL.len() + 1);
    assert_eq!(recent[0], anonymous, "newest first");

    for sql in [
        "UPDATE admin_audit_log SET subject = 'x'",
        "DELETE FROM admin_audit_log",
    ] {
        let err = sqlx::query(sql).execute(&db.pool()).await.unwrap_err();
        assert!(
            err.to_string().contains("admin_audit_log is append-only"),
            "{err}"
        );
    }
    db.cleanup().await;
}
