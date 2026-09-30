import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import type { FormEvent } from 'react'
import { Link, useSearchParams } from 'react-router'

import { PAGE_SIZE, byId, useApiMutation, useCanManage, useOptions } from '../api/hooks'
import type { Card, Page, User } from '../api/types'
import { useAuth } from '../auth/context'
import {
  Button,
  ConfirmButton,
  ErrorMessage,
  PageHeader,
  Pager,
  Panel,
  SelectField,
  StatusBadge,
  Table,
  Td,
  TextField,
} from '../components/ui'
import { formatTime } from '../lib/format'


export function CardsPage() {
  const { client } = useAuth()
  const canManage = useCanManage()
  const [params] = useSearchParams()
  const userFilter = params.get('user')
  const [offset, setOffset] = useState(0)
  const users = useOptions<User>('users')
  const userName = byId(users.data?.items, (user) => user.name)

  const cards = useQuery({
    queryKey: ['cards', userFilter, offset],
    queryFn: () =>
      client.get<Page<Card>>(
        userFilter ? `/cards?user_id=${userFilter}` : `/cards?limit=${PAGE_SIZE}&offset=${offset}`,
      ),
  })
  const change = useApiMutation(
    ({ id, action }: { id: string; action: 'suspend' | 'reactivate' | 'revoke' }) =>
      action === 'revoke'
        ? client.post(`/cards/${id}/revoke`)
        : client.patch(`/cards/${id}`, { status: action === 'suspend' ? 'suspended' : 'active' }),
    [['cards']],
  )

  return (
    <>
      <PageHeader title={userFilter ? `Cards of ${userName(userFilter)}` : 'Cards'}>
        {userFilter && (
          <Link to="/cards" className="text-sm text-slate-600 underline">
            Show all cards
          </Link>
        )}
      </PageHeader>
      {canManage && <IssueCardForm users={users.data?.items ?? []} defaultUser={userFilter} />}
      <ErrorMessage error={cards.error ?? change.error} />
      <Table headers={['Number', 'Holder', 'Status', 'Issued', 'Expires', '']} empty={cards.data?.items.length === 0}>
        {cards.data?.items.map((card) => (
          <tr key={card.id}>
            <Td className="font-mono">{card.card_number}</Td>
            <Td>{userName(card.user_id)}</Td>
            <Td>
              <StatusBadge status={card.effective_status} />
            </Td>
            <Td className="text-slate-500">{formatTime(card.issued_at)}</Td>
            <Td className="text-slate-500">{formatTime(card.expires_at)}</Td>
            <Td className="space-x-2 text-right whitespace-nowrap">
              {canManage && card.status === 'active' && (
                <Button onClick={() => change.mutate({ id: card.id, action: 'suspend' })}>Suspend</Button>
              )}
              {canManage && card.status === 'suspended' && (
                <Button onClick={() => change.mutate({ id: card.id, action: 'reactivate' })}>Reactivate</Button>
              )}
              {canManage && card.status !== 'revoked' && (
                <ConfirmButton label="Revoke" confirmLabel="Revoke permanently?" onConfirm={() => change.mutate({ id: card.id, action: 'revoke' })} />
              )}
            </Td>
          </tr>
        ))}
      </Table>
      {!userFilter && <Pager offset={offset} limit={PAGE_SIZE} count={cards.data?.items.length ?? 0} onChange={setOffset} />}
    </>
  )
}

function IssueCardForm({ users, defaultUser }: { users: User[]; defaultUser: string | null }) {
  const { client } = useAuth()
  const [userId, setUserId] = useState(defaultUser ?? '')
  const [cardNumber, setCardNumber] = useState('')
  const [expiresAt, setExpiresAt] = useState('')
  const issue = useApiMutation(
    (body: { user_id: string; card_number: string; expires_at: string | null }) => client.post<Card>('/cards', body),
    [['cards']],
  )

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    issue.mutate(
      {
        user_id: userId,
        card_number: cardNumber,
        // <input type="datetime-local"> is local time; the API wants UTC RFC 3339.
        expires_at: expiresAt ? new Date(expiresAt).toISOString() : null,
      },
      { onSuccess: () => setCardNumber('') },
    )
  }

  return (
    <Panel title="Issue a card">
      <form onSubmit={onSubmit} className="grid gap-3 sm:grid-cols-[1fr_1fr_1fr_auto] sm:items-end">
        <SelectField label="Holder" required value={userId} onChange={(e) => setUserId(e.target.value)}>
          <option value="">Choose a user…</option>
          {users
            .filter((user) => user.status === 'active')
            .map((user) => (
              <option key={user.id} value={user.id}>
                {user.name} ({user.email})
              </option>
            ))}
        </SelectField>
        <TextField label="Card number" required placeholder="CARD-10001" value={cardNumber} onChange={(e) => setCardNumber(e.target.value)} />
        <TextField label="Expires (optional)" type="datetime-local" value={expiresAt} onChange={(e) => setExpiresAt(e.target.value)} />
        <Button type="submit" variant="primary" disabled={issue.isPending}>
          Issue
        </Button>
      </form>
      <div className="mt-3">
        <ErrorMessage error={issue.error} />
      </div>
    </Panel>
  )
}
