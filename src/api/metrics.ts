import type { MetricsData } from '../types'
import { fetchWithTimeout } from './utils'

export const metricsApi = {
  get: async (): Promise<MetricsData> => {
    const res = await fetchWithTimeout('/admin/metrics')
    return res.json()
  },
}
