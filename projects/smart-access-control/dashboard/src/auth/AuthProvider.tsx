import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useMemo, useSyncExternalStore } from 'react'
import type { ReactNode } from 'react'

import type { ApiClient } from '../api/client'
import type { Me } from '../api/types'
import { AuthContext } from './context'
import type { AuthContextValue, AuthState } from './context'

const ME = ['auth', 'me'] as const

/**
 * Owns the session.
 *
 * Whether a session exists lives in the `ApiClient` (outside React), so it
 * is read with `useSyncExternalStore`. "Who am I" is an ordinary query that
 * runs only while a session exists; resuming a stored session after a page
 * reload is just that query running (the client refreshes the access token
 * first).
 */
export function AuthProvider({ client, children }: { client: ApiClient; children: ReactNode }) {
  const queryClient = useQueryClient()
  const hasSession = useSyncExternalStore(client.subscribe, client.hasSession)
  const me = useQuery({
    queryKey: ME,
    queryFn: () => client.get<Me>('/auth/me'),
    enabled: hasSession,
    retry: false,
    staleTime: Infinity,
  })

  const { isSuccess, isError, data } = me
  const state = useMemo<AuthState>(() => {
    if (!hasSession || isError) return { status: 'anonymous' }
    if (isSuccess) return { status: 'authenticated', me: data }
    return { status: 'loading' }
  }, [hasSession, isSuccess, isError, data])

  const value = useMemo<AuthContextValue>(
    () => ({
      state,
      client,
      async login(username, password) {
        await client.login(username, password)
        // Load the profile before resolving, so the caller can navigate
        // straight into the app without a "loading" flash.
        await queryClient.fetchQuery({ queryKey: ME, queryFn: () => client.get<Me>('/auth/me') })
      },
      async logout() {
        await client.logout()
        // Drop every cached response, so the next person to sign in on this
        // tab never sees the previous administrator's data.
        queryClient.clear()
      },
    }),
    [state, client, queryClient],
  )

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}
