//! Controller storage against a real PostgreSQL database.

mod common;

use chrono::{Duration, TimeZone, Utc};
use common::TestDb;
use smart_access_control::{
    application::{ControllerRepository, PageRequest, RepositoryError},
    domain::{Controller, ControllerId, ControllerStatus, Timestamp},
    infrastructure::{auth::sha256_hex, postgres::PgControllerRepository},
};

fn at(second: i64) -> Timestamp {
    Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap() + Duration::seconds(second)
}

fn controller(id: &str) -> Controller {
    Controller::register(ControllerId::parse(id).unwrap(), at(0))
}

#[tokio::test]
async fn round_trip_lookup_by_key_and_unique_ids() {
    let db = TestDb::new().await;
    let repo = PgControllerRepository::new(db.pool());
    let mut c = controller("ctrl-001");
    repo.insert(&c, &sha256_hex("key-1")).await.unwrap();

    assert_eq!(repo.find_by_id(c.id()).await.unwrap(), Some(c.clone()));
    assert_eq!(
        repo.find_by_key_hash(&sha256_hex("key-1")).await.unwrap(),
        Some(c.clone())
    );
    assert_eq!(
        repo.find_by_key_hash(&sha256_hex("other")).await.unwrap(),
        None
    );

    c.record_heartbeat(at(5));
    repo.update(&c).await.unwrap();
    assert_eq!(repo.find_by_id(c.id()).await.unwrap(), Some(c.clone()));

    repo.set_key_hash(c.id(), &sha256_hex("key-2"))
        .await
        .unwrap();
    assert_eq!(
        repo.find_by_key_hash(&sha256_hex("key-1")).await.unwrap(),
        None
    );
    assert!(
        repo.find_by_key_hash(&sha256_hex("key-2"))
            .await
            .unwrap()
            .is_some()
    );

    let err = repo
        .insert(&controller("ctrl-001"), &sha256_hex("key-3"))
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            RepositoryError::Duplicate {
                field: "controller_id"
            }
        ),
        "{err:?}"
    );
    assert_eq!(repo.list(PageRequest::default()).await.unwrap().len(), 1);
    db.cleanup().await;
}

#[tokio::test]
async fn database_refuses_a_plaintext_key() {
    let db = TestDb::new().await;
    let result = PgControllerRepository::new(db.pool())
        .insert(&controller("ctrl-001"), "my-secret-key")
        .await;
    assert!(result.is_err(), "only SHA-256 hex digests may be stored");
    db.cleanup().await;
}

#[tokio::test]
async fn expiry_uses_the_same_boundary_as_the_domain() {
    let db = TestDb::new().await;
    let repo = PgControllerRepository::new(db.pool());
    let timeout = Duration::seconds(30);
    let (mut exact, mut fresh, never) = (
        controller("exact"),
        controller("fresh"),
        controller("never"),
    );
    exact.record_heartbeat(at(0));
    fresh.record_heartbeat(at(1));
    for (n, c) in [&exact, &fresh, &never].into_iter().enumerate() {
        repo.insert(c, &sha256_hex(&format!("key-{n}")))
            .await
            .unwrap();
    }

    let now = at(30);
    let expired = repo.expire_stale(now - timeout).await.unwrap();

    // The SQL and `Controller::is_stale` agree for each controller.
    for c in [&exact, &fresh, &never] {
        assert_eq!(
            expired.contains(c.id()),
            c.is_stale(now, timeout),
            "{}",
            c.id()
        );
    }
    assert_eq!(expired, [exact.id().clone()]);
    let stored = repo.find_by_id(exact.id()).await.unwrap().unwrap();
    assert_eq!(stored.status(), ControllerStatus::Offline);
    // Running it again finds nothing new.
    assert!(repo.expire_stale(now - timeout).await.unwrap().is_empty());
    db.cleanup().await;
}
