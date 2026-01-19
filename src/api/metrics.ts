import type { MetricsData } from '../types'
import { fetchWithTimeout } from './utils'

const API_BASE = '/admin'

export const metricsApi = {
  get: async (): Promise<MetricsData> => {
    const res = await fetchWithTimeout(`${API_BASE}/metrics`)
    return res.json()
  },
}
