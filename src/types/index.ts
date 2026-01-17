// 账号类型
export interface Account {
  id: string
  name?: string
  provider?: string
  authMethod: string
  accessToken: string
  refreshToken: string
  profileArn: string
  region?: string
  expiresAt?: number
  expire?: string
  clientId?: string
  clientSecret?: string
  enabled: boolean
  status: 'active' | 'expired' | 'throttled' | 'error' | 'disabled' | 'banned'
  throttledUntil?: number
}

// 日志类型
export interface LogEntry {
  timestamp: string
  level: string
  target: string
  message: string
}

// 统计数据类型
export interface MetricsData {
  total_requests: number
  streaming_requests: number
  non_streaming_requests: number
  requests_by_endpoint: Record<string, number>
  requests_by_status: Record<string, number>
  requests_by_model: Record<string, number>
  api_type_usage: Record<string, number>
  response_times: number[]
  latency_histogram: {
    p50: number
    p95: number
    p99: number
  }
  recent_requests: RecentRequest[]
  hourly_stats: HourlyStats[]
}

export interface RecentRequest {
  timestamp: string
  endpoint: string
  status_code: number
  model: string
  response_time_ms: number
}

export interface HourlyStats {
  hour: string
  count: number
}
