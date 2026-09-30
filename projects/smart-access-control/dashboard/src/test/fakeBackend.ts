/**
 * A minimal stand-in for the backend's auth behaviour, including refresh
 * token rotation with reuse detection: presenting an old refresh token
 * fails, exactly like the real server.
 */
type Handler = (body: unknown, url: URL) => Response | unknown

export class FakeBackend {
  role: 'admin' | 'viewer' = 'admin'
  /** Extra endpoints for a test, keyed `"METHOD /path"` (no query string). */
  routes: Record<string, Handler> = {}
  /** Every authenticated request, for asserting what a page sent. */
  requests: { method: string; path: string; body: unknown }[] = []
  validAccess = new Set<string>()
  validRefresh = new Set<string>()
  /** The browser's `sac_refresh` cookie, as the real server would set it. */
  cookie: string | null = null
  refreshCalls = 0
  private counter = 0
  password = 'correct horse battery staple'

  /** A `fetch` implementation routed to this fake. */
  fetch = async (input: string, init?: RequestInit): Promise<Response> => {
    const path = input.replace('/api/v1', '')
    const body = init?.body ? JSON.parse(init.body as string) : undefined
    const auth = (init?.headers as Record<string, string> | undefined)?.Authorization
    const token = auth?.replace('Bearer ', '')

    if (path === '/auth/login') {
      if (body.password !== this.password) return error(401, 'unauthorized', 'invalid credentials')
      return json(200, this.issue(body.session === 'cookie'))
    }
    if (path === '/auth/refresh') {
      this.refreshCalls++
      const fromBody = body?.refresh_token as string | undefined
      const token = fromBody ?? this.cookie
      if (token === null || !this.validRefresh.delete(token)) {
        return error(401, 'unauthorized', 'invalid credentials')
      }
      return json(200, this.issue(fromBody === undefined))
    }
    if (path === '/auth/logout') {
      const token = (body?.refresh_token as string | undefined) ?? this.cookie
      if (token) this.validRefresh.delete(token)
      this.cookie = null
      return new Response(null, { status: 204 })
    }
    if (token === undefined || !this.validAccess.has(token)) {
      return error(401, 'unauthorized', 'invalid credentials')
    }
    const method = init?.method ?? 'GET'
    const url = new URL(path, 'http://fake')
    this.requests.push({ method, path, body })
    const handler = this.routes[`${method} ${url.pathname}`]
    if (handler) {
      const result = handler(body, url)
      return result instanceof Response ? result : json(200, result)
    }
    if (path === '/auth/me') return json(200, { id: 'admin-1', role: this.role })
    if (path === '/health') return json(200, { status: 'ok', version: '0.1.0' })
    if (path === '/health/ready') return json(200, { status: 'ready' })
    if (path === '/nothing') return new Response(null, { status: 204 })
    if (path === '/conflict') return error(409, 'conflict', 'email is already in use')
    return error(404, 'not_found', 'not found')
  }

  /** Makes every current access token invalid, as if it had expired. */
  expireAccessTokens(): void {
    this.validAccess.clear()
  }

  /** Starts an already signed-in browser session (a valid cookie). */
  seedSession(): void {
    this.cookie = 'seeded-refresh-token'
    this.validRefresh.add(this.cookie)
  }

  private issue(cookieMode: boolean) {
    this.counter++
    const access = `access-${this.counter}`
    const refresh = `refresh-${this.counter}`
    this.validAccess.add(access)
    this.validRefresh.add(refresh)
    if (cookieMode) this.cookie = refresh
    return {
      access_token: access,
      token_type: 'Bearer',
      expires_in: 900,
      ...(cookieMode ? {} : { refresh_token: refresh }),
      refresh_expires_in: 604800,
    }
  }
}


export function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

export function error(status: number, code: string, message: string): Response {
  return json(status, { error: { code, message } })
}
