//! `PgDoorRepository` against a real PostgreSQL database.

mod common;

use chrono::{TimeZone, Utc};
use common::TestDb;
use smart_access_control::{
    application::{DoorRepository, PageRequest, RepositoryError},
    domain::{ControllerId, Door, DoorId, DoorName, DoorStatus, Location},
    infrastructure::postgres::PgDoorRepository,
};

fn door(name: &str, hour: u32) -> Door {
    Door::create(
        DoorName::parse(name).unwrap(),
        Location::parse("Building A, Level 1").unwrap(),
        ControllerId::parse("ctrl-001").unwrap(),
        Utc.with_ymd_and_hms(2026, 9, 30, hour, 0, 0).unwrap(),
    )
}

#[tokio::test]
async fn create_and_find_round_trip() {
    let db = TestDb::new().await;
    let repo = PgDoorRepository::new(db.pool());
    let main = door("Main Entrance", 9);

    repo.insert(&main).await.unwrap();

    assert_eq!(repo.find_by_id(main.id()).await.unwrap(), Some(main));
    assert_eq!(repo.find_by_id(DoorId::generate()).await.unwrap(), None);
    db.cleanup().await;
}

#[tokio::test]
async fn status_changes_are_persisted() {
    let db = TestDb::new().await;
    let repo = PgDoorRepository::new(db.pool());
    let mut main = door("Main Entrance", 9);
    repo.insert(&main).await.unwrap();

    main.mark_online().unwrap();
    repo.update(&main).await.unwrap();
    assert_eq!(
        repo.find_by_id(main.id()).await.unwrap().unwrap().status(),
        DoorStatus::Online
    );

    main.disable().unwrap();
    repo.update(&main).await.unwrap();
    assert_eq!(
        repo.find_by_id(main.id()).await.unwrap().unwrap().status(),
        DoorStatus::Disabled
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_of_missing_door_is_not_found() {
    let db = TestDb::new().await;
    let repo = PgDoorRepository::new(db.pool());

    let err = repo.update(&door("Ghost", 9)).await.unwrap_err();

    assert!(
        matches!(err, RepositoryError::NotFound { entity: "door" }),
        "{err:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_is_ordered_by_creation() {
    let db = TestDb::new().await;
    let repo = PgDoorRepository::new(db.pool());
    repo.insert(&door("Loading Dock", 11)).await.unwrap();
    repo.insert(&door("Main Entrance", 9)).await.unwrap();

    let names: Vec<String> = repo
        .list(PageRequest::default())
        .await
        .unwrap()
        .iter()
        .map(|d| d.name().to_string())
        .collect();
    assert_eq!(names, ["Main Entrance", "Loading Dock"]);
    db.cleanup().await;
}
