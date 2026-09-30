//! The access event log against a real PostgreSQL database.

mod common;

use chrono::{Duration, TimeZone, Utc};
use common::TestDb;
use smart_access_control::{
    application::{AccessEventRepository, EventFilter, PageRequest},
    application::{CardRepository, DoorRepository, UserRepository},
    domain::{
        AccessCard, AccessDecision, AccessEvent, AccessFacts, CardNumber, ControllerId,
        DenialReason, Door, DoorId, DoorName, Email, Location, Timestamp, User, UserName,
    },
    infrastructure::postgres::{
        PgAccessEventRepository, PgCardRepository, PgDoorRepository, PgUserRepository,
    },
};

fn now() -> Timestamp {
    Utc.with_ymd_and_hms(2026, 9, 30, 10, 0, 0).unwrap()
}

/// An event for an unknown card at an unknown door: references nothing.
fn anonymous_event(decision: AccessDecision, at: Timestamp) -> AccessEvent {
    let presented = CardNumber::parse("CARD-GHOST").unwrap();
    let facts = AccessFacts {
        presented_card: Some(&presented),
        requested_door: DoorId::generate(),
        card: None,
        holder: None,
        door: None,
        holder_groups: &[],
        permissions: &[],
        schedules: &[],
    };
    AccessEvent::record(&facts, decision, at)
}

#[tokio::test]
async fn every_decision_round_trips() {
    let db = TestDb::new().await;
    let repo = PgAccessEventRepository::new(db.pool());

    // One event per possible outcome proves the Rust names and the table's
    // CHECK constraint agree on every denial reason.
    let decisions = std::iter::once(AccessDecision::Granted)
        .chain(DenialReason::ALL.into_iter().map(AccessDecision::Denied));
    for decision in decisions {
        let event = anonymous_event(decision, now());
        repo.append(&event).await.unwrap();
        assert_eq!(repo.find_by_id(event.id()).await.unwrap(), Some(event));
    }
    db.cleanup().await;
}

#[tokio::test]
async fn references_existing_rows() {
    let db = TestDb::new().await;
    let user = User::register(
        UserName::parse("Alice").unwrap(),
        Email::parse("alice@example.com").unwrap(),
        now(),
    );
    PgUserRepository::new(db.pool())
        .insert(&user)
        .await
        .unwrap();
    let card = AccessCard::issue(&user, CardNumber::parse("CARD-1").unwrap(), now(), None).unwrap();
    PgCardRepository::new(db.pool())
        .insert(&card)
        .await
        .unwrap();
    let door = Door::create(
        DoorName::parse("Main").unwrap(),
        Location::parse("A").unwrap(),
        ControllerId::parse("c1").unwrap(),
        now(),
    );
    PgDoorRepository::new(db.pool())
        .insert(&door)
        .await
        .unwrap();

    let facts = AccessFacts {
        presented_card: Some(card.card_number()),
        requested_door: door.id(),
        card: Some(&card),
        holder: Some(&user),
        door: Some(&door),
        holder_groups: &[],
        permissions: &[],
        schedules: &[],
    };
    let event = AccessEvent::record(
        &facts,
        AccessDecision::Denied(DenialReason::DoorOffline),
        now(),
    );
    let repo = PgAccessEventRepository::new(db.pool());
    repo.append(&event).await.unwrap();

    let stored = repo.find_by_id(event.id()).await.unwrap().unwrap();
    assert_eq!(stored.card_id(), Some(card.id()));
    assert_eq!(stored.user_id(), Some(user.id()));
    assert_eq!(stored.door_id(), Some(door.id()));
    db.cleanup().await;
}

#[tokio::test]
async fn recent_events_are_newest_first() {
    let db = TestDb::new().await;
    let repo = PgAccessEventRepository::new(db.pool());
    let older = anonymous_event(AccessDecision::Granted, now());
    let newer = anonymous_event(AccessDecision::Granted, now() + Duration::seconds(1));
    repo.append(&older).await.unwrap();
    repo.append(&newer).await.unwrap();

    let ids: Vec<_> = repo
        .list(&EventFilter::default(), PageRequest::default())
        .await
        .unwrap()
        .iter()
        .map(AccessEvent::id)
        .collect();
    assert_eq!(ids, [newer.id(), older.id()]);
    db.cleanup().await;
}

#[tokio::test]
async fn the_log_is_append_only_even_for_raw_sql() {
    let db = TestDb::new().await;
    let repo = PgAccessEventRepository::new(db.pool());
    let event = anonymous_event(AccessDecision::Denied(DenialReason::UnknownCard), now());
    repo.append(&event).await.unwrap();

    for sql in [
        "UPDATE access_events SET decision = 'granted', reason = NULL",
        "DELETE FROM access_events",
        "TRUNCATE access_events",
    ] {
        let err = sqlx::query(sql).execute(&db.pool()).await.unwrap_err();
        assert!(
            err.to_string().contains("append-only"),
            "{sql} should be rejected, got {err}"
        );
    }

    // The original record is untouched.
    assert_eq!(repo.find_by_id(event.id()).await.unwrap(), Some(event));
    db.cleanup().await;
}

