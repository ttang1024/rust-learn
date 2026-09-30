-- Administrator accounts, refresh tokens and the administrative audit log (Phase 4a).

CREATE TABLE administrators (
    id             UUID        PRIMARY KEY,
    username       TEXT        NOT NULL CHECK (username ~ '^[a-z0-9._-]{3,64}$'),
    -- Argon2id in PHC string format. Never a plaintext password.
    password_hash  TEXT        NOT NULL CHECK (password_hash LIKE '$argon2id$%'),
    role           TEXT        NOT NULL CHECK (role IN ('admin', 'viewer')),
    status         TEXT        NOT NULL CHECK (status IN ('active', 'disabled')),
    created_at     TIMESTAMPTZ NOT NULL,
    CONSTRAINT administrators_username_key UNIQUE (username)
);

CREATE TABLE refresh_tokens (
    -- SHA-256 (hex) of the token. The token itself is never stored, so a
    -- database leak does not hand out usable sessions.
    token_hash        TEXT        PRIMARY KEY CHECK (token_hash ~ '^[0-9a-f]{64}$'),
    administrator_id  UUID        NOT NULL REFERENCES administrators (id) ON DELETE CASCADE,
    created_at        TIMESTAMPTZ NOT NULL,
    expires_at        TIMESTAMPTZ NOT NULL,
    revoked_at        TIMESTAMPTZ,
    -- Why the token stopped working: only reuse of a *rotated* token is
    -- treated as theft.
    revoked_reason    TEXT        CHECK (revoked_reason IN ('rotated', 'logged_out', 'revoked')),
    CONSTRAINT refresh_tokens_revocation_complete
        CHECK ((revoked_at IS NULL) = (revoked_reason IS NULL))
);

-- "Revoke all sessions of this administrator" only touches live tokens.
CREATE INDEX refresh_tokens_live_by_administrator_idx
    ON refresh_tokens (administrator_id) WHERE revoked_at IS NULL;

CREATE TABLE admin_audit_log (
    id                UUID        PRIMARY KEY,
    -- NULL for anonymous attempts and command-line bootstrap.
    administrator_id  UUID        REFERENCES administrators (id),
    action            TEXT        NOT NULL CHECK (action IN (
        'login_succeeded', 'login_failed', 'logged_out', 'refresh_token_reused',
        'administrator_created'
    )),
    subject           TEXT        CHECK (char_length(subject) BETWEEN 1 AND 200),
    occurred_at       TIMESTAMPTZ NOT NULL
);

CREATE INDEX admin_audit_log_occurred_at_idx ON admin_audit_log (occurred_at DESC, id DESC);

-- Same append-only protection as access_events, via a generic function that
-- names the table it protects. (Applied migrations are immutable, so the
-- access_events trigger keeps its original function.)
CREATE FUNCTION reject_changes_to_append_only_table() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION '% is append-only (% rejected)', TG_TABLE_NAME, TG_OP
        USING ERRCODE = 'insufficient_privilege';
END;
$$;

CREATE TRIGGER admin_audit_log_no_update_or_delete
    BEFORE UPDATE OR DELETE ON admin_audit_log
    FOR EACH ROW EXECUTE FUNCTION reject_changes_to_append_only_table();

CREATE TRIGGER admin_audit_log_no_truncate
    BEFORE TRUNCATE ON admin_audit_log
    FOR EACH STATEMENT EXECUTE FUNCTION reject_changes_to_append_only_table();
