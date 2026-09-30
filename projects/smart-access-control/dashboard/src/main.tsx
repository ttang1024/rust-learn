import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { createBrowserRouter } from 'react-router'
import { RouterProvider } from 'react-router/dom'

import { ApiClient } from './api/client'
import { AuthProvider } from './auth/AuthProvider'
import './index.css'
import { routes } from './router'

const client = new ApiClient()
const queryClient = new QueryClient({
  defaultOptions: { queries: { staleTime: 10_000 } },
})
const router = createBrowserRouter(routes)

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <AuthProvider client={client}>
        <RouterProvider router={router} />
      </AuthProvider>
    </QueryClientProvider>
  </StrictMode>,
)
