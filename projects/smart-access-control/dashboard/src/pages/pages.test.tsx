import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'

import type { User } from '../api/types'
import { FakeBackend, error, json } from '../test/fakeBackend'
import { renderSignedIn } from '../test/renderApp'

const page = <T,>(items: T[]) => ({ items, limit: 50, offset: 0 })

function user(id: string, name: string, status: User['status'] = 'active'): User {
  return {
    id,
    name,
    email: `${name.toLowerCase()}@example.com`,
    status,
    created_at: '2026-09-30T10:00:00Z',
    updated_at: '2026-09-30T10:00:00Z',
  }
}

/** A backend with a users collection that really stores what is POSTed. */
function usersBackend(initial: User[] = []) {
  const backend = new FakeBackend()
  const users = [...initial]
  backend.routes['GET /users'] = () => page(users)
  backend.routes['POST /users'] = (body) => {
    const { name, email } = body as { name: string; email: string }
    if (!email.includes('@')) {
      return error(422, 'validation_failed', 'invalid email: must look like name@example.com')
    }
    const created = { ...user(`u${users.length + 1}`, name), email }
    users.push(created)
    return json(201, created)
  }
  return backend
}

describe('users page', () => {
  it('registers a user and shows it in the refreshed list', async () => {
    const backend = usersBackend([user('u1', 'Alice')])
    renderSignedIn('/users', backend)
    expect(await screen.findByText('Alice')).toBeInTheDocument()

    const form = screen.getByRole('heading', { name: 'Register a user' }).closest('section')!
    const ui = userEvent.setup()
    await ui.type(within(form).getByLabelText('Name'), 'Bob')
    await ui.type(within(form).getByLabelText('Email'), 'bob@example.com')
    await ui.click(within(form).getByRole('button', { name: 'Register' }))

    expect(await screen.findByText('Bob')).toBeInTheDocument()
    expect(backend.requests).toContainEqual({
      method: 'POST',
      path: '/users',
      body: { name: 'Bob', email: 'bob@example.com' },
    })
    expect(within(form).getByLabelText('Name')).toHaveValue('')
  })

  it('shows the server validation message', async () => {
    // "bob@example" passes the browser's own `type=email` check (HTML allows
    // a domain without a dot) but not the server's rule, so the server's
    // message is what the user must see.
    const backend = usersBackend()
    backend.routes['POST /users'] = () =>
      error(422, 'validation_failed', 'invalid email: must look like name@example.com')
    renderSignedIn('/users', backend)
    const form = (await screen.findByRole('heading', { name: 'Register a user' })).closest('section')!

    const ui = userEvent.setup()
    await ui.type(within(form).getByLabelText('Name'), 'Bob')
    await ui.type(within(form).getByLabelText('Email'), 'bob@example')
    await ui.click(within(form).getByRole('button', { name: 'Register' }))

    expect(await within(form).findByRole('alert')).toHaveTextContent(
      'invalid email: must look like name@example.com',
    )
  })

  it('archiving needs a second, explicit click', async () => {
    const backend = usersBackend([user('u1', 'Alice')])
    backend.routes['DELETE /users/u1'] = () => new Response(null, { status: 204 })
    renderSignedIn('/users', backend)
    await screen.findByText('Alice')
    const ui = userEvent.setup()

    await ui.click(screen.getByRole('button', { name: 'Archive' }))
    expect(backend.requests.some((r) => r.method === 'DELETE')).toBe(false)
    await ui.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(screen.getByRole('button', { name: 'Archive' })).toBeInTheDocument()

    await ui.click(screen.getByRole('button', { name: 'Archive' }))
    await ui.click(screen.getByRole('button', { name: 'Archive?' }))
    await waitFor(() =>
      expect(backend.requests).toContainEqual({ method: 'DELETE', path: '/users/u1', body: undefined }),
    )
  })

  it('viewers see the data but no write controls', async () => {
    const backend = usersBackend([user('u1', 'Alice')])
    backend.role = 'viewer'
    renderSignedIn('/users', backend)

    expect(await screen.findByText('Alice')).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: 'Register a user' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Suspend' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Archive' })).not.toBeInTheDocument()
    expect(screen.getByText('read-only')).toBeInTheDocument()
  })
})

describe('controllers page', () => {
  it('shows a new key once, and only until dismissed', async () => {
    const backend = new FakeBackend()
    const controllers: unknown[] = []
    backend.routes['GET /controllers'] = () => page(controllers)
    backend.routes['POST /controllers'] = (body) => {
      const controller = {
        id: (body as { controller_id: string }).controller_id,
        status: 'offline',
        last_seen_at: null,
        created_at: '2026-09-30T10:00:00Z',
      }
      controllers.push(controller)
      return json(201, { controller, key: 'secret-key-123' })
    }
    renderSignedIn('/controllers', backend)
    const ui = userEvent.setup()

    await ui.type(await screen.findByLabelText('Controller id'), 'ctrl-001')
    await ui.click(screen.getByRole('button', { name: 'Register' }))

    const notice = await screen.findByRole('status')
    expect(notice).toHaveTextContent('secret-key-123')
    expect(notice).toHaveTextContent('it will not be shown again')
    // The table lists the controller but never the key.
    const table = screen.getByRole('table')
    expect(await within(table).findByText('ctrl-001')).toBeInTheDocument()
    expect(table).not.toHaveTextContent('secret-key-123')

    await ui.click(screen.getByRole('button', { name: 'I have stored the key' }))
    expect(screen.queryByText('secret-key-123')).not.toBeInTheDocument()
  })
})

