//! Access card use cases.

use std::sync::Arc;

use super::{ApplicationError, CardRepository, Clock, PageRequest, UserRepository};
use crate::domain::{AccessCard, CardId, CardNumber, DomainError, Timestamp, UserId};

#[derive(Debug, Clone)]
pub struct IssueCard {
    pub user_id: UserId,
    pub card_number: String,
    /// `None` issues a card that does not expire.
    pub expires_at: Option<Timestamp>,
}

pub struct CardService<C, U> {
    cards: C,
    users: U,
    clock: Arc<dyn Clock>,
}

impl<C: CardRepository, U: UserRepository> CardService<C, U> {
    pub fn new(cards: C, users: U, clock: Arc<dyn Clock>) -> Self {
        Self {
            cards,
            users,
            clock,
        }
    }

    /// Issues a new card to an existing, active user.
    pub async fn issue(&self, cmd: IssueCard) -> Result<AccessCard, ApplicationError> {
        let number = CardNumber::parse(&cmd.card_number)?;
        let user = self
            .users
            .find_by_id(cmd.user_id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "user" })?;

        if self.cards.find_by_number(&number).await?.is_some() {
            return Err(ApplicationError::Conflict {
                field: "card_number",
            });
        }

        // The domain decides whether this user may hold a card.
        let card = AccessCard::issue(&user, number, self.clock.now(), cmd.expires_at)?;
        self.cards.insert(&card).await?;
        Ok(card)
    }

    pub async fn get(&self, id: CardId) -> Result<AccessCard, ApplicationError> {
        self.cards
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "card" })
    }

    pub async fn list(&self, page: PageRequest) -> Result<Vec<AccessCard>, ApplicationError> {
        Ok(self.cards.list(page).await?)
    }

    /// Cards held by one user. An unknown user is `NotFound`, not an empty
    /// list, so callers can tell "no cards" from "no such user".
    pub async fn list_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<AccessCard>, ApplicationError> {
        if self.users.find_by_id(user_id).await?.is_none() {
            return Err(ApplicationError::NotFound { entity: "user" });
        }
        Ok(self.cards.list_for_user(user_id).await?)
    }

    pub async fn suspend(&self, id: CardId) -> Result<AccessCard, ApplicationError> {
        self.modify(id, AccessCard::suspend).await
    }

    pub async fn reactivate(&self, id: CardId) -> Result<AccessCard, ApplicationError> {
        let now = self.clock.now();
        self.modify(id, |card| card.reactivate(now)).await
    }

    /// Permanently revokes a card (e.g. reported lost).
    pub async fn revoke(&self, id: CardId) -> Result<AccessCard, ApplicationError> {
        self.modify(id, AccessCard::revoke).await
    }

    /// Load -> apply a domain method -> save.
    async fn modify(
        &self,
        id: CardId,
        apply: impl FnOnce(&mut AccessCard) -> Result<(), DomainError>,
    ) -> Result<AccessCard, ApplicationError> {
        let mut card = self.get(id).await?;
        apply(&mut card)?;
        self.cards.update(&card).await?;
        Ok(card)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::*;
    use crate::{
        application::{
            RegisterUser, UserService,
            fakes::{FixedClock, InMemoryCards, InMemoryUsers},
        },
        domain::{CardStatus, User},
    };

    fn start() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap()
    }

    struct Fixture {
        cards: CardService<InMemoryCards, InMemoryUsers>,
        users: UserService<InMemoryUsers>,
        clock: Arc<FixedClock>,
        alice: User,
    }

    /// Both services share one user store (clones share data), the same way
    /// production services share one `PgPool`.
    async fn fixture() -> Fixture {
        let clock = FixedClock::at(start());
        let user_store = InMemoryUsers::default();
        let users = UserService::new(user_store.clone(), clock.clone());
        let cards = CardService::new(InMemoryCards::default(), user_store, clock.clone());
        let alice = users
            .register(RegisterUser {
                name: "Alice".into(),
                email: "alice@example.com".into(),
            })
            .await
            .unwrap();
        Fixture {
            cards,
            users,
            clock,
            alice,
        }
    }

    fn issue(user_id: UserId, number: &str) -> IssueCard {
        IssueCard {
            user_id,
            card_number: number.into(),
            expires_at: Some(start() + Duration::days(30)),
        }
    }

    #[tokio::test]
    async fn issue_stores_active_card() {
        let f = fixture().await;

        let card = f
            .cards
            .issue(issue(f.alice.id(), "card-10001"))
            .await
            .unwrap();

        assert_eq!(card.card_number().as_str(), "CARD-10001");
        assert_eq!(card.user_id(), f.alice.id());
        assert_eq!(card.status(), CardStatus::Active);
        assert_eq!(card.issued_at(), start());
        assert_eq!(f.cards.get(card.id()).await.unwrap(), card);
    }

    #[tokio::test]
    async fn issue_to_unknown_user_is_not_found() {
        let f = fixture().await;
        let err = f
            .cards
            .issue(issue(UserId::generate(), "CARD-1"))
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound { entity: "user" }));
    }

    #[tokio::test]
    async fn issue_to_suspended_user_is_denied() {
        let f = fixture().await;
        f.users.suspend(f.alice.id()).await.unwrap();

        let err = f
            .cards
            .issue(issue(f.alice.id(), "CARD-1"))
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::InvalidTransition { entity: "user", .. })
        ));
    }

    #[tokio::test]
    async fn duplicate_card_number_is_a_conflict() {
        let f = fixture().await;
        f.cards.issue(issue(f.alice.id(), "CARD-1")).await.unwrap();

        let err = f
            .cards
            .issue(issue(f.alice.id(), "card-1"))
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            ApplicationError::Conflict {
                field: "card_number"
            }
        ));
    }

    #[tokio::test]
    async fn expiry_in_the_past_is_rejected() {
        let f = fixture().await;
        let mut cmd = issue(f.alice.id(), "CARD-1");
        cmd.expires_at = Some(start() - Duration::seconds(1));

        let err = f.cards.issue(cmd).await.unwrap_err();

        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::Validation {
                field: "expires_at",
                ..
            })
        ));
    }

    #[tokio::test]
    async fn revoke_is_persisted_and_final() {
        let f = fixture().await;
        let card = f.cards.issue(issue(f.alice.id(), "CARD-1")).await.unwrap();

        f.cards.revoke(card.id()).await.unwrap();
        assert_eq!(
            f.cards.get(card.id()).await.unwrap().status(),
            CardStatus::Revoked
        );

        let err = f.cards.reactivate(card.id()).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::InvalidTransition { .. })
        ));
    }

    #[tokio::test]
    async fn cannot_reactivate_after_expiry() {
        let f = fixture().await;
        let card = f.cards.issue(issue(f.alice.id(), "CARD-1")).await.unwrap();
        f.cards.suspend(card.id()).await.unwrap();

        f.clock.advance(Duration::days(31));
        let err = f.cards.reactivate(card.id()).await.unwrap_err();

        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::Validation {
                field: "expires_at",
                ..
            })
        ));
        assert_eq!(
            f.cards.get(card.id()).await.unwrap().status(),
            CardStatus::Suspended
        );
    }

    #[tokio::test]
    async fn unknown_card_is_not_found() {
        let f = fixture().await;
        let err = f.cards.revoke(CardId::generate()).await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound { entity: "card" }));
    }

    #[tokio::test]
    async fn list_for_user_distinguishes_unknown_user() {
        let f = fixture().await;
        assert!(
            f.cards
                .list_for_user(f.alice.id())
                .await
                .unwrap()
                .is_empty()
        );

        let err = f.cards.list_for_user(UserId::generate()).await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound { entity: "user" }));
    }
}
