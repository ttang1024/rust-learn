//! Groups, schedules and permissions against a real PostgreSQL database.

mod common;

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use common::TestDb;
use smart_access_control::{
    application::{
        AccessGroupRepository, DoorRepository, PageRequest, PermissionRepository, RepositoryError,
        ScheduleRepository, UserRepository,
    },
    domain::{
        AccessGroup, AccessGroupId, AccessPermission, AccessSchedule, ControllerId, DaySet,
        Description, Door, DoorId, DoorName, Email, GroupName, Location, ScheduleId, ScheduleName,
        ScheduleRule, Timestamp, User, UserId, UserName, parse_time_zone,
    },
    infrastructure::postgres::{
        PgAccessGroupRepository, PgDoorRepository, PgPermissionRepository, PgScheduleRepository,
        PgUserRepository,
    },
};

fn now() -> Timestamp {
    Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap()
}

fn group(name: &str) -> AccessGroup {
    AccessGroup::create(
        GroupName::parse(name).unwrap(),
        Some(Description::parse("Test group").unwrap()),
    )
}

async fn saved_user(db: &TestDb) -> User {
    let user = User::register(
        UserName::parse("Alice").unwrap(),
        Email::parse("alice@example.com").unwrap(),
        now(),
    );
    PgUserRepository::new(db.pool())
        .insert(&user)
        .await
        .unwrap();
    user
}

async fn saved_door(db: &TestDb) -> Door {
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
    door
}

fn schedule(name: &str) -> AccessSchedule {
    let t = |h| NaiveTime::from_hms_opt(h, 0, 0).unwrap();
    AccessSchedule::new(
        ScheduleName::parse(name).unwrap(),
        parse_time_zone("Europe/London").unwrap(),
        vec![
            ScheduleRule::new(DaySet::WEEKDAYS, t(9), t(17)),
            ScheduleRule::new(DaySet::from_days([Weekday::Sat]).unwrap(), t(22), t(6)),
        ],
        Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        None,
    )
    .unwrap()
}

// ---- access groups ----

#[tokio::test]
async fn group_crud_round_trip() {
    let db = TestDb::new().await;
    let repo = PgAccessGroupRepository::new(db.pool());
    let mut staff = group("Staff");

    repo.insert(&staff).await.unwrap();
    assert_eq!(
        repo.find_by_id(staff.id()).await.unwrap(),
        Some(staff.clone())
    );

    staff.rename(GroupName::parse("All Staff").unwrap());
    staff.set_description(None);
    repo.update(&staff).await.unwrap();
    assert_eq!(
        repo.find_by_id(staff.id()).await.unwrap(),
        Some(staff.clone())
    );

    repo.delete(staff.id()).await.unwrap();
    assert_eq!(repo.find_by_id(staff.id()).await.unwrap(), None);
    assert!(matches!(
        repo.delete(staff.id()).await.unwrap_err(),
        RepositoryError::NotFound {
            entity: "access_group"
        }
    ));
    db.cleanup().await;
}

#[tokio::test]
async fn group_names_are_unique_and_listed_alphabetically() {
    let db = TestDb::new().await;
    let repo = PgAccessGroupRepository::new(db.pool());
    repo.insert(&group("Staff")).await.unwrap();
    repo.insert(&group("Cleaners")).await.unwrap();

    let err = repo.insert(&group("Staff")).await.unwrap_err();
    assert!(
        matches!(err, RepositoryError::Duplicate { field: "name" }),
        "{err:?}"
    );

    let names: Vec<String> = repo
        .list(PageRequest::default())
        .await
        .unwrap()
        .iter()
        .map(|g| g.name().to_string())
        .collect();
    assert_eq!(names, ["Cleaners", "Staff"]);
    db.cleanup().await;
}

