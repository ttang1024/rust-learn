import { createContext } from 'react'

import type { SocketFactory } from './eventStream'

/** How WebSockets are created; tests provide a fake. */
export const SocketFactoryContext = createContext<SocketFactory>((url) => new WebSocket(url))
