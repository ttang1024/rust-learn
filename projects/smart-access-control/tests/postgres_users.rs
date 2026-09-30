//! `PgUserRepository` against a real PostgreSQL database.

mod common;

use chrono::{TimeZone, Utc};
use common::TestDb;
use smart_access_control::{
    application::{PageRequest, RepositoryError, UserRepository},
    domain::{Email, Timestamp, User, UserId, UserName, UserStatus},
    infrastructure::postgres::PgUserRepository,
};

// Whole seconds: Postgres stores microseconds, so sub-microsecond parts
// would not survive a round trip.
fn at(hour: u32) -> Timestamp {
    Utc.with_ymd_and_hms(2026, 9, 30, hour, 0, 0).unwrap()
}

fn user(name: &str, email: &str, hour: u32) -> User {
    User::register(
        UserName::parse(name).unwrap(),
        Email::parse(email).unwrap(),
        at(hour),
    )
}

#[tokio::test]
async fn insert_and_find_round_trip() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool());
    let alice = user("Alice", "alice@example.com", 9);

    repo.insert(&alice).await.unwrap();

    assert_eq!(
        repo.find_by_id(alice.id()).await.unwrap(),
        Some(alice.clone())
    );
    assert_eq!(
        repo.find_by_email(alice.email()).await.unwrap(),
        Some(alice)
    );
    db.cleanup().await;
}

#[tokio::test]
async fn missing_user_is_none() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool());

    assert_eq!(repo.find_by_id(UserId::generate()).await.unwrap(), None);
    let email = Email::parse("nobody@example.com").unwrap();
    assert_eq!(repo.find_by_email(&email).await.unwrap(), None);
    db.cleanup().await;
}

#[tokio::test]
async fn duplicate_email_is_rejected() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool());
    repo.insert(&user("Alice", "alice@example.com", 9))
        .await
        .unwrap();

    // Different case, same normalised email.
    let err = repo
        .insert(&user("Other Alice", "ALICE@example.com", 10))
        .await
        .unwrap_err();

    assert!(
        matches!(err, RepositoryError::Duplicate { field: "email" }),
        "{err:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_persists_changes() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool());
    let mut alice = user("Alice", "alice@example.com", 9);
    repo.insert(&alice).await.unwrap();

    alice
        .rename(UserName::parse("Alice Smith").unwrap(), at(10))
        .unwrap();
    alice.suspend(at(11)).unwrap();
    repo.update(&alice).await.unwrap();

    let stored = repo.find_by_id(alice.id()).await.unwrap().unwrap();
    assert_eq!(stored.name().as_str(), "Alice Smith");
    assert_eq!(stored.status(), UserStatus::Suspended);
    assert_eq!(stored.updated_at(), at(11));
    assert_eq!(stored, alice);
    db.cleanup().await;
}

#[tokio::test]
async fn update_of_missing_user_is_not_found() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool());

    let err = repo
        .update(&user("Ghost", "ghost@example.com", 9))
        .await
        .unwrap_err();

    assert!(
        matches!(err, RepositoryError::NotFound { entity: "user" }),
        "{err:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_is_ordered_and_paginated() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool());
    // Inserted out of order on purpose.
    for (name, hour) in [("C", 12), ("A", 10), ("B", 11)] {
        let email = format!("{}@example.com", name.to_lowercase());
        repo.insert(&user(name, &email, hour)).await.unwrap();
    }

    let names =
        |users: Vec<User>| -> Vec<String> { users.iter().map(|u| u.name().to_string()).collect() };
    let first = repo.list(PageRequest::new(2, 0)).await.unwrap();
    let second = repo.list(PageRequest::new(2, 2)).await.unwrap();

    assert_eq!(names(first), ["A", "B"]);
    assert_eq!(names(second), ["C"]);
    db.cleanup().await;
}

#[tokio::test]
async fn corrupt_row_is_reported_as_invalid_data() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool());
    let alice = user("Alice", "alice@example.com", 9);
    repo.insert(&alice).await.unwrap();

    // Simulate data written outside the application that still passes the
    // table's CHECK constraints but not the domain rules.
    sqlx::query("UPDATE users SET email = 'not-an-email' WHERE id = $1")
        .bind(alice.id().as_uuid())
        .execute(&db.pool())
        .await
        .unwrap();

    let err = repo.find_by_id(alice.id()).await.unwrap_err();
    assert!(matches!(err, RepositoryError::InvalidData(_)), "{err:?}");
    db.cleanup().await;
}
