-- Access policy (groups, schedules, permissions) and the access event log (Phase 3).

CREATE TABLE access_groups (
    id           UUID PRIMARY KEY,
    name         TEXT NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
    description  TEXT CHECK (description IS NULL OR char_length(description) BETWEEN 1 AND 500),
    CONSTRAINT access_groups_name_key UNIQUE (name)
);

-- Deleting a group removes its memberships (and, below, its permissions):
-- deleting can only ever take access away, never grant it.
CREATE TABLE user_access_groups (
    user_id   UUID NOT NULL REFERENCES users (id),
    group_id  UUID NOT NULL REFERENCES access_groups (id) ON DELETE CASCADE,
    CONSTRAINT user_access_groups_pkey PRIMARY KEY (user_id, group_id)
);

-- The primary key serves lookups by user; this serves lookups by group.
CREATE INDEX user_access_groups_group_id_idx ON user_access_groups (group_id);

CREATE TABLE access_schedules (
    id               UUID PRIMARY KEY,
    name             TEXT NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
    -- IANA name, e.g. 'Europe/London'. Validated by the application.
    timezone         TEXT NOT NULL,
    -- Local calendar dates in `timezone`, both inclusive.
    effective_from   DATE,
    effective_until  DATE,
    CONSTRAINT access_schedules_name_key UNIQUE (name),
    CONSTRAINT access_schedules_effective_range CHECK (
        effective_from IS NULL OR effective_until IS NULL OR effective_until >= effective_from
    )
);

CREATE TABLE access_schedule_rules (
    schedule_id  UUID     NOT NULL REFERENCES access_schedules (id) ON DELETE CASCADE,
    -- Keeps rules in the order they were defined.
    position     SMALLINT NOT NULL CHECK (position >= 0),
    -- Weekday bit mask: bit 0 = Monday ... bit 6 = Sunday.
    days         SMALLINT NOT NULL CHECK (days BETWEEN 1 AND 127),
    -- Local wall-clock times. end_time <= start_time means the window ends the next day.
    start_time   TIME     NOT NULL,
    end_time     TIME     NOT NULL,
    CONSTRAINT access_schedule_rules_pkey PRIMARY KEY (schedule_id, position)
);

CREATE TABLE access_permissions (
    id           UUID        PRIMARY KEY,
    group_id     UUID        NOT NULL REFERENCES access_groups (id) ON DELETE CASCADE,
    door_id      UUID        NOT NULL REFERENCES doors (id),
    -- NULL = at any time. RESTRICT (the default) on purpose: deleting a
    -- schedule must never silently turn a limited permission into 24/7 access.
    schedule_id  UUID        REFERENCES access_schedules (id),
    created_at   TIMESTAMPTZ NOT NULL,
    -- NULLS NOT DISTINCT (PostgreSQL 15+): two unscheduled permissions for
    -- the same group and door are duplicates too.
    CONSTRAINT access_permissions_group_door_schedule_key
        UNIQUE NULLS NOT DISTINCT (group_id, door_id, schedule_id)
);

-- The decision engine looks permissions up by door.
CREATE INDEX access_permissions_door_id_idx ON access_permissions (door_id);

CREATE TABLE access_events (
    id           UUID        PRIMARY KEY,
    -- The presented number, kept even when no such card exists.
    card_number  TEXT,
    -- Set only when the referenced row exists.
    card_id      UUID        REFERENCES access_cards (id),
    user_id      UUID        REFERENCES users (id),
    door_id      UUID        REFERENCES doors (id),
    decision     TEXT        NOT NULL CHECK (decision IN ('granted', 'denied')),
    reason       TEXT        CHECK (reason IN (
        'unknown_card', 'card_revoked', 'card_suspended', 'card_expired',
        'user_suspended', 'user_archived', 'unknown_door', 'door_disabled',
        'door_offline', 'permission_denied', 'outside_schedule'
    )),
    occurred_at  TIMESTAMPTZ NOT NULL,
    -- A denial always has a reason; a grant never does.
    CONSTRAINT access_events_reason_matches_decision CHECK ((decision = 'granted') = (reason IS NULL))
);

CREATE INDEX access_events_occurred_at_idx ON access_events (occurred_at DESC, id DESC);
CREATE INDEX access_events_door_id_idx ON access_events (door_id, occurred_at DESC);
CREATE INDEX access_events_user_id_idx ON access_events (user_id, occurred_at DESC);
CREATE INDEX access_events_card_id_idx ON access_events (card_id, occurred_at DESC);

-- Audit history is append-only: reject every UPDATE, DELETE and TRUNCATE,
-- whether it comes from a bug in the application or a manual SQL session.
CREATE FUNCTION reject_access_event_changes() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'access_events is append-only (% rejected)', TG_OP
        USING ERRCODE = 'insufficient_privilege';
END;
$$;

CREATE TRIGGER access_events_no_update_or_delete
    BEFORE UPDATE OR DELETE ON access_events
    FOR EACH ROW EXECUTE FUNCTION reject_access_event_changes();

CREATE TRIGGER access_events_no_truncate
    BEFORE TRUNCATE ON access_events
    FOR EACH STATEMENT EXECUTE FUNCTION reject_access_event_changes();
