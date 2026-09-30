import { describe, expect, it } from 'vitest'

import { FakeBackend } from '../test/fakeBackend'
import { ApiClient, ApiError } from './client'

function setup() {
  const backend = new FakeBackend()
  const client = new ApiClient({ fetch: backend.fetch })
  return { backend, client }
}

const PASSWORD = 'correct horse battery staple'

describe('ApiClient', () => {
  it('logs in with a cookie session and authenticates requests', async () => {
    const { client, backend } = setup()

    await client.login('ops', PASSWORD)

    // The refresh token is only in the (HttpOnly) cookie, never in JS land.
    expect(backend.cookie).toBe('refresh-1')
    expect(backend.requests).toHaveLength(0) // login itself is unauthenticated
    expect(client.getSessionState()).toBe('active')
    await expect(client.get('/auth/me')).resolves.toEqual({ id: 'admin-1', role: 'admin' })
  })

  it('never writes a token to web storage', async () => {
    const { client, backend } = setup()
    await client.login('ops', PASSWORD)
    backend.expireAccessTokens()
    await client.get('/health')

    const stored = [...Object.values(sessionStorage), ...Object.values(localStorage)].join()
    expect(stored).not.toContain('refresh-')
    expect(stored).not.toContain('access-')
  })

  it('rejects bad credentials with the server error', async () => {
    const { client, backend } = setup()

    const failure = client.login('ops', 'wrong')

    await expect(failure).rejects.toMatchObject({ status: 401, code: 'unauthorized' })
    expect(backend.cookie).toBeNull()
  })

  it('refreshes an expired access token once and retries', async () => {
    const { client, backend } = setup()
    await client.login('ops', PASSWORD)
    backend.expireAccessTokens()

    await expect(client.get('/health')).resolves.toMatchObject({ status: 'ok' })

    expect(backend.refreshCalls).toBe(1)
    expect(backend.cookie).toBe('refresh-2') // rotated by the server
  })

  it('shares one refresh between concurrent requests', async () => {
    const { client, backend } = setup()
    await client.login('ops', PASSWORD)
    backend.expireAccessTokens()

    // Three requests fail with 401 at the same time. With a refresh each,
    // the second and third would present an already-rotated cookie and the
    // (real or fake) server would reject them.
    const results = await Promise.all([
      client.get('/health'),
      client.get('/auth/me'),
      client.get('/health/ready'),
    ])

    expect(results).toHaveLength(3)
    expect(backend.refreshCalls).toBe(1)
  })

  it('resumes a session from the cookie after a reload', async () => {
    const backend = new FakeBackend()
    backend.seedSession()
    // A new client (page reload): no token in memory, state unknown.
    const reloaded = new ApiClient({ fetch: backend.fetch })
    expect(reloaded.getSessionState()).toBe('unknown')

    await expect(reloaded.get('/auth/me')).resolves.toMatchObject({ role: 'admin' })
    expect(reloaded.getSessionState()).toBe('active')
  })

  it('without a cookie a reload ends up signed out after one try', async () => {
    const { client, backend } = setup()

    await expect(client.get('/auth/me')).rejects.toMatchObject({ status: 401 })
    expect(client.getSessionState()).toBe('none')

    // Known to be signed out: no further pointless refresh attempts.
    await expect(client.get('/auth/me')).rejects.toMatchObject({ status: 401 })
    expect(backend.refreshCalls).toBe(1)
  })

  it('the live stream does not retry refreshing once signed out', async () => {
    const { client, backend } = setup()
    await expect(client.get('/auth/me')).rejects.toMatchObject({ status: 401 })
    expect(backend.refreshCalls).toBe(1)

    // The event stream asks for a token directly (bypassing normal requests).
    await expect(client.accessTokenForStream(true)).rejects.toMatchObject({ status: 401 })
    expect(backend.refreshCalls).toBe(1)
  })

  it('ends the session when the refresh is rejected', async () => {
    const { client, backend } = setup()
    await client.login('ops', PASSWORD)
    backend.expireAccessTokens()
    backend.validRefresh.clear() // e.g. revoked on the server
    let notified = 0
    client.subscribe(() => notified++)

    const failure = client.get('/health')

    await expect(failure).rejects.toBeInstanceOf(ApiError)
    await expect(failure).rejects.toMatchObject({ status: 401 })
    expect(client.hasSession()).toBe(false)
    expect(notified).toBe(1)
  })

  it('surfaces API errors and handles empty responses', async () => {
    const { client } = setup()
    await client.login('ops', PASSWORD)

    await expect(client.get('/nothing')).resolves.toBeUndefined()
    await expect(client.post('/conflict', {})).rejects.toMatchObject({
      status: 409,
      code: 'conflict',
      message: 'email is already in use',
    })
  })

  it('logs out on the server, which deletes the cookie', async () => {
    const { client, backend } = setup()
    await client.login('ops', PASSWORD)

    await client.logout()

    expect(backend.cookie).toBeNull()
    expect(backend.validRefresh.size).toBe(0)
    expect(client.getSessionState()).toBe('none')
    await expect(client.get('/auth/me')).rejects.toMatchObject({ status: 401 })
  })
})
