import type { RouteObject } from 'react-router'

import { RequireAuth } from './auth/RequireAuth'
import { AppLayout } from './layout/AppLayout'
import { CardsPage } from './pages/CardsPage'
import { ControllersPage } from './pages/ControllersPage'
import { DoorsPage } from './pages/DoorsPage'
import { EventsPage } from './pages/EventsPage'
import { GroupDetailPage } from './pages/GroupDetailPage'
import { GroupsPage } from './pages/GroupsPage'
import { LoginPage } from './pages/LoginPage'
import { MonitorPage } from './pages/MonitorPage'
import { OverviewPage } from './pages/OverviewPage'
import { PermissionsPage } from './pages/PermissionsPage'
import { SchedulesPage } from './pages/SchedulesPage'
import { UsersPage } from './pages/UsersPage'

/** Route tree, shared by the app and by tests (with a memory router). */
export const routes: RouteObject[] = [
  { path: '/login', element: <LoginPage /> },
  {
    element: <RequireAuth />,
    children: [
      {
        element: <AppLayout />,
        children: [
          { index: true, element: <OverviewPage /> },
          { path: 'monitor', element: <MonitorPage /> },
          { path: 'events', element: <EventsPage /> },
          { path: 'users', element: <UsersPage /> },
          { path: 'cards', element: <CardsPage /> },
          { path: 'doors', element: <DoorsPage /> },
          { path: 'access-groups', element: <GroupsPage /> },
          { path: 'access-groups/:id', element: <GroupDetailPage /> },
          { path: 'schedules', element: <SchedulesPage /> },
          { path: 'permissions', element: <PermissionsPage /> },
          { path: 'controllers', element: <ControllersPage /> },
        ],
      },
    ],
  },
]
