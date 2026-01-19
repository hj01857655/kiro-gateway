import type { MetricsData } from '../types'
import { fetchWithTimeout } from './utils'

const API_BASE = 'http://127.0.0.1:8080/admin'

export const metricsApi = {
  get: async (): Promise<MetricsData> => {
    const res = await fetchWithTimeout(`${API_BASE}/metrics`)
    return res.json()
  },
}
