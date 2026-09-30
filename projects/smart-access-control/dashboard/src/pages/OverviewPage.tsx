import { useQuery } from '@tanstack/react-query'

import type { Health, Readiness } from '../api/types'
import { useAuth } from '../auth/context'

export function OverviewPage() {
  const { state, client } = useAuth()
  const health = useQuery({
    queryKey: ['health'],
    queryFn: () => client.get<Health>('/health'),
  })
  const readiness = useQuery({
    queryKey: ['health', 'ready'],
    queryFn: () => client.get<Readiness>('/health/ready'),
    // 503 is a valid answer here ("not ready"), so don't retry it.
    retry: false,
  })

  const me = state.status === 'authenticated' ? state.me : null

  return (
    <section className="space-y-6">
      <h1 className="text-2xl font-semibold">Overview</h1>

      <dl className="grid gap-4 sm:grid-cols-3">
        <Card title="Signed in as">
          <span className="font-mono text-xs">{me?.id}</span>
          <span className="block text-sm capitalize text-slate-600">{me?.role}</span>
        </Card>
        <Card title="API">
          {health.isPending ? 'Checking…' : health.isError ? 'Unreachable' : `Up (v${health.data.version})`}
        </Card>
        <Card title="Database">
          {readiness.isPending ? 'Checking…' : readiness.isSuccess ? 'Ready' : 'Unavailable'}
        </Card>
      </dl>
    </section>
  )
}

function Card({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="rounded-lg border border-slate-200 bg-white p-4">
      <dt className="text-sm text-slate-500">{title}</dt>
      <dd className="mt-1 font-medium">{children}</dd>
    </div>
  )
}
