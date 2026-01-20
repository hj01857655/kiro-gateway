import type { LogEntry } from '../types'
import { fetchWithTimeout } from './utils'

export const logsApi = {
  getAll: async (): Promise<LogEntry[]> => {
    const res = await fetchWithTimeout('/admin/logs')
    return res.json()
  },

  clear: async (): Promise<void> => {
    await fetchWithTimeout('/admin/logs/clear', { method: 'POST' })
  },
}
