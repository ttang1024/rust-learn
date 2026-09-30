//! `PgCardRepository` against a real PostgreSQL database.

mod common;

use chrono::{TimeZone, Utc};
use common::TestDb;
use smart_access_control::{
    application::{CardRepository, PageRequest, RepositoryError, UserRepository},
    domain::{AccessCard, CardNumber, CardStatus, Email, Timestamp, User, UserName},
    infrastructure::postgres::{PgCardRepository, PgUserRepository},
};

fn at(hour: u32) -> Timestamp {
    Utc.with_ymd_and_hms(2026, 9, 30, hour, 0, 0).unwrap()
}

fn user(email: &str) -> User {
    User::register(
        UserName::parse("Card Holder").unwrap(),
        Email::parse(email).unwrap(),
        at(8),
    )
}

fn card(owner: &User, number: &str, hour: u32) -> AccessCard {
    AccessCard::issue(
        owner,
        CardNumber::parse(number).unwrap(),
        at(hour),
        Some(at(hour) + chrono::Duration::days(365)),
    )
    .unwrap()
}

/// Inserts a user and returns it with both repositories.
async fn setup(db: &TestDb) -> (PgUserRepository, PgCardRepository, User) {
    let users = PgUserRepository::new(db.pool());
    let cards = PgCardRepository::new(db.pool());
    let owner = user("holder@example.com");
    users.insert(&owner).await.unwrap();
    (users, cards, owner)
}

#[tokio::test]
async fn issue_and_find_round_trip() {
    let db = TestDb::new().await;
    let (_, cards, owner) = setup(&db).await;
    let issued = card(&owner, "CARD-10001", 9);

    cards.insert(&issued).await.unwrap();

    assert_eq!(
        cards.find_by_id(issued.id()).await.unwrap(),
        Some(issued.clone())
    );
    assert_eq!(
        cards.find_by_number(issued.card_number()).await.unwrap(),
        Some(issued)
    );
    db.cleanup().await;
}

#[tokio::test]
async fn card_without_expiry_round_trips() {
    let db = TestDb::new().await;
    let (_, cards, owner) = setup(&db).await;
    let issued = AccessCard::issue(
        &owner,
        CardNumber::parse("CARD-NOEXP").unwrap(),
        at(9),
        None,
    )
    .unwrap();

    cards.insert(&issued).await.unwrap();

    let stored = cards.find_by_id(issued.id()).await.unwrap().unwrap();
    assert_eq!(stored.expires_at(), None);
    db.cleanup().await;
}

#[tokio::test]
async fn duplicate_card_number_is_rejected() {
    let db = TestDb::new().await;
    let (_, cards, owner) = setup(&db).await;
    cards.insert(&card(&owner, "CARD-10001", 9)).await.unwrap();

    let err = cards
        .insert(&card(&owner, "card-10001", 10))
        .await
        .unwrap_err();

    assert!(
        matches!(
            err,
            RepositoryError::Duplicate {
                field: "card_number"
            }
        ),
        "{err:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn card_for_unknown_user_is_rejected() {
    let db = TestDb::new().await;
    let cards = PgCardRepository::new(db.pool());
    let never_saved = user("ghost@example.com");

    let err = cards
        .insert(&card(&never_saved, "CARD-GHOST", 9))
        .await
        .unwrap_err();

    assert!(
        matches!(err, RepositoryError::NotFound { entity: "user" }),
        "{err:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn revocation_is_persisted() {
    let db = TestDb::new().await;
    let (_, cards, owner) = setup(&db).await;
    let mut issued = card(&owner, "CARD-10001", 9);
    cards.insert(&issued).await.unwrap();

    issued.revoke().unwrap();
    cards.update(&issued).await.unwrap();

    let stored = cards.find_by_id(issued.id()).await.unwrap().unwrap();
    assert_eq!(stored.status(), CardStatus::Revoked);
    db.cleanup().await;
}

#[tokio::test]
async fn lists_cards_per_user_and_paginated() {
    let db = TestDb::new().await;
    let (users, cards, owner) = setup(&db).await;
    let other = user("other@example.com");
    users.insert(&other).await.unwrap();

    cards.insert(&card(&owner, "CARD-B", 10)).await.unwrap();
    cards.insert(&card(&other, "CARD-X", 11)).await.unwrap();
    cards.insert(&card(&owner, "CARD-A", 9)).await.unwrap();

    let numbers = |list: Vec<AccessCard>| -> Vec<String> {
        list.iter().map(|c| c.card_number().to_string()).collect()
    };
    assert_eq!(
        numbers(cards.list_for_user(owner.id()).await.unwrap()),
        ["CARD-A", "CARD-B"]
    );
    assert_eq!(
        numbers(cards.list(PageRequest::new(2, 1)).await.unwrap()),
        ["CARD-B", "CARD-X"]
    );
    db.cleanup().await;
}
