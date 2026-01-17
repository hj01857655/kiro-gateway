import type { MetricsData } from '../types'

const API_BASE = '/admin'

export const metricsApi = {
  get: async (): Promise<MetricsData> => {
    const res = await fetch(`${API_BASE}/metrics`)
    if (!res.ok) throw new Error('获取统计数据失败')
    return res.json()
  },
}
