//! In-memory test doubles for the application ports (compiled for tests only).
//!
//! They mimic the database's uniqueness rules so use-case tests exercise the
//! same error paths as production.
//!
//! A `std::sync::Mutex` is fine here: each method locks, works and unlocks
//! without an `.await` in between. Holding a std guard *across* an `.await`
//! would make the future `!Send` and could block a Tokio worker thread.

use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
};

use chrono::Duration;

use super::{
    AccessClaims, AccessDataSource, AccessEventRepository, AccessGroupRepository, AccessSnapshot,
    AdministratorRepository, AuditLogRepository, BoxError, CardRepository, Clock, ConsumeOutcome,
    ControllerRepository, DoorRepository, EventFilter, EventPublisher, IssuedAccessToken,
    PageRequest, Password, PasswordHasher, PermissionRepository, RefreshTokenRecord,
    RefreshTokenRepository, RepositoryError, RepositoryResult, RevocationReason,
    ScheduleRepository, SecretGenerator, TokenIssuer, UserRepository,
};
use crate::domain::{
    AccessCard, AccessEvent, AccessGroup, AccessGroupId, AccessPermission, AccessSchedule,
    Administrator, AdministratorId, AuditEntry, CardId, CardNumber, Controller, ControllerId, Door,
    DoorId, Email, EventId, PasswordHash, PermissionId, ScheduleId, Timestamp, User, UserId,
    Username,
};

/// A clock that only moves when told to.
pub struct FixedClock(Mutex<Timestamp>);

impl FixedClock {
    pub fn at(now: Timestamp) -> Arc<Self> {
        Arc::new(Self(Mutex::new(now)))
    }

    pub fn advance(&self, by: Duration) {
        *self.0.lock().unwrap() += by;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        *self.0.lock().unwrap()
    }
}

/// The stored row with this id, like `UPDATE ... WHERE id = $1`:
/// `NotFound` if there is none.
fn row_mut<'a, K: Eq + Hash, V>(
    rows: &'a mut HashMap<K, V>,
    id: &K,
    entity: &'static str,
) -> RepositoryResult<&'a mut V> {
    rows.get_mut(id).ok_or(RepositoryError::NotFound { entity })
}

/// Applies a page to already-sorted items.
fn paginate<T>(items: Vec<T>, page: PageRequest) -> Vec<T> {
    items
        .into_iter()
        .skip(page.offset() as usize)
        .take(page.limit() as usize)
        .collect()
}

/// Cloning shares the same storage, like cloning a `PgPool`.
#[derive(Default, Clone)]
pub struct InMemoryUsers(Arc<Mutex<HashMap<UserId, User>>>);

impl UserRepository for InMemoryUsers {
    async fn insert(&self, user: &User) -> RepositoryResult<()> {
        let mut rows = self.0.lock().unwrap();
        if rows.values().any(|u| u.email() == user.email()) {
            return Err(RepositoryError::Duplicate { field: "email" });
        }
        rows.insert(user.id(), user.clone());
        Ok(())
    }

