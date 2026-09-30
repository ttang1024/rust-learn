import type { ScheduleRule } from '../api/types'

/** A timestamp in the viewer's locale and time zone; "—" for none. */
export function formatTime(value: string | null): string {
  return value ? new Date(value).toLocaleString() : '—'
}

/** e.g. "mon, tue 22:00–06:00 (ends next day)". */
export function describeRule(rule: ScheduleRule): string {
  const overnight = rule.end <= rule.start ? ' (ends next day)' : ''
  return `${rule.days.join(', ')} ${rule.start}–${rule.end}${overnight}`
}
