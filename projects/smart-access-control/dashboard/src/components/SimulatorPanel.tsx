import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import type { FormEvent } from 'react'

import { ApiError } from '../api/client'
import { useApiMutation, useCanManage } from '../api/hooks'
import type { AccessEvent, Door, SimulatedController } from '../api/types'
import { useAuth } from '../auth/context'
import { humanize } from '../lib/events'
import { formatTime } from '../lib/format'
import { Button, ErrorMessage, Panel, SelectField, StatusBadge, Table, Td, TextField } from './ui'

const SIMULATOR = ['simulator', 'controllers']

/**
 * Drives the backend's in-process controller simulator (only available when
 * the server runs with SIMULATOR_ENABLED=true). Everything here is a
 * simulation; no real hardware is involved.
 */
export function SimulatorPanel({ doors }: { doors: Door[] }) {
  const { client } = useAuth()
  const canManage = useCanManage()
  const running = useQuery({
    queryKey: SIMULATOR,
    queryFn: () => client.get<SimulatedController[]>('/simulator/controllers'),
    retry: false,
    // Heartbeat counters and network state change in the background.
    refetchInterval: 3000,
  })
  const act = useApiMutation(
    ({ id, action }: { id: string; action: 'outage' | 'disconnect' | 'reconnect' | 'stop' }) =>
      action === 'stop'
        ? client.delete(`/simulator/controllers/${id}`)
        : client.post(`/simulator/controllers/${id}/${action}`),
    [SIMULATOR, ['doors'], ['controllers']],
  )

  if (running.error instanceof ApiError && running.error.status === 404) {
    return (
      <Panel title="Simulator">
        <p className="text-sm text-slate-600">
          The simulator is disabled on this server. Start the backend with <code>SIMULATOR_ENABLED=true</code> to
          simulate door controllers.
        </p>
      </Panel>
    )
  }

  const controllers = running.data ?? []
  return (
    <Panel title="Simulator (simulated controllers, no real hardware)">
      <ErrorMessage error={running.error ?? act.error} />
      {canManage && <StartForm doors={doors} running={controllers} />}
      <Table headers={['Controller', 'Network', 'Heartbeats', 'Last heartbeat', 'Last error', '']} empty={controllers.length === 0}>
        {controllers.map((c) => (
          <tr key={c.controller_id}>
            <Td className="font-mono">{c.controller_id}</Td>
            <Td>
              <StatusBadge status={c.network === 'up' ? 'online' : 'offline'} />
            </Td>
            <Td>{c.heartbeats_sent}</Td>
            <Td className="text-slate-500">{formatTime(c.last_heartbeat_at)}</Td>
            <Td className="text-slate-500">{c.last_error ?? '—'}</Td>
            <Td className="space-x-1 text-right whitespace-nowrap">
              {canManage && c.network === 'up' && (
                <>
                  <Button onClick={() => act.mutate({ id: c.controller_id, action: 'outage' })}>Outage</Button>
                  <Button onClick={() => act.mutate({ id: c.controller_id, action: 'disconnect' })}>Disconnect</Button>
                </>
              )}
              {canManage && c.network === 'down' && (
                <Button onClick={() => act.mutate({ id: c.controller_id, action: 'reconnect' })}>Reconnect</Button>
              )}
              {canManage && <Button onClick={() => act.mutate({ id: c.controller_id, action: 'stop' })}>Stop</Button>}
            </Td>
          </tr>
        ))}
      </Table>
      {canManage && controllers.length > 0 && <SwipeForm doors={doors} running={controllers} />}
    </Panel>
  )
}

function StartForm({ doors, running }: { doors: Door[]; running: SimulatedController[] }) {
  const { client } = useAuth()
  const [controllerId, setControllerId] = useState('')
  const start = useApiMutation(
    (controller_id: string) => client.post('/simulator/controllers', { controller_id }),
    [SIMULATOR, ['controllers'], ['doors']],
  )
  const active = new Set(running.map((c) => c.controller_id))
  // Suggest the controllers that doors are wired to.
  const suggestions = [...new Set(doors.map((d) => d.controller_id))].filter((id) => !active.has(id))

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    start.mutate(controllerId, { onSuccess: () => setControllerId('') })
  }

  return (
    <form onSubmit={onSubmit} className="mb-3 grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
      <TextField
        label="Start simulating controller"
        list="controller-suggestions"
        required
        placeholder="ctrl-001"
        value={controllerId}
        onChange={(e) => setControllerId(e.target.value)}
      />
      <datalist id="controller-suggestions">
        {suggestions.map((id) => (
          <option key={id} value={id} />
        ))}
      </datalist>
      <Button type="submit" variant="primary" disabled={start.isPending}>
        Start
      </Button>
      <div className="sm:col-span-2">
        <ErrorMessage error={start.error} />
      </div>
    </form>
  )
}

function SwipeForm({ doors, running }: { doors: Door[]; running: SimulatedController[] }) {
  const { client } = useAuth()
  const [controllerId, setControllerId] = useState(running[0]?.controller_id ?? '')
  const [doorId, setDoorId] = useState('')
  const [cardNumber, setCardNumber] = useState('')
  const swipe = useApiMutation(
    (body: { controllerId: string; card_number: string; door_id: string }) =>
      client.post<AccessEvent>(`/simulator/controllers/${body.controllerId}/swipe`, {
        card_number: body.card_number,
        door_id: body.door_id,
      }),
    [['events']],
  )
  // A controller can only badge at its own doors.
  const ownDoors = doors.filter((door) => door.controller_id === controllerId)

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    swipe.mutate({ controllerId, card_number: cardNumber, door_id: doorId })
  }

  return (
    <form onSubmit={onSubmit} className="mt-4 grid gap-3 border-t border-slate-100 pt-4 sm:grid-cols-[1fr_1fr_1fr_auto] sm:items-end">
      <SelectField
        label="Controller"
        value={controllerId}
        onChange={(e) => {
          setControllerId(e.target.value)
          setDoorId('')
        }}
      >
        {running.map((c) => (
          <option key={c.controller_id} value={c.controller_id}>
            {c.controller_id}
          </option>
        ))}
      </SelectField>
      <SelectField label="At door" required value={doorId} onChange={(e) => setDoorId(e.target.value)}>
        <option value="">Choose a door…</option>
        {ownDoors.map((door) => (
          <option key={door.id} value={door.id}>
            {door.name}
          </option>
        ))}
      </SelectField>
      <TextField label="Card number" required placeholder="CARD-10001" value={cardNumber} onChange={(e) => setCardNumber(e.target.value)} />
      <Button type="submit" variant="primary" disabled={swipe.isPending}>
        Swipe card
      </Button>
      <div className="sm:col-span-4">
        {swipe.data && (
          <p className="text-sm">
            Result: <StatusBadge status={swipe.data.decision} />
            {swipe.data.reason && <span className="ml-2">{humanize(swipe.data.reason)}</span>}
          </p>
        )}
        <ErrorMessage error={swipe.error} />
      </div>
    </form>
  )
}
