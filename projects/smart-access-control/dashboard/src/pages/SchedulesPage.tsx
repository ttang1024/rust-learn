import { useState } from 'react'
import type { FormEvent } from 'react'

import { useApiMutation, useCanManage, usePagedList } from '../api/hooks'
import type { Schedule, ScheduleRule, Weekday } from '../api/types'
import { useAuth } from '../auth/context'
import { Button, ErrorMessage, PageHeader, Pager, Panel, Table, Td, TextField } from '../components/ui'
import { describeRule } from '../lib/format'
const WEEK: Weekday[] = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun']

export function SchedulesPage() {
  const canManage = useCanManage()
  const { page: schedules, pager } = usePagedList<Schedule>('schedules')

  return (
    <>
      <PageHeader title="Schedules" />
      {canManage && <CreateScheduleForm />}
      <ErrorMessage error={schedules.error} />
      <Table headers={['Name', 'Time zone', 'Windows', 'Effective']} empty={schedules.data?.items.length === 0}>
        {schedules.data?.items.map((schedule) => (
          <tr key={schedule.id}>
            <Td>{schedule.name}</Td>
            <Td className="text-slate-600">{schedule.timezone}</Td>
            <Td>
              <ul>
                {schedule.rules.map((rule, index) => (
                  <li key={index}>{describeRule(rule)}</li>
                ))}
              </ul>
            </Td>
            <Td className="text-slate-600">
              {schedule.effective_from ?? '…'} – {schedule.effective_until ?? '…'}
            </Td>
          </tr>
        ))}
      </Table>
      <Pager {...pager} />
    </>
  )
}

const newRule = (): ScheduleRule => ({ days: ['mon', 'tue', 'wed', 'thu', 'fri'], start: '09:00', end: '17:00' })

function CreateScheduleForm() {
  const { client } = useAuth()
  const [name, setName] = useState('')
  // Default to the administrator's own zone; the server validates the name.
  const [timezone, setTimezone] = useState(() => Intl.DateTimeFormat().resolvedOptions().timeZone)
  const [rules, setRules] = useState<ScheduleRule[]>([newRule()])
  const [effectiveFrom, setEffectiveFrom] = useState('')
  const [effectiveUntil, setEffectiveUntil] = useState('')
  const create = useApiMutation((body: unknown) => client.post<Schedule>('/schedules', body), [['schedules']])

  const updateRule = (index: number, change: Partial<ScheduleRule>) =>
    setRules(rules.map((rule, i) => (i === index ? { ...rule, ...change } : rule)))
  const toggleDay = (index: number, day: Weekday) => {
    const days = rules[index].days
    // Keep the week order regardless of click order.
    updateRule(index, { days: WEEK.filter((d) => (d === day ? !days.includes(d) : days.includes(d))) })
  }

  function onSubmit(event: FormEvent) {
    event.preventDefault()
    create.mutate(
      {
        name,
        timezone,
        rules,
        effective_from: effectiveFrom || null,
        effective_until: effectiveUntil || null,
      },
      {
        onSuccess: () => {
          setName('')
          setRules([newRule()])
        },
      },
    )
  }

  return (
    <Panel title="Create a schedule">
      <form onSubmit={onSubmit} className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-4">
          <TextField label="Name" required value={name} onChange={(e) => setName(e.target.value)} />
          <TextField label="Time zone (IANA)" required value={timezone} onChange={(e) => setTimezone(e.target.value)} />
          <TextField label="From (optional)" type="date" value={effectiveFrom} onChange={(e) => setEffectiveFrom(e.target.value)} />
          <TextField label="Until (optional)" type="date" value={effectiveUntil} onChange={(e) => setEffectiveUntil(e.target.value)} />
        </div>

        {rules.map((rule, index) => (
          <fieldset key={index} className="rounded border border-slate-200 p-3">
            <legend className="px-1 text-sm font-medium">Window {index + 1}</legend>
            <div className="flex flex-wrap items-end gap-3">
              <div className="flex gap-2 text-sm">
                {WEEK.map((day) => (
                  <label key={day} className="flex items-center gap-1">
                    <input type="checkbox" checked={rule.days.includes(day)} onChange={() => toggleDay(index, day)} />
                    {day}
                  </label>
                ))}
              </div>
              <TextField label="From" type="time" required value={rule.start} onChange={(e) => updateRule(index, { start: e.target.value })} />
              <TextField label="To" type="time" required value={rule.end} onChange={(e) => updateRule(index, { end: e.target.value })} />
              {rules.length > 1 && <Button onClick={() => setRules(rules.filter((_, i) => i !== index))}>Remove</Button>}
            </div>
            {rule.end <= rule.start && (
              <p className="mt-2 text-xs text-slate-500">Ends at or before it starts: the window runs past midnight.</p>
            )}
          </fieldset>
        ))}

        <div className="flex gap-2">
          <Button onClick={() => setRules([...rules, newRule()])}>Add window</Button>
          <Button type="submit" variant="primary" disabled={create.isPending}>
            Create schedule
          </Button>
        </div>
        <ErrorMessage error={create.error} />
      </form>
    </Panel>
  )
}
