import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'

import { FakeBackend } from '../test/fakeBackend'
import { renderApp } from '../test/renderApp'

async function signIn(password: string) {
  const user = userEvent.setup()
  await user.type(screen.getByLabelText('Username'), 'ops')
  await user.type(screen.getByLabelText('Password'), password)
  await user.click(screen.getByRole('button', { name: 'Sign in' }))
}

describe('login flow', () => {
  it('sends anonymous visitors to the login page', async () => {
    const { router } = renderApp('/')

    expect(await screen.findByRole('button', { name: 'Sign in' })).toBeInTheDocument()
    expect(router.state.location.pathname).toBe('/login')
  })

  it('signs in and shows the overview', async () => {
    const { router } = renderApp('/')
    await screen.findByLabelText('Username')

    await signIn('correct horse battery staple')

    expect(await screen.findByRole('heading', { name: 'Overview' })).toBeInTheDocument()
    expect(router.state.location.pathname).toBe('/')
    expect(await screen.findByText('Up (v0.1.0)')).toBeInTheDocument()
  })

  it('shows a generic error for wrong credentials', async () => {
    renderApp('/login')
    await screen.findByLabelText('Username')

    await signIn('wrong password')

    expect(await screen.findByRole('alert')).toHaveTextContent('Invalid username or password.')
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeEnabled()
  })

  it('resumes a stored session after a reload without asking again', async () => {
    // As after a page reload: no access token in memory, but the browser
    // still holds a valid session cookie.
    const backend = new FakeBackend()
    backend.seedSession()

    const { router } = renderApp('/', { backend })

    expect(await screen.findByRole('heading', { name: 'Overview' })).toBeInTheDocument()
    expect(screen.queryByLabelText('Password')).not.toBeInTheDocument()
    expect(router.state.location.pathname).toBe('/')
    expect(backend.refreshCalls).toBe(1)
  })

  it('a revoked session cookie falls back to the login page', async () => {
    const backend = new FakeBackend()
    backend.cookie = 'revoked-token' // not in validRefresh

    const { router } = renderApp('/', { backend })

    expect(await screen.findByRole('button', { name: 'Sign in' })).toBeInTheDocument()
    expect(router.state.location.pathname).toBe('/login')
  })

  it('signing out returns to the login page and forgets the session', async () => {
    const { backend, queryClient } = renderApp('/login')
    await screen.findByLabelText('Username')
    await signIn('correct horse battery staple')
    await screen.findByRole('heading', { name: 'Overview' })

    await userEvent.setup().click(screen.getByRole('button', { name: 'Sign out' }))

    expect(await screen.findByRole('button', { name: 'Sign in' })).toBeInTheDocument()
    expect(backend.cookie).toBeNull()
    await waitFor(() => expect(queryClient.getQueryCache().getAll()).toHaveLength(0))
  })
})
