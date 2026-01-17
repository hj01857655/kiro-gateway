import { useState, useEffect } from 'react'
import { fetchMetrics } from '../api/accounts'

interface MetricsData {
  totalRequests: number
  requestsByEndpoint: Record<string, number>
  requestsByStatus: Record<string, number>
  requestsByModel: Record<string, number>
  streamRequests: number
  nonStreamRequests: number
  apiTypeUsage: Record<string, number>
  avgResponseTime: number
  p50Latency: number
  p95Latency: number
  p99Latency: number
  recentRequests: Array<{
    timestamp: string
    endpoint: string
    status: number
    duration: number
    model: string
  }>
  hourlyStats: Array<{
    hour: number
    count: number
  }>
}

export default function Metrics() {
  const [metrics, setMetrics] = useState<MetricsData | null>(null)
  const [loading, setLoading] = useState(true)

  const loadMetrics = async () => {
    try {
      const data = await fetchMetrics()
      setMetrics(data)
    } catch (error) {
      console.error('加载统计数据失败:', error)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadMetrics()
    const interval = setInterval(loadMetrics, 5000) // 每 5 秒刷新
    return () => clearInterval(interval)
  }, [])

  if (loading) {
    return <div className="text-center py-8">加载中...</div>
  }

  if (!metrics) {
    return <div className="text-center py-8 text-red-500">加载失败</div>
  }

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">统计监控</h2>
        <button
          onClick={loadMetrics}
          className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600"
        >
          刷新
        </button>
      </div>

      {/* 总览卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <div className="text-sm text-gray-600 mb-1">总请求数</div>
          <div className="text-2xl font-bold">{metrics.totalRequests}</div>
        </div>
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <div className="text-sm text-gray-600 mb-1">流式请求</div>
          <div className="text-2xl font-bold">{metrics.streamRequests}</div>
        </div>
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <div className="text-sm text-gray-600 mb-1">非流式请求</div>
          <div className="text-2xl font-bold">{metrics.nonStreamRequests}</div>
        </div>
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <div className="text-sm text-gray-600 mb-1">平均响应时间</div>
          <div className="text-2xl font-bold">{metrics.avgResponseTime.toFixed(0)}ms</div>
        </div>
      </div>

      {/* 延迟统计 */}
      <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))] mb-6">
        <h3 className="text-lg font-semibold mb-4">延迟分布</h3>
        <div className="grid grid-cols-3 gap-4">
          <div>
            <div className="text-sm text-gray-600">P50</div>
            <div className="text-xl font-bold">{metrics.p50Latency.toFixed(0)}ms</div>
          </div>
          <div>
            <div className="text-sm text-gray-600">P95</div>
            <div className="text-xl font-bold">{metrics.p95Latency.toFixed(0)}ms</div>
          </div>
          <div>
            <div className="text-sm text-gray-600">P99</div>
            <div className="text-xl font-bold">{metrics.p99Latency.toFixed(0)}ms</div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
        {/* 端点统计 */}
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <h3 className="text-lg font-semibold mb-4">端点统计</h3>
          <div className="space-y-2">
            {Object.entries(metrics.requestsByEndpoint).map(([endpoint, count]) => (
              <div key={endpoint} className="flex justify-between items-center">
                <span className="text-sm">{endpoint}</span>
                <span className="font-semibold">{count}</span>
              </div>
            ))}
          </div>
        </div>

        {/* 状态码统计 */}
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <h3 className="text-lg font-semibold mb-4">状态码统计</h3>
          <div className="space-y-2">
            {Object.entries(metrics.requestsByStatus).map(([status, count]) => (
              <div key={status} className="flex justify-between items-center">
                <span className="text-sm">HTTP {status}</span>
                <span className="font-semibold">{count}</span>
              </div>
            ))}
          </div>
        </div>

        {/* 模型统计 */}
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <h3 className="text-lg font-semibold mb-4">模型使用统计</h3>
          <div className="space-y-2">
            {Object.entries(metrics.requestsByModel).map(([model, count]) => (
              <div key={model} className="flex justify-between items-center">
                <span className="text-sm">{model}</span>
                <span className="font-semibold">{count}</span>
              </div>
            ))}
          </div>
        </div>

        {/* API 类型统计 */}
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <h3 className="text-lg font-semibold mb-4">API 类型统计</h3>
          <div className="space-y-2">
            {Object.entries(metrics.apiTypeUsage).map(([type, count]) => (
              <div key={type} className="flex justify-between items-center">
                <span className="text-sm">{type}</span>
                <span className="font-semibold">{count}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* 最近请求 */}
      <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
        <h3 className="text-lg font-semibold mb-4">最近请求（最近 50 条）</h3>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[hsl(var(--border))]">
                <th className="text-left py-2">时间</th>
                <th className="text-left py-2">端点</th>
                <th className="text-left py-2">状态</th>
                <th className="text-left py-2">模型</th>
                <th className="text-right py-2">耗时</th>
              </tr>
            </thead>
            <tbody>
              {metrics.recentRequests.map((req, idx) => (
                <tr key={idx} className="border-b border-[hsl(var(--border))]">
                  <td className="py-2">{new Date(req.timestamp).toLocaleString()}</td>
                  <td className="py-2">{req.endpoint}</td>
                  <td className="py-2">
                    <span className={req.status === 200 ? 'text-green-500' : 'text-red-500'}>
                      {req.status}
                    </span>
                  </td>
                  <td className="py-2">{req.model}</td>
                  <td className="py-2 text-right">{req.duration.toFixed(0)}ms</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
