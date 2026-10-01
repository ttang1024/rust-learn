import { useState } from 'react'
import type { FormEvent } from 'react'

import { useApiMutation, useCanManage, usePagedList } from '../api/hooks'
import type { Controller } from '../api/types'
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

interface IssuedKey {
  controller: Controller
  key: string
}

export function ControllersPage() {
  const { client } = useAuth()
  const canManage = useCanManage()
  // Only ever in component state: a key is shown once and never refetched.
  const [issued, setIssued] = useState<IssuedKey | null>(null)
  const { page: controllers, pager } = usePagedList<Controller>('controllers')
  const rotate = useApiMutation(
    (id: string) => client.post<IssuedKey>(`/controllers/${id}/rotate-key`),
    [['controllers']],
  )

  return (
    <>
      <PageHeader title="Controllers" />
      <p className="mb-4 text-sm text-slate-600">
        Simulated door controllers. They authenticate with a secret key, send heartbeats, and go offline (with their
        doors) when they fall silent.
      </p>
      {issued && <KeyNotice issued={issued} onDismiss={() => setIssued(null)} />}
      {canManage && <RegisterForm onIssued={setIssued} />}
      <ErrorMessage error={controllers.error ?? rotate.error} />
      <Table headers={['Controller', 'Status', 'Last seen', 'Registered', '']} empty={controllers.data?.items.length === 0}>
        {controllers.data?.items.map((controller) => (
          <tr key={controller.id}>
            <Td className="font-mono">{controller.id}</Td>
            <Td>
              <StatusBadge status={controller.status} />
            </Td>
            <Td className="text-slate-500">{formatTime(controller.last_seen_at)}</Td>
            <Td className="text-slate-500">{formatTime(controller.created_at)}</Td>
            <Td className="text-right">
              {canManage && (
                <ConfirmButton
                  label="Rotate key"
                  confirmLabel="Replace key? The old one stops working."
                  onConfirm={() => rotate.mutate(controller.id, { onSuccess: setIssued })}
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

function KeyNotice({ issued, onDismiss }: { issued: IssuedKey; onDismiss: () => void }) {
  return (
    <div role="status" className="mb-6 rounded-lg border border-amber-300 bg-amber-50 p-4">
      <p className="font-medium">
        Key for <span className="font-mono">{issued.controller.id}</span>. Copy it now: it will not be shown again.
      </p>
      <code className="mt-2 block break-all rounded bg-white px-2 py-1 text-sm">{issued.key}</code>
      <p className="mt-2 text-sm text-slate-600">Send it as the <code>X-Controller-Key</code> header on /device requests.</p>
      <Button className="mt-3" onClick={onDismiss}>
        I have stored the key
      </Button>
    </div>
  )
}

function RegisterForm({ onIssued }: { onIssued: (issued: IssuedKey) => void }) {
  const { client } = useAuth()
  const [controllerId, setControllerId] = useState('')
  const register = useApiMutation(
    (controller_id: string) => client.post<IssuedKey>('/controllers', { controller_id }),
    [['controllers']],
  )

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    register.mutate(controllerId, {
      onSuccess: (issued) => {
        setControllerId('')
        onIssued(issued)
      },
    })
  }

  return (
    <Panel title="Register a controller">
      <form onSubmit={onSubmit} className="grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
        <TextField label="Controller id" required placeholder="ctrl-001" value={controllerId} onChange={(e) => setControllerId(e.target.value)} />
        <Button type="submit" variant="primary" disabled={register.isPending}>
          Register
        </Button>
      </form>
      <ErrorMessage className="mt-3" error={register.error} />
    </Panel>
  )
}
