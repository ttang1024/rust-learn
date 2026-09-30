import { useContext, useEffect, useMemo, useSyncExternalStore } from 'react'

import { useAuth } from '../auth/context'
import { EventStream, streamUrl } from './eventStream'
import type { EventFilter, StreamSnapshot } from './eventStream'
import { SocketFactoryContext } from './socket'

/**
 * Live access events matching `filter`. A new filter opens a new
 * subscription; unmounting closes it.
 */
export function useEventStream(filter: EventFilter): StreamSnapshot {
  const { client } = useAuth()
  const createSocket = useContext(SocketFactoryContext)
  // Compare filters by value, not by object identity.
  const key = JSON.stringify(filter)
  // Constructing a stream has no side effects; connecting happens below.
  const stream = useMemo(
    () =>
      new EventStream({
        url: streamUrl(),
        token: (force) => client.accessTokenForStream(force),
        filter: JSON.parse(key) as EventFilter,
        createSocket,
      }),
    [client, key, createSocket],
  )
  useEffect(() => {
    stream.start()
    return () => stream.stop()
  }, [stream])
  return useSyncExternalStore(stream.subscribe, stream.getSnapshot)
}
