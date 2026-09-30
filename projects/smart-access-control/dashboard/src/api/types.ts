// Response and request shapes of the backend API (`/api/v1`).
//
// Hand-written to mirror the Rust DTOs in `src/interfaces/http/*.rs`.
// Generating them from Rust would add a frontend-oriented dependency to the
// backend, which the project rules forbid; the backend's JSON shapes are
// pinned by its HTTP integration tests instead.

/** RFC 3339 timestamp, e.g. "2026-09-30T10:00:00Z". */
export type Timestamp = string
export type Uuid = string

export type Role = 'admin' | 'viewer'

export interface Page<T> {
  items: T[]
  limit: number
  offset: number
}

export interface ErrorBody {
  error: { code: string; message: string }
}

export interface TokenResponse {
  access_token: string
  token_type: 'Bearer'
  expires_in: number
  /** Only in body mode; the dashboard uses cookie mode and never sees it. */
  refresh_token?: string
  refresh_expires_in: number
}

export interface Me {
  id: Uuid
  role: Role
}

export interface Health {
  status: 'ok'
  version: string
}

export interface Readiness {
  status: 'ready' | 'unavailable'
}

export type UserStatus = 'active' | 'suspended' | 'archived'

export interface User {
  id: Uuid
  name: string
  email: string
  status: UserStatus
  created_at: Timestamp
  updated_at: Timestamp
}

export type CardStatus = 'active' | 'suspended' | 'revoked' | 'expired'

export interface Card {
  id: Uuid
  user_id: Uuid
  card_number: string
  status: CardStatus
  effective_status: CardStatus
  issued_at: Timestamp
  expires_at: Timestamp | null
}

export type DoorStatus = 'online' | 'offline' | 'disabled'

export interface Door {
  id: Uuid
  name: string
  location: string
  controller_id: string
  status: DoorStatus
  created_at: Timestamp
}

export interface AccessGroup {
  id: Uuid
  name: string
  description: string | null
}

export type Weekday = 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun'

export interface ScheduleRule {
  days: Weekday[]
  /** "HH:MM" or "HH:MM:SS" in the schedule's time zone. */
  start: string
  end: string
}

export interface Schedule {
  id: Uuid
  name: string
  timezone: string
  rules: ScheduleRule[]
  effective_from: string | null
  effective_until: string | null
}

export interface Permission {
  id: Uuid
  group_id: Uuid
  door_id: Uuid
  schedule_id: Uuid | null
  created_at: Timestamp
}

export type DenialReason =
  | 'unknown_card'
  | 'card_revoked'
  | 'card_suspended'
  | 'card_expired'
  | 'user_suspended'
  | 'user_archived'
  | 'unknown_door'
  | 'door_disabled'
  | 'door_offline'
  | 'permission_denied'
  | 'outside_schedule'

export interface AccessEvent {
  id: Uuid
  decision: 'granted' | 'denied'
  reason: DenialReason | null
  card_number: string | null
  card_id: Uuid | null
  user_id: Uuid | null
  door_id: Uuid | null
  occurred_at: Timestamp
}

export interface Controller {
  id: string
  status: 'online' | 'offline'
  last_seen_at: Timestamp | null
  created_at: Timestamp
}

export interface SimulatedController {
  controller_id: string
  simulated: true
  network: 'up' | 'down'
  heartbeats_sent: number
  last_heartbeat_at: Timestamp | null
  last_error: string | null
}
