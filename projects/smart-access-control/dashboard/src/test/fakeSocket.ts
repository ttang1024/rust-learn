/**
 * A WebSocket stand-in that tests drive by hand: open it, deliver server
 * messages, close it with a chosen code.
 */
export class FakeSocket {
  static instances: FakeSocket[] = []
  readonly url: string
  sent: unknown[] = []
  closedByClient = false
  onopen: ((event: Event) => void) | null = null
  onmessage: ((event: MessageEvent<string>) => void) | null = null
  onclose: ((event: CloseEvent) => void) | null = null

  constructor(url: string) {
    this.url = url
    FakeSocket.instances.push(this)
  }

  static reset(): void {
    FakeSocket.instances = []
  }

  static latest(): FakeSocket {
    const socket = FakeSocket.instances.at(-1)
    if (!socket) throw new Error('no socket was opened')
    return socket
  }

  /** Factory to pass as `createSocket`. */
  static factory = (url: string): WebSocket => new FakeSocket(url) as unknown as WebSocket

  send(data: string): void {
    this.sent.push(JSON.parse(data))
  }

  close(): void {
    this.closedByClient = true
  }

  // --- test controls ---

  open(): void {
    this.onopen?.(new Event('open'))
  }

  receive(message: unknown): void {
    this.onmessage?.({ data: JSON.stringify(message) } as MessageEvent<string>)
  }

  serverClose(code: number): void {
    this.onclose?.({ code } as CloseEvent)
  }
}
