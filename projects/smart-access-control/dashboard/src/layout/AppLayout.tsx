import { NavLink, Outlet } from 'react-router'

import { useAuth } from '../auth/context'

const NAV = [
  { to: '/', label: 'Overview' },
  { to: '/monitor', label: 'Monitor' },
  { to: '/events', label: 'Events' },
  { to: '/users', label: 'Users' },
  { to: '/cards', label: 'Cards' },
  { to: '/doors', label: 'Doors' },
  { to: '/access-groups', label: 'Groups' },
  { to: '/schedules', label: 'Schedules' },
  { to: '/permissions', label: 'Permissions' },
  { to: '/controllers', label: 'Controllers' },
]

export function AppLayout() {
  const { state, logout } = useAuth()
  const role = state.status === 'authenticated' ? state.me.role : null

  return (
    <div className="min-h-screen">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-3">
          <div className="flex items-center gap-6">
            <span className="font-semibold">Smart Access Control</span>
            <nav className="flex flex-wrap gap-4 text-sm">
              {NAV.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  end={item.to === '/'}
                  className={({ isActive }) =>
                    isActive ? 'font-medium text-slate-900' : 'text-slate-500 hover:text-slate-900'
                  }
                >
                  {item.label}
                </NavLink>
              ))}
            </nav>
          </div>
          <div className="flex items-center gap-3 text-sm">
            {role === 'viewer' && (
              <span className="rounded bg-amber-100 px-2 py-0.5 text-amber-800">read-only</span>
            )}
            <button onClick={() => void logout()} className="text-slate-600 hover:text-slate-900">
              Sign out
            </button>
          </div>
        </div>
      </header>
      <main className="mx-auto max-w-6xl px-4 py-6">
        <Outlet />
      </main>
    </div>
  )
}