    async fn update(&self, user: &User) -> RepositoryResult<()> {
        let mut rows = self.0.lock().unwrap();
        if rows
            .values()
            .any(|u| u.id() != user.id() && u.email() == user.email())
        {
            return Err(RepositoryError::Duplicate { field: "email" });
        }
        *row_mut(&mut rows, &user.id(), "user")? = user.clone();
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>> {
        Ok(self.0.lock().unwrap().get(&id).cloned())
    }

    async fn find_by_email(&self, email: &Email) -> RepositoryResult<Option<User>> {
        let rows = self.0.lock().unwrap();
        Ok(rows.values().find(|u| u.email() == email).cloned())
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<User>> {
        let mut users: Vec<User> = self.0.lock().unwrap().values().cloned().collect();
        users.sort_by_key(|u| (u.created_at(), u.id()));
        Ok(paginate(users, page))
    }
}

#[derive(Default, Clone)]
pub struct InMemoryCards(Arc<Mutex<HashMap<CardId, AccessCard>>>);

impl CardRepository for InMemoryCards {
    async fn insert(&self, card: &AccessCard) -> RepositoryResult<()> {
        let mut rows = self.0.lock().unwrap();
        if rows.values().any(|c| c.card_number() == card.card_number()) {
            return Err(RepositoryError::Duplicate {
                field: "card_number",
            });
        }
        rows.insert(card.id(), card.clone());
        Ok(())
    }

    async fn update(&self, card: &AccessCard) -> RepositoryResult<()> {
        *row_mut(&mut self.0.lock().unwrap(), &card.id(), "card")? = card.clone();
        Ok(())
    }

    async fn find_by_id(&self, id: CardId) -> RepositoryResult<Option<AccessCard>> {
        Ok(self.0.lock().unwrap().get(&id).cloned())
    }

    async fn find_by_number(&self, number: &CardNumber) -> RepositoryResult<Option<AccessCard>> {
        let rows = self.0.lock().unwrap();
        Ok(rows.values().find(|c| c.card_number() == number).cloned())
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<AccessCard>> {
        let mut cards: Vec<AccessCard> = self.0.lock().unwrap().values().cloned().collect();
        cards.sort_by_key(|c| (c.issued_at(), c.id()));
        Ok(paginate(cards, page))
    }

    async fn list_for_user(&self, user_id: UserId) -> RepositoryResult<Vec<AccessCard>> {
        let mut cards: Vec<AccessCard> = self
            .0
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.user_id() == user_id)
            .cloned()
            .collect();
        cards.sort_by_key(|c| (c.issued_at(), c.id()));
        Ok(cards)
    }
}

#[derive(Default, Clone)]
pub struct InMemoryDoors(Arc<Mutex<HashMap<DoorId, Door>>>);

impl DoorRepository for InMemoryDoors {
    async fn insert(&self, door: &Door) -> RepositoryResult<()> {
        self.0.lock().unwrap().insert(door.id(), door.clone());
        Ok(())
    }

    async fn update(&self, door: &Door) -> RepositoryResult<()> {
        *row_mut(&mut self.0.lock().unwrap(), &door.id(), "door")? = door.clone();
        Ok(())
    }

    async fn find_by_id(&self, id: DoorId) -> RepositoryResult<Option<Door>> {
        Ok(self.0.lock().unwrap().get(&id).cloned())
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<Door>> {
        let mut doors: Vec<Door> = self.0.lock().unwrap().values().cloned().collect();
        doors.sort_by_key(|d| (d.created_at(), d.id()));
        Ok(paginate(doors, page))
    }

    async fn list_by_controller(
        &self,
        controller_id: &ControllerId,
    ) -> RepositoryResult<Vec<Door>> {
        let rows = self.0.lock().unwrap();
        Ok(rows
            .values()
            .filter(|door| door.controller_id() == controller_id)
            .cloned()
            .collect())
    }
}

#[derive(Default, Clone)]
pub struct InMemoryGroups {
    groups: Arc<Mutex<HashMap<AccessGroupId, AccessGroup>>>,
    members: Arc<Mutex<Vec<(AccessGroupId, UserId)>>>,
}

impl AccessGroupRepository for InMemoryGroups {
    async fn insert(&self, group: &AccessGroup) -> RepositoryResult<()> {
        let mut groups = self.groups.lock().unwrap();
        if groups.values().any(|g| g.name() == group.name()) {
            return Err(RepositoryError::Duplicate { field: "name" });
        }
        groups.insert(group.id(), group.clone());
        Ok(())
    }

    async fn update(&self, group: &AccessGroup) -> RepositoryResult<()> {
        *row_mut(
            &mut self.groups.lock().unwrap(),
            &group.id(),
            "access_group",
        )? = group.clone();
        Ok(())
    }

    async fn delete(&self, id: AccessGroupId) -> RepositoryResult<()> {
        if self.groups.lock().unwrap().remove(&id).is_none() {
            return Err(RepositoryError::NotFound {
                entity: "access_group",
            });
        }
        self.members
            .lock()
            .unwrap()
            .retain(|(group, _)| *group != id);
        Ok(())
    }

    async fn find_by_id(&self, id: AccessGroupId) -> RepositoryResult<Option<AccessGroup>> {
        Ok(self.groups.lock().unwrap().get(&id).cloned())
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<AccessGroup>> {
        let mut groups: Vec<AccessGroup> = self.groups.lock().unwrap().values().cloned().collect();
        groups.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));
        Ok(paginate(groups, page))
    }

    async fn add_member(&self, group_id: AccessGroupId, user_id: UserId) -> RepositoryResult<()> {
        let mut members = self.members.lock().unwrap();
        if members.contains(&(group_id, user_id)) {
            return Err(RepositoryError::Duplicate {
                field: "membership",
            });
        }
        members.push((group_id, user_id));
        Ok(())
    }

    async fn remove_member(
        &self,
        group_id: AccessGroupId,
        user_id: UserId,
    ) -> RepositoryResult<()> {
        let mut members = self.members.lock().unwrap();
        let before = members.len();
        members.retain(|m| *m != (group_id, user_id));
        if members.len() == before {
            return Err(RepositoryError::NotFound {
                entity: "membership",
            });
        }
        Ok(())
    }

    async fn list_members(&self, group_id: AccessGroupId) -> RepositoryResult<Vec<UserId>> {
        let members = self.members.lock().unwrap();
        Ok(members
            .iter()
            .filter(|(group, _)| *group == group_id)
            .map(|(_, user)| *user)
            .collect())
    }
}

#[derive(Default, Clone)]
pub struct InMemorySchedules(Arc<Mutex<HashMap<ScheduleId, AccessSchedule>>>);

impl ScheduleRepository for InMemorySchedules {
    async fn insert(&self, schedule: &AccessSchedule) -> RepositoryResult<()> {
        let mut rows = self.0.lock().unwrap();
        if rows.values().any(|s| s.name() == schedule.name()) {
            return Err(RepositoryError::Duplicate { field: "name" });
        }
        rows.insert(schedule.id(), schedule.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: ScheduleId) -> RepositoryResult<Option<AccessSchedule>> {
        Ok(self.0.lock().unwrap().get(&id).cloned())
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<AccessSchedule>> {
        let mut rows: Vec<AccessSchedule> = self.0.lock().unwrap().values().cloned().collect();
        rows.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));
        Ok(paginate(rows, page))
    }
}

/// Mimics the unique (group, door, schedule) constraint. Foreign keys are
/// not simulated; those are covered by the PostgreSQL tests.
#[derive(Default, Clone)]
pub struct InMemoryPermissions(Arc<Mutex<HashMap<PermissionId, AccessPermission>>>);

impl PermissionRepository for InMemoryPermissions {
    async fn insert(&self, permission: &AccessPermission) -> RepositoryResult<()> {
        let mut rows = self.0.lock().unwrap();
        let key = |p: &AccessPermission| (p.group_id(), p.door_id(), p.schedule_id());
        if rows.values().any(|p| key(p) == key(permission)) {
            return Err(RepositoryError::Duplicate {
                field: "permission",
            });
        }
        rows.insert(permission.id(), permission.clone());
        Ok(())
    }

    async fn delete(&self, id: PermissionId) -> RepositoryResult<()> {
        match self.0.lock().unwrap().remove(&id) {
            Some(_) => Ok(()),
            None => Err(RepositoryError::NotFound {
                entity: "permission",
            }),
        }
    }

    async fn find_by_id(&self, id: PermissionId) -> RepositoryResult<Option<AccessPermission>> {
        Ok(self.0.lock().unwrap().get(&id).cloned())
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<AccessPermission>> {
        let mut rows: Vec<AccessPermission> = self.0.lock().unwrap().values().cloned().collect();
        rows.sort_by_key(|p| (p.created_at(), p.id()));
        Ok(paginate(rows, page))
    }
}

#[derive(Default, Clone)]
pub struct InMemoryEvents(Arc<Mutex<Vec<AccessEvent>>>);

impl AccessEventRepository for InMemoryEvents {
    async fn append(&self, event: &AccessEvent) -> RepositoryResult<()> {
        self.0.lock().unwrap().push(event.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: EventId) -> RepositoryResult<Option<AccessEvent>> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id() == id)
            .cloned())
    }

    async fn list(
        &self,
        filter: &EventFilter,
        page: PageRequest,
    ) -> RepositoryResult<Vec<AccessEvent>> {
        let mut events: Vec<AccessEvent> = self
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|event| filter.matches(event))
            .cloned()
            .collect();
        events.sort_by_key(|e| std::cmp::Reverse((e.occurred_at(), e.id())));
        Ok(paginate(events, page))
    }
}

