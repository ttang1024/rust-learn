//! End to end: management use cases set up policy in PostgreSQL, then the
//! decision service evaluates requests against it and records every attempt.

mod common;

use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use chrono::{Duration, NaiveTime, TimeZone, Utc, Weekday};
use common::TestDb;
use smart_access_control::{
    application::{
        AccessDecisionService, AccessEventRepository, AccessGroupService, AccessRequest,
        CardService, Clock, CreateDoor, CreateGroup, CreateSchedule, DoorService, EventFilter,
        GrantPermission, IssueCard, PageRequest, PermissionService, RegisterUser, RuleInput,
        ScheduleService, UserService,
    },
    domain::{
        AccessCard, AccessDecision, AccessEvent, AccessGroup, DenialReason, Door, DoorId,
        Timestamp, User,
    },
    infrastructure::event_hub::BroadcastEventHub,
    infrastructure::postgres::{
        PgAccessDataSource, PgAccessEventRepository, PgAccessGroupRepository, PgCardRepository,
        PgDoorRepository, PgPermissionRepository, PgScheduleRepository, PgUserRepository,
    },
};

/// Wednesday 2026-09-30 10:00 UTC.
fn start() -> Timestamp {
    Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap()
}

/// A clock the test can move; the application only sees `dyn Clock`.
struct TestClock(Mutex<Timestamp>);

impl Clock for TestClock {
    fn now(&self) -> Timestamp {
        *self.0.lock().unwrap()
    }
}

impl TestClock {
    fn set(&self, to: Timestamp) {
        *self.0.lock().unwrap() = to;
    }
}

struct App {
    clock: Arc<TestClock>,
    users: UserService<PgUserRepository>,
    cards: CardService<PgCardRepository, PgUserRepository>,
    doors: DoorService<PgDoorRepository>,
    groups: AccessGroupService<PgAccessGroupRepository>,
    schedules: ScheduleService<PgScheduleRepository>,
    permissions: PermissionService<PgPermissionRepository>,
    access: AccessDecisionService<PgAccessDataSource, PgAccessEventRepository>,
    events: PgAccessEventRepository,
}

/// Wires every service to one database, as `main` will in Phase 4.
fn app(db: &TestDb) -> App {
    let clock = Arc::new(TestClock(Mutex::new(start())));
    let dyn_clock: Arc<dyn Clock> = clock.clone();
    let pool = db.pool();
    let users = PgUserRepository::new(pool.clone());
    let events = PgAccessEventRepository::new(pool.clone());
    App {
        users: UserService::new(users.clone(), dyn_clock.clone()),
        cards: CardService::new(
            PgCardRepository::new(pool.clone()),
            users,
            dyn_clock.clone(),
        ),
        doors: DoorService::new(PgDoorRepository::new(pool.clone()), dyn_clock.clone()),
        groups: AccessGroupService::new(PgAccessGroupRepository::new(pool.clone())),
        schedules: ScheduleService::new(PgScheduleRepository::new(pool.clone())),
        permissions: PermissionService::new(
            PgPermissionRepository::new(pool.clone()),
            dyn_clock.clone(),
        ),
        access: AccessDecisionService::new(
            PgAccessDataSource::new(pool),
            events.clone(),
            Arc::new(BroadcastEventHub::default()),
            dyn_clock,
        ),
        events,
        clock,
    }
}

/// Alice holds CARD-10001, is in "Staff", and Staff may use the (online)
/// main entrance at any time.
struct World {
    alice: User,
    card: AccessCard,
    door: Door,
    staff: AccessGroup,
}

async fn world(app: &App) -> World {
    let alice = app
        .users
        .register(RegisterUser {
            name: "Alice".into(),
            email: "alice@example.com".into(),
        })
        .await
        .unwrap();
    let card = app
        .cards
        .issue(IssueCard {
            user_id: alice.id(),
            card_number: "CARD-10001".into(),
            expires_at: Some(start() + Duration::days(365)),
        })
        .await
        .unwrap();
    let door = app
        .doors
        .create(CreateDoor {
            name: "Main Entrance".into(),
            location: "Building A".into(),
            controller_id: "ctrl-001".into(),
        })
        .await
        .unwrap();
    let door = app.doors.report_online(door.id()).await.unwrap();
    let staff = app
        .groups
        .create(CreateGroup {
            name: "Staff".into(),
            description: None,
        })
        .await
        .unwrap();
    app.groups.add_member(staff.id(), alice.id()).await.unwrap();
    app.permissions
        .grant(GrantPermission {
            group_id: staff.id(),
            door_id: door.id(),
            schedule_id: None,
        })
        .await
        .unwrap();
    World {
        alice,
        card,
        door,
        staff,
    }
}

async fn request(app: &App, card_number: &str, door_id: DoorId) -> AccessEvent {
    app.access
        .decide(AccessRequest {
            card_number: card_number.into(),
            door_id,
        })
        .await
        .unwrap()
}

fn denied(reason: DenialReason) -> AccessDecision {
    AccessDecision::Denied(reason)
}

/// Compile-time check that the decision future can run on Axum's
/// multi-threaded runtime (see `tests/use_cases.rs` for the reasoning).
#[allow(dead_code)]
fn decision_future_is_send(app: &App) {
    fn assert_send<F: Future + Send>(_: F) {}
    assert_send(app.access.decide(AccessRequest {
        card_number: String::new(),
        door_id: DoorId::generate(),
    }));
}

