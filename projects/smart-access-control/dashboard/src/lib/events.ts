import type { DenialReason } from '../api/types'

export const DENIAL_REASONS: DenialReason[] = [
  'unknown_card',
  'card_revoked',
  'card_suspended',
  'card_expired',
  'user_suspended',
  'user_archived',
  'unknown_door',
  'door_disabled',
  'door_offline',
  'permission_denied',
  'outside_schedule',
]

/** "door_offline" -> "door offline". */
export function humanize(value: string): string {
  return value.replaceAll('_', ' ')
}