#[tokio::test]
async fn membership_rules() {
    let db = TestDb::new().await;
    let repo = PgAccessGroupRepository::new(db.pool());
    let staff = group("Staff");
    repo.insert(&staff).await.unwrap();
    let alice = saved_user(&db).await;

    repo.add_member(staff.id(), alice.id()).await.unwrap();
    assert_eq!(
        repo.list_members(staff.id()).await.unwrap(),
        vec![alice.id()]
    );

    let duplicate = repo.add_member(staff.id(), alice.id()).await.unwrap_err();
    assert!(
        matches!(
            duplicate,
            RepositoryError::Duplicate {
                field: "membership"
            }
        ),
        "{duplicate:?}"
    );
    let unknown_user = repo
        .add_member(staff.id(), UserId::generate())
        .await
        .unwrap_err();
    assert!(
        matches!(unknown_user, RepositoryError::NotFound { entity: "user" }),
        "{unknown_user:?}"
    );
    let unknown_group = repo
        .add_member(AccessGroupId::generate(), alice.id())
        .await
        .unwrap_err();
    assert!(
        matches!(
            unknown_group,
            RepositoryError::NotFound {
                entity: "access_group"
            }
        ),
        "{unknown_group:?}"
    );

    repo.remove_member(staff.id(), alice.id()).await.unwrap();
    assert!(repo.list_members(staff.id()).await.unwrap().is_empty());
    let not_member = repo
        .remove_member(staff.id(), alice.id())
        .await
        .unwrap_err();
    assert!(matches!(
        not_member,
        RepositoryError::NotFound {
            entity: "membership"
        }
    ));
    db.cleanup().await;
}

// ---- schedules ----

#[tokio::test]
async fn schedule_round_trip_keeps_rules_zone_and_dates() {
    let db = TestDb::new().await;
    let repo = PgScheduleRepository::new(db.pool());
    let hours = schedule("Business hours");

    repo.insert(&hours).await.unwrap();

    let stored = repo.find_by_id(hours.id()).await.unwrap().unwrap();
    assert_eq!(stored, hours);
    assert_eq!(stored.rules().len(), 2);
    assert!(stored.rules()[1].crosses_midnight());
    assert_eq!(repo.find_by_id(ScheduleId::generate()).await.unwrap(), None);
    db.cleanup().await;
}