/// An event store whose writes always fail, as if the database were down.
pub struct FailingEvents;

impl AccessEventRepository for FailingEvents {
    async fn append(&self, _event: &AccessEvent) -> RepositoryResult<()> {
        Err(RepositoryError::Storage("database unavailable".into()))
    }

    async fn find_by_id(&self, _id: EventId) -> RepositoryResult<Option<AccessEvent>> {
        Err(RepositoryError::Storage("database unavailable".into()))
    }

    async fn list(
        &self,
        _filter: &EventFilter,
        _page: PageRequest,
    ) -> RepositoryResult<Vec<AccessEvent>> {
        Err(RepositoryError::Storage("database unavailable".into()))
    }
}

/// Returns a fixed snapshot and remembers which card number it was asked for.
#[derive(Clone)]
pub struct StaticAccessData {
    snapshot: AccessSnapshot,
    last_card_number: Arc<Mutex<Option<Option<CardNumber>>>>,
}

impl StaticAccessData {
    pub fn new(snapshot: AccessSnapshot) -> Self {
        Self {
            snapshot,
            last_card_number: Arc::default(),
        }
    }

    /// `None` = never called; `Some(None)` = called without a card number.
    pub fn last_card_number(&self) -> Option<Option<CardNumber>> {
        self.last_card_number.lock().unwrap().clone()
    }
}

