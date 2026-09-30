import type { ErrorBody, TokenResponse } from './types'

/** A failed API call, carrying the backend's error code and message. */
export class ApiError extends Error {
  readonly status: number
  readonly code: string

  constructor(status: number, code: string, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.code = code
  }
}

type Fetch = (input: string, init?: RequestInit) => Promise<Response>

export interface ApiClientOptions {
  baseUrl?: string
  fetch?: Fetch
}

/**
 * - `unknown`: just started; a session cookie may or may not exist.
 * - `active`: signed in (an access token is held in memory).
 * - `none`: signed out, or the server rejected the session.
 */
export type SessionState = 'unknown' | 'active' | 'none'

/**
 * Talks to the backend API.
 *
 * The **access token** lives in memory only. The **refresh token** never
 * reaches JavaScript at all: the login asks for cookie mode, so the server
 * keeps it in an `HttpOnly; SameSite=Strict` cookie that the browser sends
 * to the auth endpoints by itself. An XSS bug can therefore not steal a
 * long-lived session. After a reload the client simply tries a refresh.
 *
 * A request that gets 401 triggers one refresh and is retried once.
 *
 * Refreshing is single-flight: if several requests fail at once, they all
 * wait for the *same* refresh call. That matters here because the backend
 * rotates refresh tokens and treats a reused one as theft, revoking every
 * session; two parallel refreshes with the same token would log us out.
 */
export class ApiClient {
  private readonly baseUrl: string
  private readonly fetch: Fetch
  private accessToken: string | null = null
  private session: SessionState = 'unknown'
  private refreshing: Promise<boolean> | null = null
  private listeners = new Set<() => void>()

  constructor(options: ApiClientOptions = {}) {
    this.baseUrl = options.baseUrl ?? '/api/v1'
    // Wrapped so `fetch` is always called with the right `this`.
    this.fetch = options.fetch ?? ((input, init) => fetch(input, init))
  }

  /** Whether a session may exist (not known to be signed out). */
  hasSession = (): boolean => this.session !== 'none'

  getSessionState = (): SessionState => this.session

  /**
   * Notifies `listener` whenever the session starts or ends (login, logout,
   * rejected refresh). Returns an unsubscribe function. Shaped for React's
   * `useSyncExternalStore(client.subscribe, client.hasSession)`.
   */
  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener)
    return () => {
      this.listeners.delete(listener)
    }
  }

  async login(username: string, password: string): Promise<void> {
    const response = await this.send(
      'POST',
      '/auth/login',
      { username, password, session: 'cookie' },
      null,
    )
    this.acceptTokens((await parse(response)) as TokenResponse)
  }

  /** Ends the session on the server (which deletes the cookie) and forgets
   * the access token. */
  async logout(): Promise<void> {
    this.clear()
    await this.send('POST', '/auth/logout', {}, null).catch(() => undefined)
  }

  get<T>(path: string): Promise<T> {
    return this.request<T>('GET', path)
  }

  post<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('POST', path, body)
  }

  patch<T>(path: string, body: unknown): Promise<T> {
    return this.request<T>('PATCH', path, body)
  }

  delete<T = void>(path: string): Promise<T> {
    return this.request<T>('DELETE', path)
  }

  /**
   * An access token for the WebSocket subscribe message. With `forceRefresh`
   * (the stream closed because its token expired) a new one is fetched even
   * if a token is held.
   */
  async accessTokenForStream(forceRefresh = false): Promise<string> {
    if ((forceRefresh || this.accessToken === null) && !(await this.refresh())) {
      throw new ApiError(401, 'unauthorized', 'not logged in')
    }
    return this.accessToken as string
  }

  async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    if (this.accessToken === null && this.hasSession()) {
      await this.refresh()
    }
    let response = await this.send(method, path, body, this.accessToken)
    if (response.status === 401 && this.hasSession()) {
      if (await this.refresh()) {
        response = await this.send(method, path, body, this.accessToken)
      }
    }
    return (await parse(response)) as T
  }

  private refresh(): Promise<boolean> {
    // Everyone who needs a refresh right now shares this one promise.
    this.refreshing ??= this.doRefresh().finally(() => {
      this.refreshing = null
    })
    return this.refreshing
  }

  private async doRefresh(): Promise<boolean> {
    if (this.session === 'none') return false
    try {
      // `{}`: the token is in the cookie. The body must still be JSON, which
      // is part of the backend's cross-site request protection.
      const response = await this.send('POST', '/auth/refresh', {}, null)
      this.acceptTokens((await parse(response)) as TokenResponse)
      return true
    } catch {
      this.clear()
      return false
    }
  }

  private acceptTokens(tokens: TokenResponse): void {
    this.accessToken = tokens.access_token
    this.setSession('active')
  }

  private clear(): void {
    this.accessToken = null
    this.setSession('none')
  }

  private setSession(session: SessionState): void {
    if (this.session === session) return
    this.session = session
    this.notify()
  }

  private notify(): void {
    this.listeners.forEach((listener) => listener())
  }

  private send(
    method: string,
    path: string,
    body: unknown,
    accessToken: string | null,
  ): Promise<Response> {
    const headers: Record<string, string> = {}
    if (body !== undefined) headers['Content-Type'] = 'application/json'
    if (accessToken !== null) headers.Authorization = `Bearer ${accessToken}`
    return this.fetch(`${this.baseUrl}${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
      // Send the session cookie to our own origin only (the fetch default,
      // stated here because the design depends on it).
      credentials: 'same-origin',
    })
  }
}

/** The JSON body of a successful response, or an `ApiError`. */
async function parse(response: Response): Promise<unknown> {
  if (response.ok) {
    return response.status === 204 ? undefined : response.json()
  }
  const body = (await response.json().catch(() => null)) as ErrorBody | null
  throw new ApiError(
    response.status,
    body?.error.code ?? 'http_error',
    body?.error.message ?? `request failed with status ${response.status}`,
  )
}
