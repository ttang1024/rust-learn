import type { AccessEvent } from './types'

/** The same fields as `GET /events` filters (all optional). */
export interface EventFilter {
  door_id?: string
  user_id?: string
  card_id?: string
  decision?: 'granted' | 'denied'
  reason?: string
}

export type StreamStatus = 'connecting' | 'live' | 'reconnecting' | 'stopped'

export interface StreamSnapshot {
  status: StreamStatus
  /** Newest first, at most `maxEvents`. */
  events: AccessEvent[]
  /** Events the server dropped because this client fell behind. */
  missed: number
  /** The connection was interrupted at some point: events from the gap are
   * not replayed and may be missing here (see `GET /events`). */
  interrupted: boolean
  error: string | null
}

export type SocketFactory = (url: string) => WebSocket

export interface EventStreamOptions {
  url: string
  /** An access token; `forceRefresh` when the previous one expired. */
  token: (forceRefresh: boolean) => Promise<string>
  filter: EventFilter
  createSocket?: SocketFactory
  maxEvents?: number
}

/** Close codes sent by the backend (see its websocket module). */
const BAD_MESSAGE = 4400
const UNAUTHORIZED = 4401
const TOKEN_EXPIRED = 4409
const MAX_BACKOFF_MS = 30_000

/**
 * A self-healing subscription to `/api/v1/events/stream`.
 *
 * Shaped as an external store for React (`subscribe` / `getSnapshot`): the
 * snapshot object is replaced, never mutated, so React can compare it by
 * reference.
 */
export class EventStream {
  private snapshot: StreamSnapshot = {
    status: 'stopped',
    events: [],
    missed: 0,
    interrupted: false,
    error: null,
  }
  private listeners = new Set<() => void>()
  private socket: WebSocket | null = null
  private timer: ReturnType<typeof setTimeout> | null = null
  private attempts = 0
  private retriedAuth = false
  /** Bumped on every start/stop, so work from an older run can tell it is stale. */
  private generation = 0
  private readonly options: EventStreamOptions
  private readonly createSocket: SocketFactory
  private readonly maxEvents: number

  // A plain field instead of a `private options` parameter property: the
  // project allows only TypeScript syntax that erases to plain JavaScript.
  constructor(options: EventStreamOptions) {
    this.options = options
    this.createSocket = options.createSocket ?? ((url) => new WebSocket(url))
    this.maxEvents = options.maxEvents ?? 200
  }

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener)
    return () => {
      this.listeners.delete(listener)
    }
  }

  getSnapshot = (): StreamSnapshot => this.snapshot

  start(): void {
    this.generation++
    this.attempts = 0
    this.retriedAuth = false
    void this.connect(false, this.generation)
  }

  stop(): void {
    this.generation++
    if (this.timer !== null) clearTimeout(this.timer)
    this.timer = null
    this.socket?.close(1000)
    this.socket = null
    this.update({ status: 'stopped' })
  }

  private async connect(forceRefresh: boolean, generation: number): Promise<void> {
    this.update({ status: this.snapshot.status === 'live' || this.attempts > 0 ? 'reconnecting' : 'connecting' })
    let token: string
    try {
      token = await this.options.token(forceRefresh)
    } catch {
      if (generation === this.generation) this.fail('You are not signed in.')
      return
    }
    // stop() or a restart happened while we waited for the token.
    if (generation !== this.generation) return

    const socket = this.createSocket(this.options.url)
    this.socket = socket
    socket.onopen = () => {
      socket.send(JSON.stringify({ type: 'subscribe', token, filter: this.options.filter }))
    }
    socket.onmessage = (message: MessageEvent<string>) => {
      if (generation === this.generation) this.handle(JSON.parse(message.data))
    }
    socket.onclose = (event: CloseEvent) => {
      if (generation === this.generation) this.closed(event.code, generation)
    }
  }

  private handle(message: { type: string; event?: AccessEvent; missed?: number }): void {
    switch (message.type) {
      case 'subscribed':
        this.attempts = 0
        this.retriedAuth = false
        this.update({ status: 'live', error: null })
        break
      case 'access_event':
        if (message.event) {
          this.update({ events: [message.event, ...this.snapshot.events].slice(0, this.maxEvents) })
        }
        break
      case 'lagged':
        this.update({ missed: this.snapshot.missed + (message.missed ?? 0) })
        break
    }
  }

  private closed(code: number, generation: number): void {
    this.socket = null
    const wasLive = this.snapshot.status === 'live'
    if (code === TOKEN_EXPIRED) {
      // Expected every ~15 minutes: resubscribe at once with a new token.
      void this.connect(true, generation)
      return
    }
    if (code === UNAUTHORIZED && !this.retriedAuth) {
      this.retriedAuth = true
      void this.connect(true, generation)
      return
    }
    if (code === UNAUTHORIZED) return this.fail('Your session is no longer valid.')
    if (code === BAD_MESSAGE) return this.fail('The live feed rejected this filter.')

    // Server restart (1001), network loss (1006), ...: back off and retry.
    const delay = Math.min(MAX_BACKOFF_MS, 1000 * 2 ** this.attempts)
    this.attempts++
    this.update({ status: 'reconnecting', interrupted: this.snapshot.interrupted || wasLive })
    this.timer = setTimeout(() => {
      this.timer = null
      void this.connect(false, generation)
    }, delay)
  }

  private fail(error: string): void {
    this.generation++
    this.update({ status: 'stopped', error })
  }

  private update(change: Partial<StreamSnapshot>): void {
    this.snapshot = { ...this.snapshot, ...change }
    this.listeners.forEach((listener) => listener())
  }
}

/** `ws://` or `wss://` URL of the stream on the current origin. */
export function streamUrl(location: Location = window.location): string {
  const scheme = location.protocol === 'https:' ? 'wss' : 'ws'
  return `${scheme}://${location.host}/api/v1/events/stream`
}