describe('schedules page', () => {
  it('builds the rule payload from the editor', async () => {
    const backend = new FakeBackend()
    backend.routes['GET /schedules'] = () => page([])
    backend.routes['POST /schedules'] = (body) => json(201, { id: 's1', ...(body as object) })
    renderSignedIn('/schedules', backend)
    const ui = userEvent.setup()

    await ui.type(await screen.findByLabelText('Name'), 'Night shift')
    const zone = screen.getByLabelText('Time zone (IANA)')
    await ui.clear(zone)
    await ui.type(zone, 'Europe/London')
    // Window 1: weekdays by default; drop Friday, then add Sunday *before*
    // Saturday. The payload lists days in week order regardless.
    const window1 = screen.getByRole('group', { name: 'Window 1' })
    await ui.click(within(window1).getByLabelText('fri'))
    await ui.click(within(window1).getByLabelText('sun'))
    await ui.click(within(window1).getByLabelText('sat'))
    await ui.clear(within(window1).getByLabelText('From'))
    await ui.type(within(window1).getByLabelText('From'), '22:00')
    await ui.clear(within(window1).getByLabelText('To'))
    await ui.type(within(window1).getByLabelText('To'), '06:00')
    expect(within(window1).getByText(/runs past midnight/)).toBeInTheDocument()

    await ui.click(screen.getByRole('button', { name: 'Create schedule' }))

    await waitFor(() =>
      expect(backend.requests).toContainEqual({
        method: 'POST',
        path: '/schedules',
        body: {
          name: 'Night shift',
          timezone: 'Europe/London',
          rules: [{ days: ['mon', 'tue', 'wed', 'thu', 'sat', 'sun'], start: '22:00', end: '06:00' }],
          effective_from: null,
          effective_until: null,
        },
      }),
    )
  })
})

describe('permissions and groups', () => {
  it('grants "any time" access as a null schedule', async () => {
    const backend = new FakeBackend()
    backend.routes['GET /permissions'] = () => page([])
    backend.routes['GET /access-groups'] = () => page([{ id: 'g1', name: 'Staff', description: null }])
    backend.routes['GET /doors'] = () =>
      page([{ id: 'd1', name: 'Main', location: 'A', controller_id: 'c1', status: 'online', created_at: '' }])
    backend.routes['GET /schedules'] = () => page([])
    backend.routes['POST /permissions'] = (body) => json(201, { id: 'p1', created_at: '', ...(body as object) })
    renderSignedIn('/permissions', backend)
    const ui = userEvent.setup()

    await ui.selectOptions(await screen.findByLabelText('Group'), await screen.findByRole('option', { name: 'Staff' }))
    await ui.selectOptions(screen.getByLabelText('Door'), await screen.findByRole('option', { name: 'Main (A)' }))
    await ui.click(screen.getByRole('button', { name: 'Grant' }))

    await waitFor(() =>
      expect(backend.requests).toContainEqual({
        method: 'POST',
        path: '/permissions',
        body: { group_id: 'g1', door_id: 'd1', schedule_id: null },
      }),
    )
  })

  it('shows members by name and adds only non-members', async () => {
    const backend = new FakeBackend()
    const members = ['u1']
    backend.routes['GET /access-groups/g1'] = () => ({ id: 'g1', name: 'Staff', description: 'Everyone' })
    backend.routes['GET /access-groups/g1/members'] = () => ({ user_ids: [...members] })
    backend.routes['GET /users'] = () => page([user('u1', 'Alice'), user('u2', 'Bob'), user('u3', 'Carol', 'suspended')])
    backend.routes['POST /access-groups/g1/members'] = (body) => {
      members.push((body as { user_id: string }).user_id)
      return new Response(null, { status: 204 })
    }
    renderSignedIn('/access-groups/g1', backend)

    expect(await screen.findByText('Alice (alice@example.com)')).toBeInTheDocument()
    const select = screen.getByLabelText('User')
    const options = await within(select).findAllByRole('option')
    // Alice is already a member and Carol is suspended: only Bob is offered.
    expect(options.map((o) => o.textContent)).toEqual(['Choose a user…', 'Bob (bob@example.com)'])

    const ui = userEvent.setup()
    await ui.selectOptions(select, 'u2')
    await ui.click(screen.getByRole('button', { name: 'Add' }))
    expect(await screen.findByText('Bob (bob@example.com)', { selector: 'td' })).toBeInTheDocument()
  })
})