#[tokio::test]
async fn duplicate_schedule_name_is_a_conflict() {
    let db = TestDb::new().await;
    let repo = PgScheduleRepository::new(db.pool());
    repo.insert(&schedule("Business hours")).await.unwrap();

    let err = repo.insert(&schedule("Business hours")).await.unwrap_err();
    assert!(
        matches!(err, RepositoryError::Duplicate { field: "name" }),
        "{err:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn schedule_insert_is_atomic() {
    let db = TestDb::new().await;
    // Sabotage this throwaway database: the *second* rule insert fails, after
    // the schedule row and first rule were already written.
    sqlx::raw_sql(
        "CREATE FUNCTION fail_second_rule() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN
             IF NEW.position = 1 THEN RAISE EXCEPTION 'simulated failure'; END IF;
             RETURN NEW;
         END $$;
         CREATE TRIGGER fail_second_rule BEFORE INSERT ON access_schedule_rules
             FOR EACH ROW EXECUTE FUNCTION fail_second_rule();",
    )
    .execute(&db.pool())
    .await
    .unwrap();
    let repo = PgScheduleRepository::new(db.pool());

    let hours = schedule("Business hours"); // two rules
    assert!(repo.insert(&hours).await.is_err());

    // The transaction rolled back: neither the schedule nor its first rule exist.
    assert_eq!(repo.find_by_id(hours.id()).await.unwrap(), None);
    let rule_rows: i64 = sqlx::query_scalar("SELECT count(*) FROM access_schedule_rules")
        .fetch_one(&db.pool())
        .await
        .unwrap();
    assert_eq!(rule_rows, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn schedules_list_with_rules() {
    let db = TestDb::new().await;
    let repo = PgScheduleRepository::new(db.pool());
    let (b, a) = (schedule("B hours"), schedule("A hours"));
    repo.insert(&b).await.unwrap();
    repo.insert(&a).await.unwrap();

    assert_eq!(repo.list(PageRequest::default()).await.unwrap(), vec![a, b]);
    db.cleanup().await;
}

// ---- permissions ----

#[tokio::test]
async fn permission_round_trip_and_revoke() {
    let db = TestDb::new().await;
    let staff = group("Staff");
    PgAccessGroupRepository::new(db.pool())
        .insert(&staff)
        .await
        .unwrap();
    let door = saved_door(&db).await;
    let hours = schedule("Hours");
    PgScheduleRepository::new(db.pool())
        .insert(&hours)
        .await
        .unwrap();
    let repo = PgPermissionRepository::new(db.pool());

    let scheduled = AccessPermission::grant(staff.id(), door.id(), Some(hours.id()), now());
    let always = AccessPermission::grant(staff.id(), door.id(), None, now());
    repo.insert(&scheduled).await.unwrap();
    repo.insert(&always).await.unwrap();

    assert_eq!(
        repo.find_by_id(scheduled.id()).await.unwrap(),
        Some(scheduled.clone())
    );
    assert_eq!(repo.list(PageRequest::default()).await.unwrap().len(), 2);

    repo.delete(scheduled.id()).await.unwrap();
    assert_eq!(repo.find_by_id(scheduled.id()).await.unwrap(), None);
    db.cleanup().await;
}

#[tokio::test]
async fn identical_permissions_are_rejected_even_without_schedule() {
    let db = TestDb::new().await;
    let staff = group("Staff");
    PgAccessGroupRepository::new(db.pool())
        .insert(&staff)
        .await
        .unwrap();
    let door = saved_door(&db).await;
    let repo = PgPermissionRepository::new(db.pool());
    repo.insert(&AccessPermission::grant(staff.id(), door.id(), None, now()))
        .await
        .unwrap();

    // NULL schedule twice: only caught thanks to UNIQUE NULLS NOT DISTINCT.
    let err = repo
        .insert(&AccessPermission::grant(staff.id(), door.id(), None, now()))
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            RepositoryError::Duplicate {
                field: "permission"
            }
        ),
        "{err:?}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn permission_references_must_exist() {
    let db = TestDb::new().await;
    let staff = group("Staff");
    PgAccessGroupRepository::new(db.pool())
        .insert(&staff)
        .await
        .unwrap();
    let door = saved_door(&db).await;
    let repo = PgPermissionRepository::new(db.pool());

    let cases = [
        (AccessGroupId::generate(), door.id(), None, "access_group"),
        (staff.id(), DoorId::generate(), None, "door"),
        (
            staff.id(),
            door.id(),
            Some(ScheduleId::generate()),
            "schedule",
        ),
    ];
    for (group_id, door_id, schedule_id, expected) in cases {
        let err = repo
            .insert(&AccessPermission::grant(
                group_id,
                door_id,
                schedule_id,
                now(),
            ))
            .await
            .unwrap_err();
        assert!(
            matches!(err, RepositoryError::NotFound { entity } if entity == expected),
            "expected NotFound({expected}), got {err:?}"
        );
    }
    db.cleanup().await;
}

#[tokio::test]
async fn deleting_a_group_removes_its_permissions() {
    let db = TestDb::new().await;
    let groups = PgAccessGroupRepository::new(db.pool());
    let staff = group("Staff");
    groups.insert(&staff).await.unwrap();
    let door = saved_door(&db).await;
    let permissions = PgPermissionRepository::new(db.pool());
    let permission = AccessPermission::grant(staff.id(), door.id(), None, now());
    permissions.insert(&permission).await.unwrap();

    groups.delete(staff.id()).await.unwrap();

    assert_eq!(permissions.find_by_id(permission.id()).await.unwrap(), None);
    db.cleanup().await;
}
