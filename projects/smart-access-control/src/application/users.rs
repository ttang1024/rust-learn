//! User management use cases.

use std::sync::Arc;

use super::{ApplicationError, Clock, PageRequest, UserRepository};
use crate::domain::{DomainError, Email, Timestamp, User, UserId, UserName};

/// Raw input for registering a user. Validation happens in the use case, so
/// every entry point (HTTP, CLI, simulator) gets the same rules.
#[derive(Debug, Clone)]
pub struct RegisterUser {
    pub name: String,
    pub email: String,
}

/// Partial update: `None` leaves a field unchanged (PATCH semantics).
#[derive(Debug, Clone, Default)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub struct UserService<R> {
    users: R,
    clock: Arc<dyn Clock>,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(users: R, clock: Arc<dyn Clock>) -> Self {
        Self { users, clock }
    }

    pub async fn register(&self, cmd: RegisterUser) -> Result<User, ApplicationError> {
        let name = UserName::parse(&cmd.name)?;
        let email = Email::parse(&cmd.email)?;

        // Friendly early check. The database's unique constraint still
        // decides if two registrations race; that also maps to `Conflict`.
        if self.users.find_by_email(&email).await?.is_some() {
            return Err(ApplicationError::Conflict { field: "email" });
        }

        let user = User::register(name, email, self.clock.now());
        self.users.insert(&user).await?;
        Ok(user)
    }

    pub async fn get(&self, id: UserId) -> Result<User, ApplicationError> {
        self.users
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "user" })
    }

    pub async fn list(&self, page: PageRequest) -> Result<Vec<User>, ApplicationError> {
        Ok(self.users.list(page).await?)
    }

    pub async fn update(&self, id: UserId, cmd: UpdateUser) -> Result<User, ApplicationError> {
        // Validate all input before touching storage. `Option<&str>` ->
        // `Option<Result<_>>` -> `Result<Option<_>>` via `transpose`.
        let name = cmd.name.as_deref().map(UserName::parse).transpose()?;
        let email = cmd.email.as_deref().map(Email::parse).transpose()?;

        let mut user = self.get(id).await?;
        let now = self.clock.now();

        if let Some(email) = email
            && &email != user.email()
        {
            if self.users.find_by_email(&email).await?.is_some() {
                return Err(ApplicationError::Conflict { field: "email" });
            }
            user.change_email(email, now)?;
        }
        if let Some(name) = name {
            user.rename(name, now)?;
        }

        self.users.update(&user).await?;
        Ok(user)
    }

    pub async fn suspend(&self, id: UserId) -> Result<User, ApplicationError> {
        self.change_status(id, User::suspend).await
    }

    pub async fn reactivate(&self, id: UserId) -> Result<User, ApplicationError> {
        self.change_status(id, User::reactivate).await
    }

    /// Archives (soft-deletes) a user. Their history stays intact.
    pub async fn archive(&self, id: UserId) -> Result<User, ApplicationError> {
        self.change_status(id, User::archive).await
    }

    /// Load -> apply a domain method -> save.
    ///
    /// `apply` is any function taking `&mut User` and the time, so method
    /// paths like `User::suspend` can be passed directly. `FnOnce` is the
    /// loosest bound: we call it exactly once.
    async fn change_status(
        &self,
        id: UserId,
        apply: impl FnOnce(&mut User, Timestamp) -> Result<(), DomainError>,
    ) -> Result<User, ApplicationError> {
        let mut user = self.get(id).await?;
        apply(&mut user, self.clock.now())?;
        self.users.update(&user).await?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::*;
    use crate::{
        application::fakes::{FixedClock, InMemoryUsers},
        domain::UserStatus,
    };

    fn start() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap()
    }

    fn service() -> (UserService<InMemoryUsers>, Arc<FixedClock>) {
        let clock = FixedClock::at(start());
        (
            UserService::new(InMemoryUsers::default(), clock.clone()),
            clock,
        )
    }

    fn alice() -> RegisterUser {
        RegisterUser {
            name: "Alice".into(),
            email: "alice@example.com".into(),
        }
    }

    #[tokio::test]
    async fn register_stores_active_user() {
        let (service, _) = service();

        let user = service.register(alice()).await.unwrap();

        assert_eq!(user.status(), UserStatus::Active);
        assert_eq!(user.created_at(), start());
        assert_eq!(service.get(user.id()).await.unwrap(), user);
    }

    #[tokio::test]
    async fn register_rejects_invalid_input() {
        let (service, _) = service();

        let err = service
            .register(RegisterUser {
                name: "Alice".into(),
                email: "not-an-email".into(),
            })
            .await
            .unwrap_err();

        assert!(
            matches!(
                err,
                ApplicationError::Domain(DomainError::Validation { field: "email", .. })
            ),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn register_rejects_duplicate_email_case_insensitively() {
        let (service, _) = service();
        service.register(alice()).await.unwrap();

        let err = service
            .register(RegisterUser {
                name: "Other".into(),
                email: "ALICE@Example.com".into(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, ApplicationError::Conflict { field: "email" }));
    }

    #[tokio::test]
    async fn get_unknown_user_is_not_found() {
        let (service, _) = service();
        let err = service.get(UserId::generate()).await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound { entity: "user" }));
    }

    #[tokio::test]
    async fn update_changes_given_fields_only() {
        let (service, clock) = service();
        let user = service.register(alice()).await.unwrap();
        clock.advance(Duration::hours(1));

        let updated = service
            .update(
                user.id(),
                UpdateUser {
                    name: Some("Alice Smith".into()),
                    email: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.name().as_str(), "Alice Smith");
        assert_eq!(updated.email(), user.email());
        assert_eq!(updated.updated_at(), start() + Duration::hours(1));
        assert_eq!(service.get(user.id()).await.unwrap(), updated);
    }

    #[tokio::test]
    async fn update_to_same_email_is_allowed() {
        let (service, _) = service();
        let user = service.register(alice()).await.unwrap();

        let result = service
            .update(
                user.id(),
                UpdateUser {
                    name: None,
                    email: Some("Alice@Example.com".into()),
                },
            )
            .await;

        assert!(result.is_ok(), "{result:?}");
    }

    #[tokio::test]
    async fn update_rejects_email_owned_by_another_user() {
        let (service, _) = service();
        service.register(alice()).await.unwrap();
        let bob = service
            .register(RegisterUser {
                name: "Bob".into(),
                email: "bob@example.com".into(),
            })
            .await
            .unwrap();

        let err = service
            .update(
                bob.id(),
                UpdateUser {
                    name: None,
                    email: Some("alice@example.com".into()),
                },
            )
            .await
            .unwrap_err();

        assert!(matches!(err, ApplicationError::Conflict { field: "email" }));
        assert_eq!(service.get(bob.id()).await.unwrap(), bob);
    }

    #[tokio::test]
    async fn invalid_update_changes_nothing() {
        let (service, _) = service();
        let user = service.register(alice()).await.unwrap();

        // The valid name must not be applied when the email is invalid.
        let result = service
            .update(
                user.id(),
                UpdateUser {
                    name: Some("New Name".into()),
                    email: Some("broken".into()),
                },
            )
            .await;

        assert!(result.is_err());
        assert_eq!(service.get(user.id()).await.unwrap(), user);
    }

    #[tokio::test]
    async fn status_changes_use_the_clock() {
        let (service, clock) = service();
        let user = service.register(alice()).await.unwrap();

        clock.advance(Duration::minutes(5));
        let suspended = service.suspend(user.id()).await.unwrap();
        assert_eq!(suspended.status(), UserStatus::Suspended);
        assert_eq!(suspended.updated_at(), start() + Duration::minutes(5));

        let active = service.reactivate(user.id()).await.unwrap();
        assert!(active.is_active());
    }

    #[tokio::test]
    async fn archived_user_cannot_be_changed() {
        let (service, _) = service();
        let user = service.register(alice()).await.unwrap();
        service.archive(user.id()).await.unwrap();

        let err = service.suspend(user.id()).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::InvalidTransition { .. })
        ));
        let err = service
            .update(
                user.id(),
                UpdateUser {
                    name: Some("Ghost".into()),
                    email: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[tokio::test]
    async fn list_returns_users_in_registration_order() {
        let (service, clock) = service();
        for (name, email) in [("A", "a@example.com"), ("B", "b@example.com")] {
            service
                .register(RegisterUser {
                    name: name.into(),
                    email: email.into(),
                })
                .await
                .unwrap();
            clock.advance(Duration::seconds(1));
        }

        let names: Vec<String> = service
            .list(PageRequest::default())
            .await
            .unwrap()
            .iter()
            .map(|u| u.name().to_string())
            .collect();
        assert_eq!(names, ["A", "B"]);
    }
}
