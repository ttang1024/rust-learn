import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { FakeSocket } from '../test/fakeSocket'
import { EventStream } from './eventStream'
import type { AccessEvent } from './types'

function event(id: string): AccessEvent {
  return {
    id,
    decision: 'denied',
    reason: 'unknown_card',
    card_number: 'CARD-1',
    card_id: null,
    user_id: null,
    door_id: 'd1',
    occurred_at: '2026-09-30T10:00:00Z',
  }
}

/** Lets pending promises (the token lookup) settle. */
const flush = () => vi.advanceTimersByTimeAsync(0)

function setup(maxEvents?: number) {
  const token = vi.fn(async (force: boolean) => (force ? 'fresh-token' : 'token'))
  const stream = new EventStream({
    url: 'ws://test/api/v1/events/stream',
    token,
    filter: { decision: 'denied' },
    createSocket: FakeSocket.factory,
    maxEvents,
  })
  return { stream, token }
}

/** Starts the stream and completes the subscribe handshake. */
async function goLive(stream: EventStream) {
  stream.start()
  await flush()
  const socket = FakeSocket.latest()
  socket.open()
  socket.receive({ type: 'subscribed', expires_at: '2026-09-30T10:15:00Z' })
  return socket
}

beforeEach(() => {
  vi.useFakeTimers()
  FakeSocket.reset()
})
afterEach(() => {
  vi.useRealTimers()
})

describe('EventStream', () => {
  it('subscribes with the token and filter, then goes live', async () => {
    const { stream } = setup()
    stream.start()
    expect(stream.getSnapshot().status).toBe('connecting')
    await flush()

    const socket = FakeSocket.latest()
    socket.open()
    expect(socket.sent).toEqual([{ type: 'subscribe', token: 'token', filter: { decision: 'denied' } }])

    socket.receive({ type: 'subscribed', expires_at: '2026-09-30T10:15:00Z' })
    expect(stream.getSnapshot().status).toBe('live')
  })

  it('keeps the newest events first, up to the limit', async () => {
    const { stream } = setup(2)
    const socket = await goLive(stream)

    for (const id of ['e1', 'e2', 'e3']) socket.receive({ type: 'access_event', event: event(id) })

    expect(stream.getSnapshot().events.map((e) => e.id)).toEqual(['e3', 'e2'])
  })

  it('counts events the server skipped', async () => {
    const { stream } = setup()
    const socket = await goLive(stream)

    socket.receive({ type: 'lagged', missed: 5 })
    socket.receive({ type: 'lagged', missed: 2 })

    expect(stream.getSnapshot().missed).toBe(7)
  })

  it('resubscribes at once with a fresh token when the token expires', async () => {
    const { stream, token } = setup()
    const first = await goLive(stream)

    first.serverClose(4409)
    await flush()

    expect(token).toHaveBeenLastCalledWith(true)
    const second = FakeSocket.latest()
    expect(second).not.toBe(first)
    second.open()
    expect(second.sent[0]).toMatchObject({ token: 'fresh-token' })
    // Token rotation is routine, not an interruption of the feed.
    expect(stream.getSnapshot().interrupted).toBe(false)
  })

  it('reconnects with exponential backoff after a dropped connection', async () => {
    const { stream } = setup()
    await goLive(stream)

    FakeSocket.latest().serverClose(1006)
    expect(stream.getSnapshot()).toMatchObject({ status: 'reconnecting', interrupted: true })
    expect(FakeSocket.instances).toHaveLength(1)

    await vi.advanceTimersByTimeAsync(999)
    expect(FakeSocket.instances).toHaveLength(1)
    await vi.advanceTimersByTimeAsync(1)
    expect(FakeSocket.instances).toHaveLength(2) // after 1 s

    FakeSocket.latest().serverClose(1006)
    await vi.advanceTimersByTimeAsync(1999)
    expect(FakeSocket.instances).toHaveLength(2)
    await vi.advanceTimersByTimeAsync(1)
    expect(FakeSocket.instances).toHaveLength(3) // then after 2 s
  })

  it('stops for good on a rejected filter', async () => {
    const { stream } = setup()
    stream.start()
    await flush()

    FakeSocket.latest().serverClose(4400)
    await vi.advanceTimersByTimeAsync(60_000)

    expect(stream.getSnapshot()).toMatchObject({ status: 'stopped', error: 'The live feed rejected this filter.' })
    expect(FakeSocket.instances).toHaveLength(1)
  })

  it('retries a rejected token once, then gives up', async () => {
    const { stream, token } = setup()
    stream.start()
    await flush()

    FakeSocket.latest().serverClose(4401)
    await flush()
    expect(token).toHaveBeenLastCalledWith(true)
    FakeSocket.latest().serverClose(4401)
    await vi.advanceTimersByTimeAsync(60_000)

    expect(stream.getSnapshot()).toMatchObject({ status: 'stopped', error: 'Your session is no longer valid.' })
    expect(FakeSocket.instances).toHaveLength(2)
  })

  it('stop() closes the socket and cancels any retry', async () => {
    const { stream } = setup()
    await goLive(stream)
    FakeSocket.latest().serverClose(1006)

    stream.stop()
    await vi.advanceTimersByTimeAsync(60_000)

    expect(FakeSocket.instances).toHaveLength(1)
    expect(stream.getSnapshot().status).toBe('stopped')
  })

  it('a stop/start while fetching a token opens only one socket', async () => {
    // React StrictMode runs effects twice in development: start, stop, start.
    const { stream } = setup()
    stream.start()
    stream.stop()
    stream.start()
    await flush()

    expect(FakeSocket.instances).toHaveLength(1)
  })
})
