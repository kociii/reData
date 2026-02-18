import { vi } from 'vitest'
import {
  mockProjects,
  mockFields,
  mockAiConfigs,
  mockProcessingTask,
  mockQueryResult,
  createProjectRequest,
  createFieldRequest,
  createAiConfigRequest
} from '../fixtures/testData'

const API_BASE = 'http://127.0.0.1:8000/api'

// 创建 Mock 响应
export function createMockResponse(data: any, ok = true, status = 200) {
  return Promise.resolve({
    ok,
    status,
    json: () => Promise.resolve(data),
    text: () => Promise.resolve(typeof data === 'string' ? data : JSON.stringify(data))
  } as Response)
}

// 存储 WebSocket 实例用于测试
let wsInstances: Map<string, any> = new Map()

// Mock WebSocket 类
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

  constructor(public url: string) {
    // 存储实例
    const taskId = url.split('/').pop()
    if (taskId) {
      wsInstances.set(taskId, this)
    }
  }

  send(data: string) {}
  close() {
    this.readyState = MockWebSocket.CLOSED
    if (this.onclose) {
      this.onclose.call(this as any, new CloseEvent('close'))
    }
  }

  // 测试辅助方法：模拟接收消息
  simulateMessage(data: any) {
    if (this.onmessage) {
      this.onmessage.call(this as any, new MessageEvent('message', { data: JSON.stringify(data) }))
    }
  }

  // 测试辅助方法：模拟打开连接
  simulateOpen() {
    if (this.onopen) {
      this.onopen.call(this as any, new Event('open'))
    }
  }

  // 测试辅助方法：模拟错误
  simulateError() {
    if (this.onerror) {
      this.onerror.call(this as any, new Event('error'))
    }
  }
}

// 重置 WebSocket 实例
export function resetWebSocketInstances() {
  wsInstances.clear()
}

// 获取 WebSocket 实例（用于测试）
export function getWebSocketInstance(taskId: string) {
  return wsInstances.get(taskId)
}

// API Mock 工厂
export function createApiMocks() {
  const mockFetch = vi.fn()

  // 项目 API
  mockFetch.mockImplementation(async (url: string, options?: RequestInit) => {
    const method = options?.method || 'GET'
    const urlObj = new URL(url, 'http://localhost')

    // 项目列表
    if (url === `${API_BASE}/projects/` && method === 'GET') {
      return createMockResponse(mockProjects)
    }

    // 获取单个项目
    if (url.match(/\/projects\/\d+$/) && method === 'GET') {
      const id = parseInt(url.split('/').pop()!)
      const project = mockProjects.find(p => p.id === id)
      if (project) {
        return createMockResponse(project)
      }
      return createMockResponse({ detail: '项目不存在' }, false, 404)
    }

    // 创建项目
    if (url === `${API_BASE}/projects/` && method === 'POST') {
      const body = JSON.parse(options?.body as string)
      const newProject = {
        id: mockProjects.length + 1,
        ...body,
        created_at: new Date().toISOString(),
        updated_at: null
      }
      return createMockResponse(newProject, true, 201)
    }

    // 更新项目
    if (url.match(/\/projects\/\d+$/) && method === 'PUT') {
      const id = parseInt(url.split('/').pop()!)
      const body = JSON.parse(options?.body as string)
      const project = mockProjects.find(p => p.id === id)
      if (project) {
        return createMockResponse({ ...project, ...body })
      }
      return createMockResponse({ detail: '项目不存在' }, false, 404)
    }

    // 删除项目
    if (url.match(/\/projects\/\d+$/) && method === 'DELETE') {
      return createMockResponse({ message: '项目已删除' })
    }

    // 字段列表
    if (url.match(/\/fields\/project\/\d+$/) && method === 'GET') {
      const projectId = parseInt(url.split('/').pop()!)
      const fields = mockFields.filter(f => f.project_id === projectId)
      return createMockResponse(fields)
    }

    // 创建字段
    if (url === `${API_BASE}/fields/` && method === 'POST') {
      const body = JSON.parse(options?.body as string)
      const newField = {
        id: mockFields.length + 1,
        ...body,
        created_at: new Date().toISOString()
      }
      return createMockResponse(newField, true, 201)
    }

    // 更新字段
    if (url.match(/\/fields\/\d+$/) && method === 'PUT') {
      const id = parseInt(url.split('/').pop()!)
      const body = JSON.parse(options?.body as string)
      const field = mockFields.find(f => f.id === id)
      if (field) {
        return createMockResponse({ ...field, ...body })
      }
      return createMockResponse({ detail: '字段不存在' }, false, 404)
    }

    // 删除字段
    if (url.match(/\/fields\/\d+$/) && method === 'DELETE') {
      return createMockResponse({ message: '字段已删除' })
    }

    // AI 配置列表
    if (url === `${API_BASE}/ai-configs/` && method === 'GET') {
      return createMockResponse(mockAiConfigs)
    }

    // 获取默认 AI 配置
    if (url === `${API_BASE}/ai-configs/default` && method === 'GET') {
      const defaultConfig = mockAiConfigs.find(c => c.is_default)
      if (defaultConfig) {
        return createMockResponse(defaultConfig)
      }
      return createMockResponse({ detail: '未找到默认配置' }, false, 404)
    }

    // 创建 AI 配置
    if (url === `${API_BASE}/ai-configs/` && method === 'POST') {
      const body = JSON.parse(options?.body as string)
      const newConfig = {
        id: mockAiConfigs.length + 1,
        ...body,
        created_at: new Date().toISOString(),
        updated_at: null
      }
      return createMockResponse(newConfig, true, 201)
    }

    // 设置默认 AI 配置
    if (url.match(/\/ai-configs\/\d+\/set-default$/) && method === 'POST') {
      const id = parseInt(url.split('/')[5])
      const config = mockAiConfigs.find(c => c.id === id)
      if (config) {
        return createMockResponse({ ...config, is_default: true })
      }
      return createMockResponse({ detail: '配置不存在' }, false, 404)
    }

    // 测试 AI 连接
    if (url === `${API_BASE}/ai-configs/test-connection` && method === 'POST') {
      return createMockResponse({ success: true, message: '连接成功', response: 'OK' })
    }

    // 启动处理
    if (url === `${API_BASE}/processing/start` && method === 'POST') {
      return createMockResponse(mockProcessingTask)
    }

    // 暂停处理
    if (url.match(/\/processing\/pause\/[\w-]+$/) && method === 'POST') {
      return createMockResponse({ ...mockProcessingTask, status: 'paused' })
    }

    // 恢复处理
    if (url.match(/\/processing\/resume\/[\w-]+$/) && method === 'POST') {
      return createMockResponse({ ...mockProcessingTask, status: 'processing' })
    }

    // 取消处理
    if (url.match(/\/processing\/cancel\/[\w-]+$/) && method === 'POST') {
      return createMockResponse({ ...mockProcessingTask, status: 'cancelled' })
    }

    // 查询结果
    if (url.match(/\/results\/\d+\?/) && method === 'GET') {
      return createMockResponse(mockQueryResult)
    }

    // 更新记录
    if (url.match(/\/results\/\d+\/\d+$/) && method === 'PUT') {
      return createMockResponse({})
    }

    // 删除记录
    if (url.match(/\/results\/\d+\/\d+$/) && method === 'DELETE') {
      return createMockResponse({})
    }

    // 默认返回 404
    return createMockResponse({ detail: 'Not found' }, false, 404)
  })

  return mockFetch
}

// 导出测试用的 mock fetch
export const mockApiFetch = createApiMocks()

// 导出 MockWebSocket 以便测试使用
export { MockWebSocket }
