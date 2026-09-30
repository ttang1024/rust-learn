//! Access group management use cases.

use super::{AccessGroupRepository, ApplicationError, PageRequest};
use crate::domain::{AccessGroup, AccessGroupId, Description, GroupName, UserId};

#[derive(Debug, Clone)]
pub struct CreateGroup {
    pub name: String,
    pub description: Option<String>,
}

/// Partial update.
///
/// `description` is an `Option<Option<_>>` so a PATCH can say three things:
/// `None` = leave unchanged, `Some(None)` = clear it, `Some(Some(text))` = set it.
#[derive(Debug, Clone, Default)]
pub struct UpdateGroup {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

pub struct AccessGroupService<G> {
    groups: G,
}

impl<G: AccessGroupRepository> AccessGroupService<G> {
    pub fn new(groups: G) -> Self {
        Self { groups }
    }

    pub async fn create(&self, cmd: CreateGroup) -> Result<AccessGroup, ApplicationError> {
        let name = GroupName::parse(&cmd.name)?;
        let description = cmd
            .description
            .as_deref()
            .map(Description::parse)
            .transpose()?;
        let group = AccessGroup::create(name, description);
        self.groups.insert(&group).await?;
        Ok(group)
    }

    pub async fn get(&self, id: AccessGroupId) -> Result<AccessGroup, ApplicationError> {
        self.groups
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound {
                entity: "access_group",
            })
    }

    pub async fn list(&self, page: PageRequest) -> Result<Vec<AccessGroup>, ApplicationError> {
        Ok(self.groups.list(page).await?)
    }

    pub async fn update(
        &self,
        id: AccessGroupId,
        cmd: UpdateGroup,
    ) -> Result<AccessGroup, ApplicationError> {
        let name = cmd.name.as_deref().map(GroupName::parse).transpose()?;
        // Option<Option<String>> -> Option<Option<Description>>, validating
        // only when a new text is actually given.
        let description = cmd
            .description
            .map(|text| text.as_deref().map(Description::parse).transpose())
            .transpose()?;

        let mut group = self.get(id).await?;
        if let Some(name) = name {
            group.rename(name);
        }
        if let Some(description) = description {
            group.set_description(description);
        }
        self.groups.update(&group).await?;
        Ok(group)
    }

    /// Deletes the group; its members lose the access it granted.
    pub async fn delete(&self, id: AccessGroupId) -> Result<(), ApplicationError> {
        Ok(self.groups.delete(id).await?)
    }

    pub async fn add_member(
        &self,
        group_id: AccessGroupId,
        user_id: UserId,
    ) -> Result<(), ApplicationError> {
        Ok(self.groups.add_member(group_id, user_id).await?)
    }

    pub async fn remove_member(
        &self,
        group_id: AccessGroupId,
        user_id: UserId,
    ) -> Result<(), ApplicationError> {
        Ok(self.groups.remove_member(group_id, user_id).await?)
    }

    /// Member ids; `NotFound` for an unknown group (not an empty list).
    pub async fn members(&self, group_id: AccessGroupId) -> Result<Vec<UserId>, ApplicationError> {
        self.get(group_id).await?;
        Ok(self.groups.list_members(group_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{application::fakes::InMemoryGroups, domain::DomainError};

    fn service() -> AccessGroupService<InMemoryGroups> {
        AccessGroupService::new(InMemoryGroups::default())
    }

    fn staff() -> CreateGroup {
        CreateGroup {
            name: "Staff".into(),
            description: Some("All employees".into()),
        }
    }

    #[tokio::test]
    async fn create_and_get() {
        let service = service();
        let group = service.create(staff()).await.unwrap();
        assert_eq!(service.get(group.id()).await.unwrap(), group);
        assert_eq!(
            group.description().map(Description::as_str),
            Some("All employees")
        );
    }

    #[tokio::test]
    async fn duplicate_name_is_a_conflict() {
        let service = service();
        service.create(staff()).await.unwrap();
        let err = service.create(staff()).await.unwrap_err();
        assert!(matches!(err, ApplicationError::Conflict { field: "name" }));
    }

    #[tokio::test]
    async fn empty_description_is_rejected() {
        let err = service()
            .create(CreateGroup {
                name: "Staff".into(),
                description: Some("  ".into()),
            })
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::Validation {
                field: "description",
                ..
            })
        ));
    }

    #[tokio::test]
    async fn update_distinguishes_unchanged_from_cleared() {
        let service = service();
        let group = service.create(staff()).await.unwrap();

        // Rename only: description untouched.
        let renamed = service
            .update(
                group.id(),
                UpdateGroup {
                    name: Some("All Staff".into()),
                    description: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(renamed.name().as_str(), "All Staff");
        assert!(renamed.description().is_some());

        // Explicitly clear the description.
        let cleared = service
            .update(
                group.id(),
                UpdateGroup {
                    name: None,
                    description: Some(None),
                },
            )
            .await
            .unwrap();
        assert_eq!(cleared.description(), None);
        assert_eq!(service.get(group.id()).await.unwrap(), cleared);
    }

    #[tokio::test]
    async fn delete_then_get_is_not_found() {
        let service = service();
        let group = service.create(staff()).await.unwrap();
        service.delete(group.id()).await.unwrap();

        let err = service.get(group.id()).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::NotFound {
                entity: "access_group"
            }
        ));
        let err = service.delete(group.id()).await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound { .. }));
    }

    #[tokio::test]
    async fn members_of_unknown_group_is_not_found() {
        let err = service()
            .members(AccessGroupId::generate())
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::NotFound {
                entity: "access_group"
            }
        ));
    }

    #[tokio::test]
    async fn membership_add_and_remove() {
        let service = service();
        let group = service.create(staff()).await.unwrap();
        let user = UserId::generate();

        service.add_member(group.id(), user).await.unwrap();
        let err = service.add_member(group.id(), user).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Conflict {
                field: "membership"
            }
        ));

        assert_eq!(service.members(group.id()).await.unwrap(), vec![user]);

        service.remove_member(group.id(), user).await.unwrap();
        assert!(service.members(group.id()).await.unwrap().is_empty());
        let err = service.remove_member(group.id(), user).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::NotFound {
                entity: "membership"
            }
        ));
    }
}
