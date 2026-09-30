//! Use cases wired to the real PostgreSQL repositories and system clock,
//! exactly as the application will compose them.

mod common;

use std::{future::Future, sync::Arc};

use chrono::Duration;
use common::TestDb;
use smart_access_control::{
    application::{
        ApplicationError, CardService, Clock, CreateDoor, DoorService, IssueCard, RegisterUser,
        UserService,
    },
    domain::{CardStatus, DoorStatus, UserStatus},
    infrastructure::{
        clock::SystemClock,
        postgres::{PgCardRepository, PgDoorRepository, PgUserRepository},
    },
};

struct Services {
    users: UserService<PgUserRepository>,
    cards: CardService<PgCardRepository, PgUserRepository>,
    doors: DoorService<PgDoorRepository>,
}

fn services(db: &TestDb) -> Services {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let users = PgUserRepository::new(db.pool());
    Services {
        users: UserService::new(users.clone(), clock.clone()),
        cards: CardService::new(PgCardRepository::new(db.pool()), users, clock.clone()),
        doors: DoorService::new(PgDoorRepository::new(db.pool()), clock),
    }
}

fn alice() -> RegisterUser {
    RegisterUser {
        name: "Alice".into(),
        email: "alice@example.com".into(),
    }
}

/// Compile-time check: Axum runs handlers on a multi-threaded runtime, so the
/// futures returned by services must be `Send`. If a non-`Send` value (e.g. a
/// `std::sync::MutexGuard`) were ever held across an `.await` inside a use
/// case, this function would stop compiling.
#[allow(dead_code)]
fn service_futures_are_send(s: &Services) {
    fn assert_send<F: Future + Send>(_: F) {}
    assert_send(s.users.register(alice()));
    assert_send(
        s.cards
            .revoke(smart_access_control::domain::CardId::generate()),
    );
    assert_send(s.doors.create(CreateDoor {
        name: String::new(),
        location: String::new(),
        controller_id: String::new(),
    }));
}

#[tokio::test]
async fn user_and_card_lifecycle() {
    let db = TestDb::new().await;
    let s = services(&db);

    let user = s.users.register(alice()).await.unwrap();
    let card = s
        .cards
        .issue(IssueCard {
            user_id: user.id(),
            card_number: "CARD-10001".into(),
            expires_at: Some(user.created_at() + Duration::days(365)),
        })
        .await
        .unwrap();

    // Real clock timestamps survive the database round trip unchanged
    // (the system clock truncates to microseconds).
    assert_eq!(s.users.get(user.id()).await.unwrap(), user);
    assert_eq!(s.cards.get(card.id()).await.unwrap(), card);

    s.cards.revoke(card.id()).await.unwrap();
    s.users.archive(user.id()).await.unwrap();

    assert_eq!(
        s.cards.get(card.id()).await.unwrap().status(),
        CardStatus::Revoked
    );
    assert_eq!(
        s.users.get(user.id()).await.unwrap().status(),
        UserStatus::Archived
    );
    db.cleanup().await;
}

#[tokio::test]
async fn conflicts_come_from_the_database_too() {
    let db = TestDb::new().await;
    let s = services(&db);
    s.users.register(alice()).await.unwrap();

    let err = s
        .users
        .register(RegisterUser {
            name: "Alice Again".into(),
            email: "ALICE@example.com".into(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, ApplicationError::Conflict { field: "email" }));
    db.cleanup().await;
}

#[tokio::test]
async fn door_lifecycle() {
    let db = TestDb::new().await;
    let s = services(&db);

    let door = s
        .doors
        .create(CreateDoor {
            name: "Main Entrance".into(),
            location: "Building A".into(),
            controller_id: "ctrl-001".into(),
        })
        .await
        .unwrap();
    s.doors.disable(door.id()).await.unwrap();

    let stored = s.doors.get(door.id()).await.unwrap();
    assert_eq!(stored.status(), DoorStatus::Disabled);
    assert_eq!(stored.created_at(), door.created_at());
    db.cleanup().await;
}
