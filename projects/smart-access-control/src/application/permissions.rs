//! Permission management use cases.

use std::sync::Arc;

use super::{ApplicationError, Clock, PageRequest, PermissionRepository};
use crate::domain::{AccessGroupId, AccessPermission, DoorId, PermissionId, ScheduleId};

#[derive(Debug, Clone, Copy)]
pub struct GrantPermission {
    pub group_id: AccessGroupId,
    pub door_id: DoorId,
    /// `None` grants access at any time.
    pub schedule_id: Option<ScheduleId>,
}

pub struct PermissionService<P> {
    permissions: P,
    clock: Arc<dyn Clock>,
}

impl<P: PermissionRepository> PermissionService<P> {
    pub fn new(permissions: P, clock: Arc<dyn Clock>) -> Self {
        Self { permissions, clock }
    }

    /// Grants a group access to a door. The repository reports a missing
    /// group, door or schedule as `NotFound` (enforced by foreign keys, so
    /// there is no check-then-insert race).
    pub async fn grant(&self, cmd: GrantPermission) -> Result<AccessPermission, ApplicationError> {
        let permission =
            AccessPermission::grant(cmd.group_id, cmd.door_id, cmd.schedule_id, self.clock.now());
        self.permissions.insert(&permission).await?;
        Ok(permission)
    }

    /// Removes a permission; access through it ends immediately.
    pub async fn revoke(&self, id: PermissionId) -> Result<(), ApplicationError> {
        Ok(self.permissions.delete(id).await?)
    }

    pub async fn get(&self, id: PermissionId) -> Result<AccessPermission, ApplicationError> {
        self.permissions
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound {
                entity: "permission",
            })
    }

    pub async fn list(&self, page: PageRequest) -> Result<Vec<AccessPermission>, ApplicationError> {
        Ok(self.permissions.list(page).await?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::application::fakes::{FixedClock, InMemoryPermissions};

    fn service() -> PermissionService<InMemoryPermissions> {
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap());
        PermissionService::new(InMemoryPermissions::default(), clock)
    }

    fn grant() -> GrantPermission {
        GrantPermission {
            group_id: AccessGroupId::generate(),
            door_id: DoorId::generate(),
            schedule_id: None,
        }
    }

    #[tokio::test]
    async fn grant_get_and_revoke() {
        let service = service();
        let permission = service.grant(grant()).await.unwrap();
        assert_eq!(service.get(permission.id()).await.unwrap(), permission);

        service.revoke(permission.id()).await.unwrap();
        let err = service.get(permission.id()).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::NotFound {
                entity: "permission"
            }
        ));
    }

    #[tokio::test]
    async fn identical_permission_is_a_conflict() {
        let service = service();
        let cmd = grant();
        service.grant(cmd).await.unwrap();

        let err = service.grant(cmd).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Conflict {
                field: "permission"
            }
        ));

        // A different schedule for the same group and door is fine.
        let scheduled = GrantPermission {
            schedule_id: Some(ScheduleId::generate()),
            ..cmd
        };
        assert!(service.grant(scheduled).await.is_ok());
    }

    #[tokio::test]
    async fn revoking_unknown_permission_is_not_found() {
        let err = service()
            .revoke(PermissionId::generate())
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound { .. }));
    }
}
