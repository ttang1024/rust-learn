import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { QueryKey } from '@tanstack/react-query'
import { useState } from 'react'

import { useAuth } from '../auth/context'
import type { Page } from './types'

/** Rows per page on list screens. */
export const PAGE_SIZE = 50

/** Largest page the API serves; option lists load at most this many. */
const MAX_PAGE = 100

/** GET `path` (with `?limit=&offset=`) as a paginated list. */
function usePage<T>(resource: string, offset = 0, limit = PAGE_SIZE) {
  const { client } = useAuth()
  return useQuery({
    queryKey: [resource, 'page', offset, limit],
    queryFn: () => client.get<Page<T>>(`/${resource}?limit=${limit}&offset=${offset}`),
    placeholderData: (previous) => previous,
  })
}

/** A list screen's current page (with its own offset) and the props for `<Pager>`. */
export function usePagedList<T>(resource: string) {
  const [offset, setOffset] = useState(0)
  const page = usePage<T>(resource, offset)
  const pager = { offset, limit: PAGE_SIZE, count: page.data?.items.length ?? 0, onChange: setOffset }
  return { page, pager }
}

/** Up to `MAX_PAGE` items, for select boxes and name lookups. */
export function useOptions<T>(resource: string) {
  return usePage<T>(resource, 0, MAX_PAGE)
}

/**
 * A write that refreshes every query under the given keys afterwards, so
 * lists show the server's truth rather than a guess.
 */
export function useApiMutation<TVariables, TResult = unknown>(
  mutationFn: (variables: TVariables) => Promise<TResult>,
  invalidate: QueryKey[],
) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () =>
      Promise.all(invalidate.map((queryKey) => queryClient.invalidateQueries({ queryKey }))),
  })
}

/** Whether the signed-in administrator may change things. The server
 * enforces this regardless; the UI just hides controls that would fail. */
export function useCanManage(): boolean {
  const { state } = useAuth()
  return state.status === 'authenticated' && state.me.role === 'admin'
}

/** id -> label lookup from a list. */
export function byId<T extends { id: string }>(items: T[] | undefined, label: (item: T) => string) {
  const map = new Map((items ?? []).map((item) => [item.id, label(item)]))
  return (id: string | null) => (id === null ? '—' : (map.get(id) ?? id.slice(0, 8)))
}
