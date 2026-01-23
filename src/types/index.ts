// 账号类型
export interface Account {
  id: string
  name?: string
  provider?: string
  email?: string  // 新增 email 字段用于去重
  authMethod: string
  accessToken: string
  refreshToken: string
  profileArn: string
  region?: string
  expiresAt?: string  // ISO 8601 格式字符串，与 Kiro IDE 一致
  expire?: string
  clientId?: string
  clientSecret?: string
  startUrl?: string  // Enterprise 专用
  enabled: boolean
  status: 'active' | 'expired' | 'throttled' | 'error' | 'disabled' | 'banned'
  throttledUntil?: number
  quota?: QuotaInfo // 缓存的配额信息
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

// 配额信息类型
export interface QuotaInfo {
  daysUntilReset: number
  nextDateReset: number
  subscriptionInfo: {
    type: string
    subscriptionTitle: string
    overageCapability: string
    upgradeCapability: string
    subscriptionManagementTarget?: string
  }
  usageBreakdownList: UsageBreakdown[]
  userInfo: {
    email: string
    userId: string
  }
  overageConfiguration?: {
    overageEnabled: boolean
  }
}

export interface UsageBreakdown {
  resourceType: string
  unit: string
  displayName: string
  displayNamePlural: string
  usageLimit: number
  usageLimitWithPrecision: number
  currentUsage: number
  currentUsageWithPrecision: number
  currency: string
  overageRate: number
  overageCap: number
  overageCapWithPrecision: number
  currentOverages: number
  currentOveragesWithPrecision: number
  overageCharges: number
  nextDateReset: number
  freeTrialInfo?: {
    freeTrialStatus: string
    usageLimit: number
    usageLimitWithPrecision: number
    currentUsage: number
    currentUsageWithPrecision: number
    freeTrialExpiry: number
  }
  bonuses: Bonus[]
}

export interface Bonus {
  bonusCode: string
  displayName: string
  usageLimit: number
  usageLimitWithPrecision: number
  currentUsage: number
  currentUsageWithPrecision: number
  expiresAt: number
  status: string
}

// 会话类型
export interface Session {
  sessionId: string
  title: string
  workspaceDirectory?: string
  history: any[]  // 对话历史消息
  hidden?: boolean
}

// 会话元数据类型
export interface SessionInfo {
  sessionId: string
  title: string
  dateCreated: string  // Unix 时间戳字符串
  workspaceDirectory?: string
  hidden?: boolean
}
