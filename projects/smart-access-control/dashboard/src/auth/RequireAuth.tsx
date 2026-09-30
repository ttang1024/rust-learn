import { Navigate, Outlet, useLocation } from 'react-router'

import { useAuth } from './context'

/**
 * Renders its child routes only for a logged-in administrator; everyone
 * else is sent to the login page (and back here afterwards).
 *
 * This is navigation, not security: the backend enforces every permission.
 */
export function RequireAuth() {
  const { state } = useAuth()
  const location = useLocation()

  if (state.status === 'loading') {
    return <p className="p-8 text-slate-500">Loading…</p>
  }
  if (state.status === 'anonymous') {
    return <Navigate to="/login" replace state={{ from: location.pathname }} />
  }
  return <Outlet />
}
