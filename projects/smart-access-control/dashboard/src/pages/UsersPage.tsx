import { useState } from 'react'
import type { FormEvent } from 'react'
import { Link } from 'react-router'

import { useApiMutation, useCanManage, usePagedList } from '../api/hooks'
import type { User } from '../api/types'
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

export function UsersPage() {
  const { client } = useAuth()
  const canManage = useCanManage()
  const { page: users, pager } = usePagedList<User>('users')
  const setStatus = useApiMutation(
    ({ id, status }: { id: string; status: User['status'] }) =>
      status === 'archived' ? client.delete(`/users/${id}`) : client.patch(`/users/${id}`, { status }),
    [['users']],
  )

  return (
    <>
      <PageHeader title="Users" />
      {canManage && <CreateUserForm />}
      <ErrorMessage error={users.error ?? setStatus.error} />
      <Table headers={['Name', 'Email', 'Status', 'Created', '']} empty={users.data?.items.length === 0}>
        {users.data?.items.map((user) => (
          <tr key={user.id}>
            <Td>{user.name}</Td>
            <Td className="text-slate-600">{user.email}</Td>
            <Td>
              <StatusBadge status={user.status} />
            </Td>
            <Td className="text-slate-500">{formatTime(user.created_at)}</Td>
            <Td className="space-x-2 text-right whitespace-nowrap">
              <Link className="text-sm text-slate-600 underline" to={`/cards?user=${user.id}`}>
                Cards
              </Link>
              {canManage && (
                <>
                  {user.status === 'active' && (
                    <Button onClick={() => setStatus.mutate({ id: user.id, status: 'suspended' })}>Suspend</Button>
                  )}
                  {user.status === 'suspended' && (
                    <Button onClick={() => setStatus.mutate({ id: user.id, status: 'active' })}>Reactivate</Button>
                  )}
                  {user.status !== 'archived' && (
                    <ConfirmButton label="Archive" onConfirm={() => setStatus.mutate({ id: user.id, status: 'archived' })} />
                  )}
                </>
              )}
            </Td>
          </tr>
        ))}
      </Table>
      <Pager {...pager} />
    </>
  )
}

function CreateUserForm() {
  const { client } = useAuth()
  const [name, setName] = useState('')
  const [email, setEmail] = useState('')
  const create = useApiMutation((body: { name: string; email: string }) => client.post<User>('/users', body), [['users']])

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    create.mutate(
      { name, email },
      {
        onSuccess: () => {
          setName('')
          setEmail('')
        },
      },
    )
  }

  return (
    <Panel title="Register a user">
      <form onSubmit={onSubmit} className="grid gap-3 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
        <TextField label="Name" required value={name} onChange={(e) => setName(e.target.value)} />
        <TextField label="Email" type="email" required value={email} onChange={(e) => setEmail(e.target.value)} />
        <Button type="submit" variant="primary" disabled={create.isPending}>
          Register
        </Button>
      </form>
      <ErrorMessage className="mt-3" error={create.error} />
    </Panel>
  )
}