#[tokio::test]
async fn granted_request_is_recorded() {
    let db = TestDb::new().await;
    let app = app(&db);
    let w = world(&app).await;

    let event = request(&app, "card-10001", w.door.id()).await;

    assert_eq!(event.decision(), AccessDecision::Granted);
    assert_eq!(event.card_id(), Some(w.card.id()));
    assert_eq!(event.user_id(), Some(w.alice.id()));
    assert_eq!(event.door_id(), Some(w.door.id()));
    assert_eq!(event.occurred_at(), start());
    // Persisted exactly as returned.
    assert_eq!(
        app.events.find_by_id(event.id()).await.unwrap(),
        Some(event)
    );
    db.cleanup().await;
}

#[tokio::test]
async fn unknown_card_and_unknown_door_are_recorded_without_phantom_references() {
    let db = TestDb::new().await;
    let app = app(&db);
    let w = world(&app).await;

    let unknown_card = request(&app, "CARD-99999", w.door.id()).await;
    assert_eq!(unknown_card.decision(), denied(DenialReason::UnknownCard));
    assert_eq!(
        unknown_card.card_number().map(|n| n.as_str()),
        Some("CARD-99999")
    );
    assert_eq!(unknown_card.card_id(), None);
    assert_eq!(unknown_card.door_id(), Some(w.door.id()));

    let unknown_door = request(&app, "CARD-10001", DoorId::generate()).await;
    assert_eq!(unknown_door.decision(), denied(DenialReason::UnknownDoor));
    assert_eq!(unknown_door.door_id(), None);

    let malformed = request(&app, "'; DROP TABLE users; --", w.door.id()).await;
    assert_eq!(malformed.decision(), denied(DenialReason::UnknownCard));
    assert_eq!(malformed.card_number(), None);

    let recorded = app
        .events
        .list(&EventFilter::default(), PageRequest::default())
        .await
        .unwrap();
    assert_eq!(recorded.len(), 3);
    db.cleanup().await;
}

#[tokio::test]
async fn door_state_is_respected() {
    let db = TestDb::new().await;
    let app = app(&db);
    let w = world(&app).await;

    app.doors.report_offline(w.door.id()).await.unwrap();
    let event = request(&app, "CARD-10001", w.door.id()).await;
    assert_eq!(event.decision(), denied(DenialReason::DoorOffline));

    app.doors.disable(w.door.id()).await.unwrap();
    let event = request(&app, "CARD-10001", w.door.id()).await;
    assert_eq!(event.decision(), denied(DenialReason::DoorDisabled));
    db.cleanup().await;
}

#[tokio::test]
async fn policy_changes_take_effect_immediately() {
    let db = TestDb::new().await;
    let app = app(&db);
    let w = world(&app).await;
    assert!(
        request(&app, "CARD-10001", w.door.id())
            .await
            .decision()
            .is_granted()
    );

    app.groups
        .remove_member(w.staff.id(), w.alice.id())
        .await
        .unwrap();
    let event = request(&app, "CARD-10001", w.door.id()).await;
    assert_eq!(event.decision(), denied(DenialReason::PermissionDenied));

    app.groups
        .add_member(w.staff.id(), w.alice.id())
        .await
        .unwrap();
    app.users.suspend(w.alice.id()).await.unwrap();
    let event = request(&app, "CARD-10001", w.door.id()).await;
    assert_eq!(event.decision(), denied(DenialReason::UserSuspended));

    app.users.reactivate(w.alice.id()).await.unwrap();
    app.cards.revoke(w.card.id()).await.unwrap();
    let event = request(&app, "CARD-10001", w.door.id()).await;
    assert_eq!(event.decision(), denied(DenialReason::CardRevoked));
    db.cleanup().await;
}

#[tokio::test]
async fn deleting_the_group_removes_access() {
    let db = TestDb::new().await;
    let app = app(&db);
    let w = world(&app).await;

    app.groups.delete(w.staff.id()).await.unwrap();

    let event = request(&app, "CARD-10001", w.door.id()).await;
    assert_eq!(event.decision(), denied(DenialReason::PermissionDenied));
    db.cleanup().await;
}

#[tokio::test]
async fn scheduled_permission_follows_local_time() {
    let db = TestDb::new().await;
    let app = app(&db);
    let w = world(&app).await;

    // Replace 24/7 access with a New York business-hours schedule. Deleting
    // and re-creating the group drops the old permission.
    app.groups.delete(w.staff.id()).await.unwrap();
    let day_staff = app
        .groups
        .create(CreateGroup {
            name: "Day staff".into(),
            description: None,
        })
        .await
        .unwrap();
    app.groups
        .add_member(day_staff.id(), w.alice.id())
        .await
        .unwrap();
    let hours = app
        .schedules
        .create(CreateSchedule {
            name: "NY business hours".into(),
            timezone: "America/New_York".into(),
            rules: vec![RuleInput {
                days: vec![
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                ],
                start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            }],
            effective_from: None,
            effective_until: None,
        })
        .await
        .unwrap();
    app.permissions
        .grant(GrantPermission {
            group_id: day_staff.id(),
            door_id: w.door.id(),
            schedule_id: Some(hours.id()),
        })
        .await
        .unwrap();

    // 10:00 UTC = 06:00 in New York (EDT): too early.
    let early = request(&app, "CARD-10001", w.door.id()).await;
    assert_eq!(early.decision(), denied(DenialReason::OutsideSchedule));

    // 13:00 UTC = 09:00 in New York: the window opens.
    app.clock.set(start() + Duration::hours(3));
    let on_time = request(&app, "CARD-10001", w.door.id()).await;
    assert_eq!(on_time.decision(), AccessDecision::Granted);
    assert_eq!(on_time.occurred_at(), start() + Duration::hours(3));
    db.cleanup().await;
}
