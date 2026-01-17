import { useState, useEffect } from 'react'
import { fetchLogs, clearLogs } from '../api/accounts'

interface LogEntry {
  timestamp: string
  level: string
  message: string
  target: string
}

export default function Logs() {
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [filter, setFilter] = useState('')
  const [levelFilter, setLevelFilter] = useState('all')

  const loadLogs = async () => {
    try {
      const data = await fetchLogs()
      setLogs(data.logs)
    } catch (error) {
      console.error('加载日志失败:', error)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadLogs()
    const interval = setInterval(loadLogs, 3000) // 每 3 秒刷新
    return () => clearInterval(interval)
  }, [])

  const handleClearLogs = async () => {
    if (!confirm('确定清空所有日志？')) return
    try {
      await clearLogs()
      await loadLogs()
    } catch (error) {
      alert('清空失败: ' + error)
    }
  }

  const getLevelColor = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error':
        return 'text-red-500'
      case 'warn':
        return 'text-yellow-500'
      case 'info':
        return 'text-blue-500'
      case 'debug':
        return 'text-gray-500'
      default:
        return ''
    }
  }

  const filteredLogs = logs.filter((log) => {
    const matchesText = filter === '' || log.message.toLowerCase().includes(filter.toLowerCase())
    const matchesLevel = levelFilter === 'all' || log.level.toLowerCase() === levelFilter.toLowerCase()
    return matchesText && matchesLevel
  })

  if (loading) {
    return <div className="text-center py-8">加载中...</div>
  }

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">日志查看</h2>
        <div className="flex gap-2">
          <button
            onClick={loadLogs}
            className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600"
          >
            刷新
          </button>
          <button
            onClick={handleClearLogs}
            className="px-4 py-2 bg-red-500 text-white rounded-md hover:bg-red-600"
          >
            清空日志
          </button>
        </div>
      </div>

      {/* 过滤器 */}
      <div className="flex gap-4 mb-4">
        <input
          type="text"
          placeholder="搜索日志..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="flex-1 px-3 py-2 border border-[hsl(var(--border))] rounded-md bg-[hsl(var(--background))]"
        />
        <select
          value={levelFilter}
          onChange={(e) => setLevelFilter(e.target.value)}
          className="px-3 py-2 border border-[hsl(var(--border))] rounded-md bg-[hsl(var(--background))]"
        >
          <option value="all">所有级别</option>
          <option value="error">ERROR</option>
          <option value="warn">WARN</option>
          <option value="info">INFO</option>
          <option value="debug">DEBUG</option>
        </select>
      </div>

      {/* 日志列表 */}
      <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
        <div className="space-y-2 max-h-[600px] overflow-y-auto">
          {filteredLogs.length === 0 ? (
            <div className="text-center py-8 text-gray-500">暂无日志</div>
          ) : (
            filteredLogs.map((log, idx) => (
              <div
                key={idx}
                className="p-2 border-b border-[hsl(var(--border))] last:border-b-0 font-mono text-sm"
              >
                <div className="flex items-start gap-3">
                  <span className="text-gray-500 whitespace-nowrap">
                    {new Date(log.timestamp).toLocaleString()}
                  </span>
                  <span className={`font-semibold whitespace-nowrap ${getLevelColor(log.level)}`}>
                    [{log.level.toUpperCase()}]
                  </span>
                  <span className="text-gray-600 whitespace-nowrap">{log.target}</span>
                  <span className="flex-1 break-all">{log.message}</span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      <div className="mt-4 text-sm text-gray-500 text-center">
        显示 {filteredLogs.length} / {logs.length} 条日志（最多保留 1000 条）
      </div>
    </div>
  )
}
