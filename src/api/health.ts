// 健康检查 API
import { fetchWithTimeout } from './utils'

export interface AccountHealth {
  id: string
  name?: string
  status: string
  enabled: boolean
  is_throttled: boolean
  is_available: boolean
  success_count: number
  fail_count: number
  success_rate: number
  last_used?: number
}

export interface HealthResponse {
  accounts: AccountHealth[]
  total: number
  available: number
}

export interface HealthCheckResult {
  success: boolean
  checked: number
  valid: number
  invalid: number
}

export interface AllocatorStats {
  account_id: string
  success_count: number
  fail_count: number
  success_rate: number
  last_used?: number
}

export interface AllocatorStatsResponse {
  stats: AllocatorStats[]
}

// 获取健康状态
export async function getHealth(): Promise<HealthResponse> {
  const response = await fetchWithTimeout('/admin/health')
  return response.json()
}

// 手动触发健康检查
export async function checkHealth(): Promise<HealthCheckResult> {
  const response = await fetchWithTimeout('/admin/health', {
    method: 'POST',
  })
  return response.json()
}

// 获取智能分配器统计
export async function getAllocatorStats(): Promise<AllocatorStatsResponse> {
  const response = await fetchWithTimeout('/admin/allocator/stats')
  return response.json()
}