impl AccessDataSource for StaticAccessData {
    async fn load(
        &self,
        card_number: Option<&CardNumber>,
        _door_id: DoorId,
    ) -> RepositoryResult<AccessSnapshot> {
        *self.last_card_number.lock().unwrap() = Some(card_number.cloned());
        Ok(self.snapshot.clone())
    }
}

#[derive(Default, Clone)]
pub struct InMemoryAdmins(Arc<Mutex<HashMap<AdministratorId, Administrator>>>);

impl InMemoryAdmins {
    /// Changes a stored administrator in place, e.g. `Administrator::disable`.
    pub fn update_with(&self, id: AdministratorId, change: impl FnOnce(&mut Administrator)) {
        change(
            self.0
                .lock()
                .unwrap()
                .get_mut(&id)
                .expect("administrator exists"),
        );
    }
}

impl AdministratorRepository for InMemoryAdmins {
    async fn insert(&self, admin: &Administrator) -> RepositoryResult<()> {
        let mut rows = self.0.lock().unwrap();
        if rows.values().any(|a| a.username() == admin.username()) {
            return Err(RepositoryError::Duplicate { field: "username" });
        }
        rows.insert(admin.id(), admin.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: AdministratorId) -> RepositoryResult<Option<Administrator>> {
        Ok(self.0.lock().unwrap().get(&id).cloned())
    }

    async fn find_by_username(
        &self,
        username: &Username,
    ) -> RepositoryResult<Option<Administrator>> {
        let rows = self.0.lock().unwrap();
        Ok(rows.values().find(|a| a.username() == username).cloned())
    }
}

/// A stored token and, once it is no longer usable, why.
type TokenRow = (RefreshTokenRecord, Option<RevocationReason>);

/// Same semantics as the PostgreSQL implementation, including which
/// revocation reason turns into which outcome.
#[derive(Default, Clone)]
pub struct InMemoryRefreshTokens(Arc<Mutex<HashMap<String, TokenRow>>>);

impl RefreshTokenRepository for InMemoryRefreshTokens {
    async fn insert(&self, record: &RefreshTokenRecord) -> RepositoryResult<()> {
        self.0
            .lock()
            .unwrap()
            .insert(record.token_hash.clone(), (record.clone(), None));
        Ok(())
    }

