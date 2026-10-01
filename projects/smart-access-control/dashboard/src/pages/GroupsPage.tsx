import { useState } from 'react'
import type { FormEvent } from 'react'
import { Link } from 'react-router'

import { useApiMutation, useCanManage, usePagedList } from '../api/hooks'
import type { AccessGroup } from '../api/types'
import { useAuth } from '../auth/context'
import { Button, ConfirmButton, ErrorMessage, PageHeader, Pager, Panel, Table, Td, TextField } from '../components/ui'

export function GroupsPage() {
  const { client } = useAuth()
  const canManage = useCanManage()
  const { page: groups, pager } = usePagedList<AccessGroup>('access-groups')
  const remove = useApiMutation((id: string) => client.delete(`/access-groups/${id}`), [['access-groups'], ['permissions']])

  return (
    <>
      <PageHeader title="Access groups" />
      {canManage && <CreateGroupForm />}
      <ErrorMessage error={groups.error ?? remove.error} />
      <Table headers={['Name', 'Description', '']} empty={groups.data?.items.length === 0}>
        {groups.data?.items.map((group) => (
          <tr key={group.id}>
            <Td>
              <Link to={`/access-groups/${group.id}`} className="font-medium underline">
                {group.name}
              </Link>
            </Td>
            <Td className="text-slate-600">{group.description ?? '—'}</Td>
            <Td className="text-right">
              {canManage && (
                <ConfirmButton
                  label="Delete"
                  confirmLabel="Delete group and its permissions?"
                  onConfirm={() => remove.mutate(group.id)}
                />
              )}
            </Td>
          </tr>
        ))}
      </Table>
      <Pager {...pager} />
    </>
  )
}

function CreateGroupForm() {
  const { client } = useAuth()
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const create = useApiMutation(
    (body: { name: string; description: string | null }) => client.post<AccessGroup>('/access-groups', body),
    [['access-groups']],
  )

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    create.mutate(
      { name, description: description.trim() === '' ? null : description },
      {
        onSuccess: () => {
          setName('')
          setDescription('')
        },
      },
    )
  }

  return (
    <Panel title="Create a group">
      <form onSubmit={onSubmit} className="grid gap-3 sm:grid-cols-[1fr_2fr_auto] sm:items-end">
        <TextField label="Name" required value={name} onChange={(e) => setName(e.target.value)} />
        <TextField label="Description (optional)" value={description} onChange={(e) => setDescription(e.target.value)} />
        <Button type="submit" variant="primary" disabled={create.isPending}>
          Create
        </Button>
      </form>
      <ErrorMessage className="mt-3" error={create.error} />
    </Panel>
  )
}
