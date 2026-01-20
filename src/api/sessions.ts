import type { Session, SessionInfo } from '../types'
import { fetchWithTimeout } from './utils'

export const sessionsApi = {
  // 列出所有会话
  list: async (workspaceDir?: string): Promise<SessionInfo[]> => {
    const params = new URLSearchParams()
    if (workspaceDir) {
      params.append('workspace', workspaceDir)
    }
    const res = await fetchWithTimeout(`/admin/sessions?${params}`)
    const data = await res.json()
    return data.sessions || []
  },

  // 获取会话详情
  get: async (id: string, workspaceDir?: string): Promise<Session> => {
    const params = new URLSearchParams()
    if (workspaceDir) {
      params.append('workspace', workspaceDir)
    }
    const res = await fetchWithTimeout(`/admin/sessions/${id}?${params}`)
    return await res.json()
  },

  // 创建新会话
  create: async (session: Partial<Session>): Promise<Session> => {
    const res = await fetchWithTimeout('/admin/sessions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(session),
    })
    return await res.json()
  },

  // 更新会话
  update: async (id: string, updates: Partial<Session>): Promise<void> => {
    await fetchWithTimeout(`/admin/sessions/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(updates),
    })
  },

  // 删除会话
  delete: async (id: string, workspaceDir?: string): Promise<void> => {
    const params = new URLSearchParams()
    if (workspaceDir) {
      params.append('workspace', workspaceDir)
    }
    await fetchWithTimeout(`/admin/sessions/${id}?${params}`, { method: 'DELETE' })
  },

  // 搜索会话
  search: async (query: string, workspaceDir?: string): Promise<SessionInfo[]> => {
    const params = new URLSearchParams({ q: query })
    if (workspaceDir) {
      params.append('workspace', workspaceDir)
    }
    const res = await fetchWithTimeout(`/admin/sessions/search?${params}`)
    const data = await res.json()
    return data.sessions || []
  },
}
