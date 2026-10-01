import { useState } from 'react'
import type { ButtonHTMLAttributes, InputHTMLAttributes, ReactNode, SelectHTMLAttributes } from 'react'

import { ApiError } from '../api/client'

export function PageHeader({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
      <h1 className="text-2xl font-semibold">{title}</h1>
      <div className="flex gap-2">{children}</div>
    </div>
  )
}

type Variant = 'primary' | 'secondary' | 'danger'

const VARIANTS: Record<Variant, string> = {
  primary: 'bg-slate-900 text-white hover:bg-slate-700',
  secondary: 'border border-slate-300 bg-white text-slate-800 hover:bg-slate-100',
  danger: 'border border-red-300 bg-white text-red-700 hover:bg-red-50',
}

export function Button({
  variant = 'secondary',
  className = '',
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant }) {
  return (
    <button
      type="button"
      className={`rounded px-3 py-1.5 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50 ${VARIANTS[variant]} ${className}`}
      {...props}
    />
  )
}

/**
 * A destructive action that needs a second click to confirm. Avoids
 * `window.confirm`, whose modal blocks the whole page.
 */
export function ConfirmButton({
  label,
  confirmLabel = `${label}?`,
  onConfirm,
  disabled,
}: {
  label: string
  confirmLabel?: string
  onConfirm: () => void
  disabled?: boolean
}) {
  const [armed, setArmed] = useState(false)
  return armed ? (
    <span className="inline-flex gap-1">
      <Button
        variant="danger"
        disabled={disabled}
        onClick={() => {
          setArmed(false)
          onConfirm()
        }}
      >
        {confirmLabel}
      </Button>
      <Button onClick={() => setArmed(false)}>Cancel</Button>
    </span>
  ) : (
    <Button variant="danger" disabled={disabled} onClick={() => setArmed(true)}>
      {label}
    </Button>
  )
}

export function TextField({
  label,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { label: string }) {
  return (
    <label className="block text-sm font-medium">
      {label}
      <input className="mt-1 block w-full rounded border border-slate-300 px-2 py-1.5 font-normal" {...props} />
    </label>
  )
}

export function SelectField({
  label,
  children,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & { label: string }) {
  return (
    <label className="block text-sm font-medium">
      {label}
      <select className="mt-1 block w-full rounded border border-slate-300 bg-white px-2 py-1.5 font-normal" {...props}>
        {children}
      </select>
    </label>
  )
}

/** Shows an API error the way the backend phrased it (validation messages
 * are written for people). */
export function ErrorMessage({ error, className = '' }: { error: unknown; className?: string }) {
  if (!error) return null
  const message =
    error instanceof ApiError
      ? error.status === 403
        ? 'Your role does not allow this action.'
        : error.message
      : 'Something went wrong. Please try again.'
  return (
    <p role="alert" className={`rounded border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800 ${className}`}>
      {message}
    </p>
  )
}

const BADGE_COLORS: Record<string, string> = {
  active: 'bg-green-100 text-green-800',
  online: 'bg-green-100 text-green-800',
  granted: 'bg-green-100 text-green-800',
  suspended: 'bg-amber-100 text-amber-800',
  offline: 'bg-slate-200 text-slate-700',
  expired: 'bg-amber-100 text-amber-800',
  disabled: 'bg-red-100 text-red-800',
  revoked: 'bg-red-100 text-red-800',
  denied: 'bg-red-100 text-red-800',
  archived: 'bg-slate-200 text-slate-600',
}

export function StatusBadge({ status }: { status: string }) {
  return (
    <span className={`rounded px-2 py-0.5 text-xs font-medium ${BADGE_COLORS[status] ?? 'bg-slate-100 text-slate-700'}`}>
      {status.replaceAll('_', ' ')}
    </span>
  )
}

export function Table({ headers, children, empty }: { headers: string[]; children: ReactNode; empty?: boolean }) {
  return (
    <div className="overflow-x-auto rounded-lg border border-slate-200 bg-white">
      <table className="w-full text-left text-sm">
        <thead className="border-b border-slate-200 bg-slate-50 text-slate-600">
          <tr>
            {headers.map((header) => (
              <th key={header} className="px-3 py-2 font-medium">
                {header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100">
          {empty ? (
            <tr>
              <td colSpan={headers.length} className="px-3 py-6 text-center text-slate-500">
                Nothing here yet.
              </td>
            </tr>
          ) : (
            children
          )}
        </tbody>
      </table>
    </div>
  )
}

export function Td({ children, className = '' }: { children?: ReactNode; className?: string }) {
  return <td className={`px-3 py-2 align-middle ${className}`}>{children}</td>
}

export function Panel({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mb-6 rounded-lg border border-slate-200 bg-white p-4">
      <h2 className="mb-3 font-medium">{title}</h2>
      {children}
    </section>
  )
}

export function Pager({
  offset,
  limit,
  count,
  onChange,
}: {
  offset: number
  limit: number
  count: number
  onChange: (offset: number) => void
}) {
  if (offset === 0 && count < limit) return null
  return (
    <div className="mt-3 flex items-center gap-2 text-sm text-slate-600">
      <Button disabled={offset === 0} onClick={() => onChange(Math.max(0, offset - limit))}>
        Previous
      </Button>
      <span>
        {offset + 1}–{offset + count}
      </span>
      <Button disabled={count < limit} onClick={() => onChange(offset + limit)}>
        Next
      </Button>
    </div>
  )
}
