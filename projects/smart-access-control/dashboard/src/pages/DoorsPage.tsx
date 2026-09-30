import { useState } from 'react'
import type { FormEvent } from 'react'

import { useApiMutation, useCanManage, usePage } from '../api/hooks'
import type { Door, DoorStatus } from '../api/types'
import { useAuth } from '../auth/context'
import {
  Button,
  ConfirmButton,
  ErrorMessage,
  PageHeader,
  Pager,
  Panel,
  StatusBadge,
  Table,
  Td,
  TextField,
} from '../components/ui'
import { formatTime } from '../lib/format'

const LIMIT = 50

export function DoorsPage() {
  const { client } = useAuth()
  const canManage = useCanManage()
  const [offset, setOffset] = useState(0)
  const doors = usePage<Door>('doors', offset, LIMIT)
  const setStatus = useApiMutation(
    ({ id, status }: { id: string; status: DoorStatus }) => client.patch(`/doors/${id}/status`, { status }),
    [['doors']],
  )

  return (
    <>
      <PageHeader title="Doors" />
      <p className="mb-4 text-sm text-slate-600">
        Doors come online when their controller sends a heartbeat. The buttons below are manual overrides.
      </p>
      {canManage && <CreateDoorForm />}
      <ErrorMessage error={doors.error ?? setStatus.error} />
      <Table headers={['Name', 'Location', 'Controller', 'Status', 'Created', '']} empty={doors.data?.items.length === 0}>
        {doors.data?.items.map((door) => (
          <tr key={door.id}>
            <Td>{door.name}</Td>
            <Td className="text-slate-600">{door.location}</Td>
            <Td className="font-mono text-slate-600">{door.controller_id}</Td>
            <Td>
              <StatusBadge status={door.status} />
            </Td>
            <Td className="text-slate-500">{formatTime(door.created_at)}</Td>
            <Td className="space-x-2 text-right whitespace-nowrap">
              {canManage && door.status === 'disabled' && (
                <Button onClick={() => setStatus.mutate({ id: door.id, status: 'offline' })}>Enable</Button>
              )}
              {canManage && door.status === 'offline' && (
                <Button onClick={() => setStatus.mutate({ id: door.id, status: 'online' })}>Bring online</Button>
              )}
              {canManage && door.status === 'online' && (
                <Button onClick={() => setStatus.mutate({ id: door.id, status: 'offline' })}>Take offline</Button>
              )}
              {canManage && door.status !== 'disabled' && (
                <ConfirmButton label="Disable" onConfirm={() => setStatus.mutate({ id: door.id, status: 'disabled' })} />
              )}
            </Td>
          </tr>
        ))}
      </Table>
      <Pager offset={offset} limit={LIMIT} count={doors.data?.items.length ?? 0} onChange={setOffset} />
    </>
  )
}

function CreateDoorForm() {
  const { client } = useAuth()
  const [form, setForm] = useState({ name: '', location: '', controller_id: '' })
  const create = useApiMutation((body: typeof form) => client.post<Door>('/doors', body), [['doors']])
  const set = (field: keyof typeof form) => (e: { target: { value: string } }) =>
    setForm({ ...form, [field]: e.target.value })

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    create.mutate(form, { onSuccess: () => setForm({ name: '', location: '', controller_id: '' }) })
  }

  return (
    <Panel title="Add a door">
      <form onSubmit={onSubmit} className="grid gap-3 sm:grid-cols-[1fr_1fr_1fr_auto] sm:items-end">
        <TextField label="Name" required value={form.name} onChange={set('name')} />
        <TextField label="Location" required value={form.location} onChange={set('location')} />
        <TextField label="Controller id" required placeholder="ctrl-001" value={form.controller_id} onChange={set('controller_id')} />
        <Button type="submit" variant="primary" disabled={create.isPending}>
          Add
        </Button>
      </form>
      <div className="mt-3">
        <ErrorMessage error={create.error} />
      </div>
    </Panel>
  )
}
