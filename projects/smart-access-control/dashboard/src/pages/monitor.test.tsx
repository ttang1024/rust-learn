import { act, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it } from 'vitest'

import { FakeBackend, error, json } from '../test/fakeBackend'
import { FakeSocket } from '../test/fakeSocket'
import { renderSignedIn } from '../test/renderApp'

const page = <T,>(items: T[]) => ({ items, limit: 100, offset: 0 })
const door = (id: string, name: string, controller: string) => ({
  id,
  name,
  location: 'Lab',
  controller_id: controller,
  status: 'online',
  created_at: '2026-09-30T10:00:00Z',
})

function monitorBackend() {
  const backend = new FakeBackend()
  backend.routes['GET /doors'] = () => page([door('d1', 'Main Entrance', 'ctrl-1'), door('d2', 'Back Door', 'ctrl-2')])
  backend.routes['GET /users'] = () => page([])
  backend.routes['GET /simulator/controllers'] = () => error(404, 'not_found', 'not found')
  return backend
}

/** Waits for the page's socket, then completes the handshake. */
async function liveSocket() {
  await waitFor(() => expect(FakeSocket.instances.length).toBeGreaterThan(0))
  const socket = FakeSocket.latest()
  act(() => {
    socket.open()
    socket.receive({ type: 'subscribed', expires_at: '2026-09-30T10:15:00Z' })
  })
  return socket
}

beforeEach(() => FakeSocket.reset())

describe('live monitor', () => {
  it('shows events as they arrive, with door names', async () => {
    renderSignedIn('/monitor', monitorBackend())
    const socket = await liveSocket()
    expect(await screen.findByRole('status')).toHaveTextContent('Live')
    // Door names come from a separate query; wait until they are loaded.
    expect(await screen.findByRole('option', { name: 'Main Entrance' })).toBeInTheDocument()

    act(() =>
      socket.receive({
        type: 'access_event',
        event: {
          id: 'e1',
          decision: 'denied',
          reason: 'unknown_card',
          card_number: 'CARD-404',
          card_id: null,
          user_id: null,
          door_id: 'd1',
          occurred_at: '2026-09-30T10:00:00Z',
        },
      }),
    )

    const row = (await screen.findByText('CARD-404')).closest('tr')!
    expect(row).toHaveTextContent('Main Entrance')
    expect(row).toHaveTextContent('unknown card')
  })

  it('warns about skipped events and points to the history', async () => {
    renderSignedIn('/monitor', monitorBackend())
    const socket = await liveSocket()

    act(() => socket.receive({ type: 'lagged', missed: 12 }))

    expect(await screen.findByText(/12 events were skipped/)).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'event history' })).toHaveAttribute('href', '/events')
  })

  it('changing the filter resubscribes with the new filter', async () => {
    renderSignedIn('/monitor', monitorBackend())
    const first = await liveSocket()
    const ui = userEvent.setup()

    await ui.selectOptions(screen.getByLabelText('Door'), await screen.findByRole('option', { name: 'Back Door' }))

    await waitFor(() => expect(FakeSocket.instances.length).toBeGreaterThan(1))
    const second = FakeSocket.latest()
    act(() => second.open())
    expect(second.sent[0]).toMatchObject({ type: 'subscribe', filter: { door_id: 'd2' } })
    expect(first.closedByClient).toBe(true)
  })
})

describe('simulator panel', () => {
  it('explains when the simulator is disabled', async () => {
    renderSignedIn('/monitor', monitorBackend())
    expect(await screen.findByText(/simulator is disabled on this server/i)).toBeInTheDocument()
  })

  it('swipes a card at one of the controller\'s own doors and shows the result', async () => {
    const backend = monitorBackend()
    backend.routes['GET /simulator/controllers'] = () => [
      { controller_id: 'ctrl-1', simulated: true, network: 'up', heartbeats_sent: 3, last_heartbeat_at: null, last_error: null },
    ]
    backend.routes['POST /simulator/controllers/ctrl-1/swipe'] = () =>
      json(200, {
        id: 'e1',
        decision: 'granted',
        reason: null,
        card_number: 'CARD-1',
        card_id: 'c1',
        user_id: 'u1',
        door_id: 'd1',
        occurred_at: '2026-09-30T10:00:00Z',
      })
    renderSignedIn('/monitor', backend)
    const ui = userEvent.setup()

    const doorSelect = await screen.findByLabelText('At door')
    // Only ctrl-1's doors are offered.
    const options = within(doorSelect).getAllByRole('option').map((o) => o.textContent)
    expect(options).toEqual(['Choose a door…', 'Main Entrance'])

    await ui.selectOptions(doorSelect, 'd1')
    await ui.type(screen.getByLabelText('Card number'), 'CARD-1')
    await ui.click(screen.getByRole('button', { name: 'Swipe card' }))

    expect(await screen.findByText(/Result:/)).toHaveTextContent('granted')
    expect(backend.requests).toContainEqual({
      method: 'POST',
      path: '/simulator/controllers/ctrl-1/swipe',
      body: { card_number: 'CARD-1', door_id: 'd1' },
    })
  })

  it('shows the offline error when the simulated network is down', async () => {
    const backend = monitorBackend()
    backend.routes['GET /simulator/controllers'] = () => [
      { controller_id: 'ctrl-1', simulated: true, network: 'down', heartbeats_sent: 3, last_heartbeat_at: null, last_error: null },
    ]
    backend.routes['POST /simulator/controllers/ctrl-1/swipe'] = () =>
      error(503, 'controller_offline', 'simulated controller ctrl-1 has no network connection; access denied locally')
    renderSignedIn('/monitor', backend)
    const ui = userEvent.setup()

    await ui.selectOptions(await screen.findByLabelText('At door'), 'd1')
    await ui.type(screen.getByLabelText('Card number'), 'CARD-1')
    await ui.click(screen.getByRole('button', { name: 'Swipe card' }))

    expect(await screen.findByText(/access denied locally/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Reconnect' })).toBeInTheDocument()
  })
})

describe('event history', () => {
  it('sends the URL filters to the API as RFC 3339 and paginates', async () => {
    const backend = monitorBackend()
    backend.routes['GET /events'] = () => page([])
    renderSignedIn('/events?decision=denied&door_id=d1&from=2026-09-30T08:00', backend)

    await waitFor(() => expect(backend.requests.some((r) => r.path.startsWith('/events?'))).toBe(true))
    const request = backend.requests.find((r) => r.path.startsWith('/events?'))!
    const query = new URL(request.path, 'http://x').searchParams
    expect(query.get('decision')).toBe('denied')
    expect(query.get('door_id')).toBe('d1')
    expect(query.get('from')).toBe(new Date('2026-09-30T08:00').toISOString())
    expect(query.get('limit')).toBe('50')
    expect(screen.getByLabelText('Decision')).toHaveValue('denied')
  })
})
