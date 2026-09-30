# Smart Access Control: dashboard

The administration UI for the Smart Access Control backend (the Rust API in
the parent directory). React 19, TypeScript, Vite, Tailwind CSS, TanStack
Query and React Router. It is a separate application: nothing here is part of
the Rust build.

> Status: **Phase 7 complete**: sign-in and session handling, management
> pages, a live monitor with simulator controls, and the event history.

## Running

Start the backend first (see `../README.md`), then:

```sh
npm install
npm run dev          # http://localhost:5173
```

The dev server forwards `/api` (including the WebSocket) to the backend on
`http://127.0.0.1:8080`, so the browser only ever talks to one origin and no
CORS setup is needed. Create an account with the backend's `create-admin`
command and sign in.

## Scripts

| Command | What it does |
| ------- | ------------ |
| `npm run dev` | Development server with hot reload |
| `npm run build` | Type-check and build to `dist/` |
| `npm run typecheck` | TypeScript only |
| `npm run lint` | oxlint |
| `npm test` | Component and client tests (Vitest, jsdom) |

## How sessions work

- `src/api/client.ts` holds the **access token in memory** only. The
  **refresh token never reaches JavaScript**: the login asks for cookie
  mode, and the server keeps it in an `HttpOnly; SameSite=Strict` cookie
  that the browser sends to `/api/v1/auth/*` by itself. After a reload the
  client simply tries a refresh. Nothing is stored in `localStorage` or
  `sessionStorage`.
- A request that gets `401` triggers a token refresh and is retried once.
  Refreshing is **single-flight**: concurrent requests share one refresh.
  The backend rotates refresh tokens and treats a reused one as theft, so
  parallel refreshes would end the session.
- If the refresh is rejected, the session ends and the app returns to the
  sign-in page. Signing out also clears every cached query, so the next
  person to sign in on the tab never sees the previous administrator's data.
- Route guards are for navigation only. The backend enforces every
  permission; a viewer sees a "read-only" badge and gets `403` for changes.

## Pages

| Page | What an admin can do (viewers see the same data, read-only) |
| ---- | ------------------------------------------------------------ |
| Users | Register; suspend, reactivate, archive; jump to a user's cards |
| Cards | Issue (optional expiry); suspend, reactivate, revoke; filter by user |
| Doors | Add; manual online/offline/disable/enable overrides |
| Groups | Create, delete; group page to add and remove members |
| Schedules | Create with a weekly window editor (overnight windows allowed) |
| Permissions | Grant a group access to a door, any time or on a schedule; revoke |
| Controllers | Register (the key is shown once), rotate keys |
| Monitor | Live access events over the WebSocket, filtered by door and decision; simulator controls (start, swipe, outage, disconnect, reconnect, stop) |
| Events | The complete stored history, with all filters kept in the URL |

Shared building blocks live in `src/components/ui.tsx` (tables, fields,
badges, a two-click `ConfirmButton` instead of blocking `window.confirm`)
and `src/api/hooks.ts` (paged queries, writes that refresh the affected
lists, `useCanManage`). Validation errors are shown exactly as the backend
phrases them.

Select boxes load at most 100 items (the API's page size limit), which is
enough for a demo but not for large installations.

## Live monitor

`src/api/eventStream.ts` keeps one WebSocket subscription healthy:

- It subscribes with the current access token and the page's filter.
- When the backend closes the stream because the token expired (`4409`),
  it resubscribes at once with a refreshed token. A rejected token (`4401`)
  gets one retry with a forced refresh; a rejected filter (`4400`) stops the
  feed with a message.
- Any other close (server restart, lost network) is retried with
  exponential backoff (1 s, 2 s, … 30 s).
- The backend never replays events. After an interruption, or when the
  server reports `lagged` (this client fell behind), the page says so and
  links to the event history, which is complete.

It is used through `useSyncExternalStore`; each run of the effect gets a
generation number, so React StrictMode's start-stop-start in development
never leaves two sockets open.

The simulator panel appears only when the backend runs with
`SIMULATOR_ENABLED=true`; otherwise it explains how to enable it. Its status
table polls every 3 seconds while the tab is visible (TanStack Query pauses
polling for hidden tabs); the event feed itself is pushed, not polled.

## API types

`src/api/types.ts` mirrors the backend's JSON by hand. Generating it from
the Rust code would add a frontend-oriented dependency to the backend, which
the project rules forbid. The backend's HTTP integration tests pin those
shapes.