#[tokio::test]
async fn inconsistent_rows_are_rejected_by_the_database() {
    let db = TestDb::new().await;
    // A grant with a reason, a denial without one, an unknown reason.
    for (decision, reason) in [
        ("granted", Some("door_offline")),
        ("denied", None),
        ("denied", Some("because")),
    ] {
        let result = sqlx::query(
            "INSERT INTO access_events (id, decision, reason, occurred_at)
             VALUES (gen_random_uuid(), $1, $2, now())",
        )
        .bind(decision)
        .bind(reason)
        .execute(&db.pool())
        .await;
        assert!(
            result.is_err(),
            "({decision}, {reason:?}) should be rejected"
        );
    }
    db.cleanup().await;
}

/// The SQL `WHERE` clause and `EventFilter::matches` (used for live
/// WebSocket subscriptions) must select exactly the same events.
#[tokio::test]
async fn sql_filter_agrees_with_in_memory_filter() {
    use smart_access_control::application::DecisionFilter;

    let db = TestDb::new().await;
    let user = User::register(
        UserName::parse("Alice").unwrap(),
        Email::parse("alice@example.com").unwrap(),
        now(),
    );
    PgUserRepository::new(db.pool())
        .insert(&user)
        .await
        .unwrap();
    let card = AccessCard::issue(&user, CardNumber::parse("CARD-1").unwrap(), now(), None).unwrap();
    PgCardRepository::new(db.pool())
        .insert(&card)
        .await
        .unwrap();
    let doors: Vec<Door> = ["A", "B"]
        .into_iter()
        .map(|name| {
            Door::create(
                DoorName::parse(name).unwrap(),
                Location::parse("L").unwrap(),
                ControllerId::parse("c1").unwrap(),
                now(),
            )
        })
        .collect();
    for door in &doors {
        PgDoorRepository::new(db.pool()).insert(door).await.unwrap();
    }

    // 12 events: 2 doors x (known card | unknown card) x 3 outcomes, spread over time.
    let unknown = CardNumber::parse("CARD-GHOST").unwrap();
    let outcomes = [
        AccessDecision::Granted,
        AccessDecision::Denied(DenialReason::DoorOffline),
        AccessDecision::Denied(DenialReason::UnknownCard),
    ];
    let repo = PgAccessEventRepository::new(db.pool());
    let mut all = Vec::new();
    let mut minute = 0;
    for door in &doors {
        for known in [true, false] {
            for decision in outcomes {
                let facts = AccessFacts {
                    presented_card: Some(if known { card.card_number() } else { &unknown }),
                    requested_door: door.id(),
                    card: known.then_some(&card),
                    holder: known.then_some(&user),
                    door: Some(door),
                    holder_groups: &[],
                    permissions: &[],
                    schedules: &[],
                };
                let event =
                    AccessEvent::record(&facts, decision, now() + Duration::minutes(minute));
                repo.append(&event).await.unwrap();
                all.push(event);
                minute += 1;
            }
        }
    }

    let filters = [
        EventFilter::default(),
        EventFilter {
            door_id: Some(doors[0].id()),
            ..EventFilter::default()
        },
        EventFilter {
            user_id: Some(user.id()),
            ..EventFilter::default()
        },
        EventFilter {
            card_id: Some(card.id()),
            ..EventFilter::default()
        },
        EventFilter {
            decision: Some(DecisionFilter::Granted),
            ..EventFilter::default()
        },
        EventFilter {
            decision: Some(DecisionFilter::Denied),
            ..EventFilter::default()
        },
        EventFilter {
            reason: Some(DenialReason::DoorOffline),
            ..EventFilter::default()
        },
        EventFilter {
            from: Some(now() + Duration::minutes(3)),
            until: Some(now() + Duration::minutes(8)),
            ..EventFilter::default()
        },
        EventFilter {
            door_id: Some(doors[1].id()),
            user_id: Some(user.id()),
            decision: Some(DecisionFilter::Denied),
            ..EventFilter::default()
        },
    ];
    for filter in filters {
        let from_sql: Vec<_> = repo
            .list(&filter, PageRequest::new(100, 0))
            .await
            .unwrap()
            .iter()
            .map(AccessEvent::id)
            .collect();
        let mut in_memory: Vec<_> = all.iter().filter(|e| filter.matches(e)).collect();
        in_memory.sort_by_key(|e| std::cmp::Reverse(e.occurred_at()));
        let in_memory: Vec<_> = in_memory.iter().map(|e| e.id()).collect();

        assert_eq!(from_sql, in_memory, "{filter:?}");
        assert!(
            !from_sql.is_empty() || filter.user_id.is_some(),
            "filter selects something: {filter:?}"
        );
    }
    db.cleanup().await;
}
