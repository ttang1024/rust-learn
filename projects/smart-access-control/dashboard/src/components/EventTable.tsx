import { byId, useOptions } from '../api/hooks'
import type { AccessEvent, Door, User } from '../api/types'
import { formatTime } from '../lib/format'
import { StatusBadge, Table, Td } from './ui'

/** Access events with door and user names resolved. */
export function EventTable({ events }: { events: AccessEvent[] }) {
  const doors = useOptions<Door>('doors')
  const users = useOptions<User>('users')
  const doorName = byId(doors.data?.items, (door) => door.name)
  const userName = byId(users.data?.items, (user) => user.name)

  return (
    <Table headers={['Time', 'Door', 'Card', 'Holder', 'Decision']} empty={events.length === 0}>
      {events.map((event) => (
        <tr key={event.id}>
          <Td className="whitespace-nowrap text-slate-500">{formatTime(event.occurred_at)}</Td>
          <Td>{doorName(event.door_id)}</Td>
          <Td className="font-mono">{event.card_number ?? '—'}</Td>
          <Td>{userName(event.user_id)}</Td>
          <Td>
            <StatusBadge status={event.decision} />
            {event.reason && <span className="ml-2 text-slate-600">{event.reason.replaceAll('_', ' ')}</span>}
          </Td>
        </tr>
      ))}
    </Table>
  )
}
