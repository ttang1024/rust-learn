import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import type { FormEvent } from 'react'
import { Link, useParams } from 'react-router'

import { byId, useApiMutation, useCanManage, useOptions } from '../api/hooks'
import type { AccessGroup, User } from '../api/types'
import { useAuth } from '../auth/context'
import { Button, ErrorMessage, PageHeader, Panel, SelectField, Table, Td } from '../components/ui'

export function GroupDetailPage() {
  const { id = '' } = useParams()
  const { client } = useAuth()
  const canManage = useCanManage()
  const group = useQuery({
    queryKey: ['access-groups', id],
    queryFn: () => client.get<AccessGroup>(`/access-groups/${id}`),
  })
  const members = useQuery({
    queryKey: ['access-groups', id, 'members'],
    queryFn: () => client.get<{ user_ids: string[] }>(`/access-groups/${id}/members`),
  })
  const users = useOptions<User>('users')
  const userName = byId(users.data?.items, (user) => `${user.name} (${user.email})`)
  const memberIds = new Set(members.data?.user_ids ?? [])

  const removeMember = useApiMutation(
    (userId: string) => client.delete(`/access-groups/${id}/members/${userId}`),
    [['access-groups', id, 'members']],
  )

  return (
    <>
      <PageHeader title={group.data?.name ?? 'Access group'}>
        <Link to="/access-groups" className="text-sm text-slate-600 underline">
          All groups
        </Link>
      </PageHeader>
      {group.data?.description && <p className="mb-4 text-slate-600">{group.data.description}</p>}
      <ErrorMessage error={group.error ?? members.error ?? removeMember.error} />

      {canManage && (
        <AddMemberForm
          groupId={id}
          candidates={(users.data?.items ?? []).filter((user) => user.status === 'active' && !memberIds.has(user.id))}
        />
      )}

      <Table headers={['Member', '']} empty={members.data?.user_ids.length === 0}>
        {members.data?.user_ids.map((userId) => (
          <tr key={userId}>
            <Td>{userName(userId)}</Td>
            <Td className="text-right">
              {canManage && <Button onClick={() => removeMember.mutate(userId)}>Remove</Button>}
            </Td>
          </tr>
        ))}
      </Table>
    </>
  )
}

function AddMemberForm({ groupId, candidates }: { groupId: string; candidates: User[] }) {
  const { client } = useAuth()
  const [userId, setUserId] = useState('')
  const add = useApiMutation(
    (user_id: string) => client.post(`/access-groups/${groupId}/members`, { user_id }),
    [['access-groups', groupId, 'members']],
  )

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    add.mutate(userId, { onSuccess: () => setUserId('') })
  }

  return (
    <Panel title="Add a member">
      <form onSubmit={onSubmit} className="grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
        <SelectField label="User" required value={userId} onChange={(e) => setUserId(e.target.value)}>
          <option value="">Choose a user…</option>
          {candidates.map((user) => (
            <option key={user.id} value={user.id}>
              {user.name} ({user.email})
            </option>
          ))}
        </SelectField>
        <Button type="submit" variant="primary" disabled={add.isPending}>
          Add
        </Button>
      </form>
      <div className="mt-3">
        <ErrorMessage error={add.error} />
      </div>
    </Panel>
  )
}