    async fn consume(
        &self,
        token_hash: &str,
        reason: RevocationReason,
        _now: Timestamp,
    ) -> RepositoryResult<ConsumeOutcome> {
        let mut rows = self.0.lock().unwrap();
        Ok(match rows.get_mut(token_hash) {
            None => ConsumeOutcome::Unknown,
            Some((record, revoked @ None)) => {
                *revoked = Some(reason);
                ConsumeOutcome::Consumed(record.clone())
            }
            Some((record, Some(RevocationReason::Rotated))) => ConsumeOutcome::Reused {
                admin_id: record.admin_id,
            },
            Some((_, Some(_))) => ConsumeOutcome::Revoked,
        })
    }

    async fn revoke_all(&self, admin_id: AdministratorId, _now: Timestamp) -> RepositoryResult<()> {
        for (record, revoked) in self.0.lock().unwrap().values_mut() {
            if record.admin_id == admin_id && revoked.is_none() {
                *revoked = Some(RevocationReason::Revoked);
            }
        }
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct InMemoryAudit(Arc<Mutex<Vec<AuditEntry>>>);

impl AuditLogRepository for InMemoryAudit {
    async fn append(&self, entry: &AuditEntry) -> RepositoryResult<()> {
        self.0.lock().unwrap().push(entry.clone());
        Ok(())
    }

    /// Newest first. Insertion order breaks ties between equal timestamps.
    async fn list_recent(&self, page: PageRequest) -> RepositoryResult<Vec<AuditEntry>> {
        let mut entries = self.0.lock().unwrap().clone();
        entries.reverse();
        Ok(paginate(entries, page))
    }
}

/// "Hashes" by prefixing, and records how `verify` was called, so tests can
/// check the dummy-verification path without the cost of Argon2.
#[derive(Default, Clone)]
pub struct FakeHasher {
    calls: Arc<Mutex<Vec<bool>>>,
}

impl FakeHasher {
    pub fn verify_calls(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Whether the last `verify` received a real hash (`None` = never called).
    pub fn last_verify_had_hash(&self) -> Option<bool> {
        self.calls.lock().unwrap().last().copied()
    }
}

impl PasswordHasher for FakeHasher {
    async fn hash(&self, password: &Password) -> Result<PasswordHash, BoxError> {
        Ok(PasswordHash::new(format!("fake:{}", password.expose())))
    }

    async fn verify(&self, password: &Password, hash: Option<&PasswordHash>) -> bool {
        self.calls.lock().unwrap().push(hash.is_some());
        hash.is_some_and(|hash| hash.as_str() == format!("fake:{}", password.expose()))
    }
}

/// Readable, unsigned tokens: `access:<admin id>:<role>:<expiry unix secs>`.
/// Only for exercising the service logic; never a model for real tokens.
#[derive(Default)]
pub struct FakeTokens {
    counter: Mutex<u64>,
}

impl TokenIssuer for FakeTokens {
    fn issue_access_token(
        &self,
        admin: &Administrator,
        now: Timestamp,
    ) -> Result<IssuedAccessToken, BoxError> {
        let expires_at = now + Duration::minutes(15);
        Ok(IssuedAccessToken {
            token: format!(
                "access:{}:{}:{}",
                admin.id(),
                admin.role().as_str(),
                expires_at.timestamp()
            ),
            expires_at,
        })
    }

    fn verify_access_token(&self, token: &str, now: Timestamp) -> Option<AccessClaims> {
        let mut parts = token.strip_prefix("access:")?.split(':');
        let admin_id = AdministratorId::from_uuid(parts.next()?.parse().ok()?);
        let role = parts.next()?.parse().ok()?;
        let expires_at = Timestamp::from_timestamp(parts.next()?.parse().ok()?, 0)?;
        (now < expires_at).then_some(AccessClaims {
            admin_id,
            role,
            expires_at,
        })
    }

    fn generate_refresh_token(&self) -> Result<String, BoxError> {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        Ok(format!("refresh-{counter}"))
    }

    fn hash_refresh_token(&self, token: &str) -> String {
        format!("hashed:{token}")
    }
}

/// Remembers every published event.
#[derive(Default, Clone)]
pub struct RecordingPublisher(Arc<Mutex<Vec<AccessEvent>>>);

impl RecordingPublisher {
    pub fn events(&self) -> Vec<AccessEvent> {
        self.0.lock().unwrap().clone()
    }
}

impl EventPublisher for RecordingPublisher {
    fn publish(&self, event: &AccessEvent) {
        self.0.lock().unwrap().push(event.clone());
    }
}

#[derive(Default, Clone)]
pub struct InMemoryControllers(Arc<Mutex<HashMap<ControllerId, (Controller, String)>>>);

impl ControllerRepository for InMemoryControllers {
    async fn insert(&self, controller: &Controller, key_hash: &str) -> RepositoryResult<()> {
        let mut rows = self.0.lock().unwrap();
        if rows.contains_key(controller.id()) {
            return Err(RepositoryError::Duplicate {
                field: "controller_id",
            });
        }
        rows.insert(
            controller.id().clone(),
            (controller.clone(), key_hash.to_owned()),
        );
        Ok(())
    }

    async fn update(&self, controller: &Controller) -> RepositoryResult<()> {
        row_mut(&mut self.0.lock().unwrap(), controller.id(), "controller")?.0 = controller.clone();
        Ok(())
    }

    async fn set_key_hash(&self, id: &ControllerId, key_hash: &str) -> RepositoryResult<()> {
        row_mut(&mut self.0.lock().unwrap(), id, "controller")?.1 = key_hash.to_owned();
        Ok(())
    }

    async fn find_by_id(&self, id: &ControllerId) -> RepositoryResult<Option<Controller>> {
        Ok(self.0.lock().unwrap().get(id).map(|(c, _)| c.clone()))
    }

    async fn find_by_key_hash(&self, key_hash: &str) -> RepositoryResult<Option<Controller>> {
        let rows = self.0.lock().unwrap();
        Ok(rows
            .values()
            .find(|(_, hash)| hash == key_hash)
            .map(|(c, _)| c.clone()))
    }

    async fn list(&self, page: PageRequest) -> RepositoryResult<Vec<Controller>> {
        let mut rows: Vec<Controller> = self
            .0
            .lock()
            .unwrap()
            .values()
            .map(|(c, _)| c.clone())
            .collect();
        rows.sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
        Ok(paginate(rows, page))
    }

    async fn expire_stale(&self, cutoff: Timestamp) -> RepositoryResult<Vec<ControllerId>> {
        let mut expired = Vec::new();
        for (controller, _) in self.0.lock().unwrap().values_mut() {
            if controller.is_online() && controller.last_seen_at().is_none_or(|seen| seen <= cutoff)
            {
                controller.mark_offline();
                expired.push(controller.id().clone());
            }
        }
        Ok(expired)
    }
}

/// Sequential "secrets" with a readable hash, for deterministic tests.
#[derive(Default)]
pub struct FakeSecrets(Mutex<u64>);

impl SecretGenerator for FakeSecrets {
    fn generate(&self) -> Result<String, BoxError> {
        let mut counter = self.0.lock().unwrap();
        *counter += 1;
        Ok(format!("secret-{counter}"))
    }

    fn hash(&self, secret: &str) -> String {
        format!("hashed:{secret}")
    }
}
