-- Door controllers and their credentials (Phase 6). Simulated devices only.

CREATE TABLE controllers (
    -- The same identifier doors use in doors.controller_id. No foreign key
    -- from doors: a door may name a controller that is not registered yet.
    id            TEXT        PRIMARY KEY CHECK (id ~ '^[A-Za-z0-9_-]{1,64}$'),
    -- SHA-256 (hex) of the controller's secret key; the key is never stored.
    key_hash      TEXT        NOT NULL CHECK (key_hash ~ '^[0-9a-f]{64}$'),
    status        TEXT        NOT NULL CHECK (status IN ('online', 'offline')),
    last_seen_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL,
    CONSTRAINT controllers_key_hash_key UNIQUE (key_hash)
);

-- The liveness monitor looks for online controllers that went silent.
CREATE INDEX controllers_online_last_seen_idx ON controllers (last_seen_at) WHERE status = 'online';

-- Doors are looked up by controller on every heartbeat.
-- (doors_controller_id_idx already exists from the first migration.)

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
        'permission_granted', 'permission_revoked',
        'controller_registered', 'controller_key_rotated'
    ));
