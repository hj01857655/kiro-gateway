import type { LogEntry } from '../types'

const API_BASE = '/admin'

export const logsApi = {
  getAll: async (): Promise<LogEntry[]> => {
    const res = await fetch(`${API_BASE}/logs`)
    if (!res.ok) throw new Error('获取日志失败')
    return res.json()
  },

  clear: async (): Promise<void> => {
    const res = await fetch(`${API_BASE}/logs/clear`, { method: 'POST' })
    if (!res.ok) throw new Error('清空日志失败')
  },
}
