import { vi } from 'vitest'
import { config } from '@vue/test-utils'

// 全局 Mock fetch
const mockFetch = vi.fn()
global.fetch = mockFetch

// 重置 Mock
beforeEach(() => {
  mockFetch.mockReset()
})

// Mock WebSocket
class MockWebSocket {
  static CONNECTING = 0
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3

  readyState = MockWebSocket.OPEN
  onopen: ((this: WebSocket, ev: Event) => any) | null = null
  onclose: ((this: WebSocket, ev: CloseEvent) => any) | null = null
  onmessage: ((this: WebSocket, ev: MessageEvent) => any) | null = null
  onerror: ((this: WebSocket, ev: Event) => any) | null = null

  constructor(public url: string) {}

  send(data: string) {}
  close() {
    this.readyState = MockWebSocket.CLOSED
    if (this.onclose) {
      this.onclose.call(this as any, new CloseEvent('close'))
    }
  }

  // 测试辅助方法
  simulateMessage(data: any) {
    if (this.onmessage) {
      this.onmessage.call(this as any, new MessageEvent('message', { data: JSON.stringify(data) }))
    }
  }
}

global.WebSocket = MockWebSocket as any

// 配置 Vue Test Utils
config.global.stubs = {}
