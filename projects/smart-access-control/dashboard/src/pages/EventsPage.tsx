import { useQuery } from '@tanstack/react-query'
import { useSearchParams } from 'react-router'

import { useOptions } from '../api/hooks'
import type { AccessEvent, Door, Page, User } from '../api/types'
import { useAuth } from '../auth/context'
import { EventTable } from '../components/EventTable'
import { Button, ErrorMessage, PageHeader, Pager, SelectField, TextField } from '../components/ui'
import { DENIAL_REASONS, humanize } from '../lib/events'

const LIMIT = 50
const FILTERS = ['door_id', 'user_id', 'decision', 'reason', 'from', 'until'] as const

/**
 * The complete, stored event history. Filters live in the URL, so a
 * filtered view can be bookmarked or shared.
 */
export function EventsPage() {
  const { client } = useAuth()
  const [params, setParams] = useSearchParams()
  const doors = useOptions<Door>('doors')
  const users = useOptions<User>('users')
  const offset = Number(params.get('offset') ?? 0)

  const query = new URLSearchParams()
  for (const key of FILTERS) {
    const value = params.get(key)
    // <input type="datetime-local"> gives local time; the API wants RFC 3339.
    if (value) query.set(key, key === 'from' || key === 'until' ? new Date(value).toISOString() : value)
  }
  query.set('limit', String(LIMIT))
  query.set('offset', String(offset))

  const events = useQuery({
    queryKey: ['events', query.toString()],
    queryFn: () => client.get<Page<AccessEvent>>(`/events?${query}`),
    placeholderData: (previous) => previous,
  })

  const setFilter = (key: string, value: string) => {
    const next = new URLSearchParams(params)
    if (value) next.set(key, value)
    else next.delete(key)
    next.delete('offset') // a new filter starts at the first page
    setParams(next)
  }

  return (
    <>
      <PageHeader title="Event history">
        {FILTERS.some((key) => params.get(key)) && <Button onClick={() => setParams({})}>Clear filters</Button>}
      </PageHeader>
      <div className="mb-4 grid gap-3 sm:grid-cols-3 lg:grid-cols-6">
        <SelectField label="Door" value={params.get('door_id') ?? ''} onChange={(e) => setFilter('door_id', e.target.value)}>
          <option value="">Any</option>
          {doors.data?.items.map((door) => (
            <option key={door.id} value={door.id}>
              {door.name}
            </option>
          ))}
        </SelectField>
        <SelectField label="User" value={params.get('user_id') ?? ''} onChange={(e) => setFilter('user_id', e.target.value)}>
          <option value="">Any</option>
          {users.data?.items.map((user) => (
            <option key={user.id} value={user.id}>
              {user.name}
            </option>
          ))}
        </SelectField>
        <SelectField label="Decision" value={params.get('decision') ?? ''} onChange={(e) => setFilter('decision', e.target.value)}>
          <option value="">Any</option>
          <option value="granted">Granted</option>
          <option value="denied">Denied</option>
        </SelectField>
        <SelectField label="Reason" value={params.get('reason') ?? ''} onChange={(e) => setFilter('reason', e.target.value)}>
          <option value="">Any</option>
          {DENIAL_REASONS.map((reason) => (
            <option key={reason} value={reason}>
              {humanize(reason)}
            </option>
          ))}
        </SelectField>
        <TextField label="From" type="datetime-local" value={params.get('from') ?? ''} onChange={(e) => setFilter('from', e.target.value)} />
        <TextField label="Until" type="datetime-local" value={params.get('until') ?? ''} onChange={(e) => setFilter('until', e.target.value)} />
      </div>
      <ErrorMessage error={events.error} />
      <EventTable events={events.data?.items ?? []} />
      <Pager
        offset={offset}
        limit={LIMIT}
        count={events.data?.items.length ?? 0}
        onChange={(next) => {
          const updated = new URLSearchParams(params)
          updated.set('offset', String(next))
          setParams(updated)
        }}
      />
    </>
  )
}
