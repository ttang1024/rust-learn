import { useState } from 'react'
import { Link } from 'react-router'

import type { EventFilter, StreamStatus } from '../api/eventStream'
import { useOptions } from '../api/hooks'
import type { Door } from '../api/types'
import { useEventStream } from '../api/useEventStream'
import { EventTable } from '../components/EventTable'
import { SimulatorPanel } from '../components/SimulatorPanel'
import { PageHeader, SelectField } from '../components/ui'

const STATUS_LABEL: Record<StreamStatus, { text: string; color: string }> = {
  connecting: { text: 'Connecting…', color: 'bg-slate-200 text-slate-700' },
  live: { text: 'Live', color: 'bg-green-100 text-green-800' },
  reconnecting: { text: 'Reconnecting…', color: 'bg-amber-100 text-amber-800' },
  stopped: { text: 'Disconnected', color: 'bg-red-100 text-red-800' },
}

export function MonitorPage() {
  const doors = useOptions<Door>('doors')
  const [doorId, setDoorId] = useState('')
  const [decision, setDecision] = useState('')
  const filter: EventFilter = {
    ...(doorId && { door_id: doorId }),
    ...(decision && { decision: decision as EventFilter['decision'] }),
  }
  const stream = useEventStream(filter)
  const status = STATUS_LABEL[stream.status]

  return (
    <>
      <PageHeader title="Live monitor">
        <span role="status" className={`rounded px-2 py-1 text-sm font-medium ${status.color}`}>
          {status.text}
        </span>
      </PageHeader>

      <SimulatorPanel doors={doors.data?.items ?? []} />

      <div className="mb-3 grid gap-3 sm:grid-cols-3">
        <SelectField label="Door" value={doorId} onChange={(e) => setDoorId(e.target.value)}>
          <option value="">All doors</option>
          {doors.data?.items.map((door) => (
            <option key={door.id} value={door.id}>
              {door.name}
            </option>
          ))}
        </SelectField>
        <SelectField label="Decision" value={decision} onChange={(e) => setDecision(e.target.value)}>
          <option value="">Granted and denied</option>
          <option value="granted">Granted</option>
          <option value="denied">Denied</option>
        </SelectField>
      </div>

      {stream.error && (
        <p role="alert" className="mb-3 rounded border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
          {stream.error}
        </p>
      )}
      {(stream.missed > 0 || stream.interrupted) && (
        <p className="mb-3 rounded border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-900">
          {stream.missed > 0 && `${stream.missed} events were skipped because the feed fell behind. `}
          {stream.interrupted && 'The connection was interrupted; events from the gap are not replayed. '}
          The <Link to="/events" className="underline">event history</Link> is complete.
        </p>
      )}

      <p className="mb-2 text-sm text-slate-500">
        Shows events as they happen from the moment this page opened (newest first, last 200).
      </p>
      <EventTable events={stream.events} />
    </>
  )
}
