import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render } from '@testing-library/react'
import { createMemoryRouter } from 'react-router'
import { RouterProvider } from 'react-router/dom'

import { ApiClient } from '../api/client'
import { SocketFactoryContext } from '../api/socket'
import { AuthProvider } from '../auth/AuthProvider'
import { routes } from '../router'
import { FakeBackend } from './fakeBackend'
import { FakeSocket } from './fakeSocket'

/** Renders the real route tree at `path`, backed by a fake API. */
/** Renders the app at `path` for an already signed-in administrator. */
export function renderSignedIn(path: string, backend = new FakeBackend()) {
  backend.seedSession()
  return renderApp(path, { backend })
}

export function renderApp(path: string, options: { backend?: FakeBackend } = {}) {
  const backend = options.backend ?? new FakeBackend()
  const client = new ApiClient({ fetch: backend.fetch })
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createMemoryRouter(routes, { initialEntries: [path] })

  render(
    <QueryClientProvider client={queryClient}>
      <AuthProvider client={client}>
        {/* WebSockets never reach the network in tests. */}
        <SocketFactoryContext.Provider value={FakeSocket.factory}>
          <RouterProvider router={router} />
        </SocketFactoryContext.Provider>
      </AuthProvider>
    </QueryClientProvider>,
  )
  return { backend, client, router, queryClient }
}
