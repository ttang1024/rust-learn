-- Phase 4b: the admin audit log also records management changes.
-- Replaces the CHECK constraint that lists the allowed actions (Postgres
-- names an unnamed column CHECK `<table>_<column>_check`).

ALTER TABLE admin_audit_log
    DROP CONSTRAINT admin_audit_log_action_check,
    ADD CONSTRAINT admin_audit_log_action_check CHECK (action IN (
        'login_succeeded', 'login_failed', 'logged_out', 'refresh_token_reused',
        'administrator_created',
        'user_registered', 'user_updated', 'user_suspended', 'user_reactivated', 'user_archived',
        'card_issued', 'card_suspended', 'card_reactivated', 'card_revoked',
        'door_created', 'door_updated', 'door_status_changed',
        'access_group_created', 'access_group_updated', 'access_group_deleted',
        'group_member_added', 'group_member_removed',
        'schedule_created',
        'permission_granted', 'permission_revoked'
    ));
