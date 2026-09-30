//! Door management use cases (administrator actions).
//!
//! Also accepts controller status reports (online/offline). Doors start
//! offline, and the decision engine denies access through offline doors.

use std::sync::Arc;

use super::{ApplicationError, Clock, DoorRepository, PageRequest};
use crate::domain::{ControllerId, DomainError, Door, DoorId, DoorName, Location};

#[derive(Debug, Clone)]
pub struct CreateDoor {
    pub name: String,
    pub location: String,
    pub controller_id: String,
}

/// Partial update: `None` leaves a field unchanged.
#[derive(Debug, Clone, Default)]
pub struct UpdateDoor {
    pub name: Option<String>,
    pub location: Option<String>,
    pub controller_id: Option<String>,
}

pub struct DoorService<D> {
    doors: D,
    clock: Arc<dyn Clock>,
}

impl<D: DoorRepository> DoorService<D> {
    pub fn new(doors: D, clock: Arc<dyn Clock>) -> Self {
        Self { doors, clock }
    }

    pub async fn create(&self, cmd: CreateDoor) -> Result<Door, ApplicationError> {
        let door = Door::create(
            DoorName::parse(&cmd.name)?,
            Location::parse(&cmd.location)?,
            ControllerId::parse(&cmd.controller_id)?,
            self.clock.now(),
        );
        self.doors.insert(&door).await?;
        Ok(door)
    }

    pub async fn get(&self, id: DoorId) -> Result<Door, ApplicationError> {
        self.doors
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "door" })
    }

    pub async fn list(&self, page: PageRequest) -> Result<Vec<Door>, ApplicationError> {
        Ok(self.doors.list(page).await?)
    }

    pub async fn update(&self, id: DoorId, cmd: UpdateDoor) -> Result<Door, ApplicationError> {
        let name = cmd.name.as_deref().map(DoorName::parse).transpose()?;
        let location = cmd.location.as_deref().map(Location::parse).transpose()?;
        let controller_id = cmd
            .controller_id
            .as_deref()
            .map(ControllerId::parse)
            .transpose()?;

        let mut door = self.get(id).await?;
        if let Some(name) = name {
            door.rename(name);
        }
        if let Some(location) = location {
            door.relocate(location);
        }
        if let Some(controller_id) = controller_id {
            door.assign_controller(controller_id);
        }
        self.doors.update(&door).await?;
        Ok(door)
    }

    /// Takes a door out of service; all access through it will be denied.
    pub async fn disable(&self, id: DoorId) -> Result<Door, ApplicationError> {
        self.modify(id, Door::disable).await
    }

    /// Returns a door to service. It stays offline until its controller reports.
    pub async fn enable(&self, id: DoorId) -> Result<Door, ApplicationError> {
        self.modify(id, Door::enable).await
    }

    /// A controller reports that it is connected.
    pub async fn report_online(&self, id: DoorId) -> Result<Door, ApplicationError> {
        self.modify(id, Door::mark_online).await
    }

    /// A controller reports (or is detected) as disconnected.
    pub async fn report_offline(&self, id: DoorId) -> Result<Door, ApplicationError> {
        self.modify(id, Door::mark_offline).await
    }

    /// Load -> apply a domain method -> save.
    async fn modify(
        &self,
        id: DoorId,
        apply: impl FnOnce(&mut Door) -> Result<(), DomainError>,
    ) -> Result<Door, ApplicationError> {
        let mut door = self.get(id).await?;
        apply(&mut door)?;
        self.doors.update(&door).await?;
        Ok(door)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::{
        application::fakes::{FixedClock, InMemoryDoors},
        domain::{DomainError, DoorStatus},
    };

    fn service() -> DoorService<InMemoryDoors> {
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap());
        DoorService::new(InMemoryDoors::default(), clock)
    }

    fn main_entrance() -> CreateDoor {
        CreateDoor {
            name: "Main Entrance".into(),
            location: "Building A, Level 1".into(),
            controller_id: "ctrl-001".into(),
        }
    }

    #[tokio::test]
    async fn create_stores_offline_door() {
        let service = service();

        let door = service.create(main_entrance()).await.unwrap();

        assert_eq!(door.status(), DoorStatus::Offline);
        assert_eq!(service.get(door.id()).await.unwrap(), door);
    }

    #[tokio::test]
    async fn create_rejects_invalid_controller_id() {
        let service = service();
        let mut cmd = main_entrance();
        cmd.controller_id = "ctrl 001".into();

        let err = service.create(cmd).await.unwrap_err();

        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::Validation {
                field: "controller_id",
                ..
            })
        ));
        assert!(
            service
                .list(PageRequest::default())
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn disable_and_enable_are_persisted() {
        let service = service();
        let door = service.create(main_entrance()).await.unwrap();

        service.disable(door.id()).await.unwrap();
        assert_eq!(
            service.get(door.id()).await.unwrap().status(),
            DoorStatus::Disabled
        );

        service.enable(door.id()).await.unwrap();
        assert_eq!(
            service.get(door.id()).await.unwrap().status(),
            DoorStatus::Offline
        );
    }

    #[tokio::test]
    async fn enable_of_active_door_is_rejected() {
        let service = service();
        let door = service.create(main_entrance()).await.unwrap();

        let err = service.enable(door.id()).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::InvalidTransition { .. })
        ));
    }

    #[tokio::test]
    async fn controller_reports_change_online_state() {
        let service = service();
        let door = service.create(main_entrance()).await.unwrap();

        assert!(service.report_online(door.id()).await.unwrap().is_online());
        assert!(!service.report_offline(door.id()).await.unwrap().is_online());

        // Reports for a disabled door are rejected, and it stays disabled.
        service.disable(door.id()).await.unwrap();
        assert!(service.report_online(door.id()).await.is_err());
        assert_eq!(
            service.get(door.id()).await.unwrap().status(),
            DoorStatus::Disabled
        );
    }

    #[tokio::test]
    async fn update_changes_only_given_fields() {
        let service = service();
        let door = service.create(main_entrance()).await.unwrap();

        let updated = service
            .update(
                door.id(),
                UpdateDoor {
                    name: Some("Front Door".into()),
                    ..UpdateDoor::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.name().as_str(), "Front Door");
        assert_eq!(updated.location(), door.location());
        assert_eq!(service.get(door.id()).await.unwrap(), updated);

        let err = service
            .update(
                door.id(),
                UpdateDoor {
                    controller_id: Some("bad id!".into()),
                    ..UpdateDoor::default()
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[tokio::test]
    async fn unknown_door_is_not_found() {
        let err = service().disable(DoorId::generate()).await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound { entity: "door" }));
    }
}
