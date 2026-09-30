import { useState } from 'react'
import type { FormEvent } from 'react'

import { byId, useApiMutation, useCanManage, useOptions, usePage } from '../api/hooks'
import type { AccessGroup, Door, Permission, Schedule } from '../api/types'
import { useAuth } from '../auth/context'
import { Button, ConfirmButton, ErrorMessage, PageHeader, Pager, Panel, SelectField, Table, Td } from '../components/ui'
import { formatTime } from '../lib/format'

const LIMIT = 50

export function PermissionsPage() {
  const { client } = useAuth()
  const canManage = useCanManage()
  const [offset, setOffset] = useState(0)
  const permissions = usePage<Permission>('permissions', offset, LIMIT)
  const groups = useOptions<AccessGroup>('access-groups')
  const doors = useOptions<Door>('doors')
  const schedules = useOptions<Schedule>('schedules')
  const groupName = byId(groups.data?.items, (g) => g.name)
  const doorName = byId(doors.data?.items, (d) => d.name)
  const scheduleName = byId(schedules.data?.items, (s) => s.name)
  const revoke = useApiMutation((id: string) => client.delete(`/permissions/${id}`), [['permissions']])

  return (
    <>
      <PageHeader title="Permissions" />
      <p className="mb-4 text-sm text-slate-600">
        A permission lets every member of a group use a door, either at any time or within a schedule.
      </p>
      {canManage && (
        <GrantForm groups={groups.data?.items ?? []} doors={doors.data?.items ?? []} schedules={schedules.data?.items ?? []} />
      )}
      <ErrorMessage error={permissions.error ?? revoke.error} />
      <Table headers={['Group', 'Door', 'When', 'Granted', '']} empty={permissions.data?.items.length === 0}>
        {permissions.data?.items.map((permission) => (
          <tr key={permission.id}>
            <Td>{groupName(permission.group_id)}</Td>
            <Td>{doorName(permission.door_id)}</Td>
            <Td>{permission.schedule_id ? scheduleName(permission.schedule_id) : 'Any time'}</Td>
            <Td className="text-slate-500">{formatTime(permission.created_at)}</Td>
            <Td className="text-right">
              {canManage && <ConfirmButton label="Revoke" onConfirm={() => revoke.mutate(permission.id)} />}
            </Td>
          </tr>
        ))}
      </Table>
      <Pager offset={offset} limit={LIMIT} count={permissions.data?.items.length ?? 0} onChange={setOffset} />
    </>
  )
}

function GrantForm({ groups, doors, schedules }: { groups: AccessGroup[]; doors: Door[]; schedules: Schedule[] }) {
  const { client } = useAuth()
  const [groupId, setGroupId] = useState('')
  const [doorId, setDoorId] = useState('')
  const [scheduleId, setScheduleId] = useState('')
  const grant = useApiMutation(
    (body: { group_id: string; door_id: string; schedule_id: string | null }) => client.post<Permission>('/permissions', body),
    [['permissions']],
  )

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    grant.mutate({ group_id: groupId, door_id: doorId, schedule_id: scheduleId || null })
  }

  return (
    <Panel title="Grant access">
      <form onSubmit={onSubmit} className="grid gap-3 sm:grid-cols-[1fr_1fr_1fr_auto] sm:items-end">
        <SelectField label="Group" required value={groupId} onChange={(e) => setGroupId(e.target.value)}>
          <option value="">Choose a group…</option>
          {groups.map((g) => (
            <option key={g.id} value={g.id}>
              {g.name}
            </option>
          ))}
        </SelectField>
        <SelectField label="Door" required value={doorId} onChange={(e) => setDoorId(e.target.value)}>
          <option value="">Choose a door…</option>
          {doors.map((d) => (
            <option key={d.id} value={d.id}>
              {d.name} ({d.location})
            </option>
          ))}
        </SelectField>
        <SelectField label="When" value={scheduleId} onChange={(e) => setScheduleId(e.target.value)}>
          <option value="">Any time</option>
          {schedules.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </SelectField>
        <Button type="submit" variant="primary" disabled={grant.isPending}>
          Grant
        </Button>
      </form>
      <div className="mt-3">
        <ErrorMessage error={grant.error} />
      </div>
    </Panel>
  )
}
