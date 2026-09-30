//! Recording and reading the administrative audit trail.

use std::sync::Arc;

use super::{ApplicationError, AuditLogRepository, Clock, PageRequest};
use crate::{
    domain::{AdministratorId, AuditAction, AuditEntry, AuditSubject},
    error_chain,
};

pub struct AuditTrail<L> {
    log: L,
    clock: Arc<dyn Clock>,
}

impl<L: AuditLogRepository> AuditTrail<L> {
    pub fn new(log: L, clock: Arc<dyn Clock>) -> Self {
        Self { log, clock }
    }

    /// Records that `actor` completed `action` on `subject` (e.g. `user:<id>`).
    ///
    /// Called *after* the change succeeded, and not in the same transaction.
    /// If writing the entry fails, the change stands: the failure is logged
    /// at error level instead of being returned, because reporting an error
    /// for a change that did happen would invite a confusing retry.
    pub async fn record(&self, actor: AdministratorId, action: AuditAction, subject: &str) {
        let subject = match AuditSubject::parse(subject) {
            Ok(subject) => Some(subject),
            Err(err) => {
                tracing::error!(%err, action = action.as_str(), "invalid audit subject");
                None
            }
        };
        let entry = AuditEntry::record(Some(actor), action, subject, self.clock.now());
        if let Err(err) = self.log.append(&entry).await {
            tracing::error!(
                error = %error_chain(&err),
                %actor,
                action = action.as_str(),
                "failed to write audit entry"
            );
        }
    }

    pub async fn list_recent(
        &self,
        page: PageRequest,
    ) -> Result<Vec<AuditEntry>, ApplicationError> {
        Ok(self.log.list_recent(page).await?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::application::fakes::{FixedClock, InMemoryAudit};

    #[tokio::test]
    async fn records_actor_action_subject_and_time() {
        let now = Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap();
        let trail = AuditTrail::new(InMemoryAudit::default(), FixedClock::at(now));
        let actor = AdministratorId::generate();

        trail
            .record(actor, AuditAction::CardRevoked, "card:123")
            .await;

        let entries = trail.list_recent(PageRequest::default()).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].actor(), Some(actor));
        assert_eq!(entries[0].action(), AuditAction::CardRevoked);
        assert_eq!(
            entries[0].subject().map(AuditSubject::as_str),
            Some("card:123")
        );
        assert_eq!(entries[0].occurred_at(), now);
    }
}
