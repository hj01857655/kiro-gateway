import type { LogEntry } from '../types'
import { fetchWithTimeout } from './utils'

const API_BASE = 'http://127.0.0.1:8080/admin'

export const logsApi = {
  getAll: async (): Promise<LogEntry[]> => {
    const res = await fetchWithTimeout(`${API_BASE}/logs`)
    return res.json()
  },

  clear: async (): Promise<void> => {
    await fetchWithTimeout(`${API_BASE}/logs/clear`, { method: 'POST' })
  },
}
