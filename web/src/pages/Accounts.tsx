import { useState, useEffect } from 'react'
import { api, type Account } from '../api/client'

const statusColors: Record<string, string> = {
  active: 'bg-green-500',
  expired: 'bg-red-500',
  throttled: 'bg-yellow-500',
  error: 'bg-red-500',
  disabled: 'bg-gray-500',
}

const statusLabels: Record<string, string> = {
  active: '正常',
  expired: '已过期',
  throttled: '限流中',
  error: '错误',
  disabled: '已禁用',
}

const exampleJson = `// Social 账号（Google/GitHub）
{
  "provider": "Google",
  "authMethod": "social",
  "refreshToken": "aorAAAAA..."
}

// IdC 账号（BuilderId/Enterprise）
{
  "provider": "BuilderId",
  "authMethod": "idc",
  "refreshToken": "aorAAAAA...",
  "clientId": "MkAG97...",
  "clientSecret": "eyJraWQ..."
}

// 批量导入（数组）
[
  { "provider": "Google", "authMethod": "social", "refreshToken": "..." },
  { "provider": "BuilderId", "authMethod": "idc", "refreshToken": "...", "clientId": "...", "clientSecret": "..." }
]`

interface QuotaInfo {
  email?: string
  used: number
  limit: number
  usagePercent: number
  subscriptionType?: string
  freeTrialStatus?: string
  freeTrialExpiry?: number
  nextReset?: number
  currency?: string
  overageRate?: number
}

export default function Accounts() {
  const [accounts, setAccounts] = useState<Account[]>([])
  const [quotas, setQuotas] = useState<Record<string, QuotaInfo | null>>({})
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showModal, setShowModal] = useState(false)
  const [jsonInput, setJsonInput] = useState('')

  const parseQuotaResponse = (response: any): QuotaInfo | null => {
    try {
      console.log('🔍 原始配额响应:', response)
      
      const breakdown = response.usageBreakdownList?.[0]
      if (!breakdown) {
        console.warn('⚠️ 没有找到 usageBreakdownList')
        return null
      }

      const freeTrialInfo = breakdown.freeTrialInfo
      const isFreeTrialActive = freeTrialInfo?.freeTrialStatus === 'ACTIVE'

      const used = isFreeTrialActive 
        ? (freeTrialInfo.currentUsageWithPrecision ?? 0)
        : (breakdown.currentUsageWithPrecision ?? 0)
      
      const limit = isFreeTrialActive
        ? (freeTrialInfo.usageLimitWithPrecision ?? 0)
        : (breakdown.usageLimitWithPrecision ?? 0)

      const quotaInfo = {
        email: response.userInfo?.email,
        used,
        limit,
        usagePercent: limit > 0 ? Math.round((used / limit) * 100) : 0,
        subscriptionType: response.subscriptionInfo?.subscriptionTitle,
        freeTrialStatus: freeTrialInfo?.freeTrialStatus,
        freeTrialExpiry: freeTrialInfo?.freeTrialExpiry,
        nextReset: response.nextDateReset || breakdown.nextDateReset,
        currency: breakdown.currency,
        overageRate: breakdown.overageRate,
      }
      
      console.log('✅ 解析后的配额信息:', quotaInfo)
      return quotaInfo
    } catch (e) {
      console.error('❌ 解析配额响应失败:', e)
      return null
    }
  }

  const fetchAccounts = async () => {
    try {
      setLoading(true)
      const data = await api.getAccounts()
      setAccounts(data)
      setError(null)
      // 获取每个账号的配额
      const quotaPromises = data.map(async (acc) => {
        try {
          const response = await api.getQuota(acc.id)
          const quota = parseQuotaResponse(response)
          return { id: acc.id, quota }
        } catch (e) {
          console.error(`获取账号 ${acc.id} 配额失败:`, e)
          return { id: acc.id, quota: null }
        }
      })
      const results = await Promise.all(quotaPromises)
      const quotaMap: Record<string, QuotaInfo | null> = {}
      results.forEach(r => { quotaMap[r.id] = r.quota })
      setQuotas(quotaMap)
    } catch (e) {
      setError(e instanceof Error ? e.message : '获取账号失败')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { fetchAccounts() }, [])

  const handleRefresh = async (id: string) => {
    try {
      await api.refreshAccount(id)
      await fetchAccounts()
    } catch (e) {
      alert(e instanceof Error ? e.message : '刷新失败')
    }
  }

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await api.toggleAccount(id, enabled)
      await fetchAccounts()
    } catch (e) {
      alert(e instanceof Error ? e.message : '操作失败')
    }
  }

  const handleImportKiro = async () => {
    try {
      const result = await api.importFromKiro()
      alert(`导入成功: ${result.accountId}`)
      await fetchAccounts()
    } catch (e) {
      alert(e instanceof Error ? e.message : '导入失败')
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除该账号？')) return
    try {
      await api.deleteAccount(id)
      await fetchAccounts()
    } catch (e) {
      alert(e instanceof Error ? e.message : '删除失败')
    }
  }

  const handleImportJson = async () => {
    try {
      const data = JSON.parse(jsonInput)
      const arr = Array.isArray(data) ? data : [data]
      for (const acc of arr) { await api.addAccount(acc) }
      alert(`导入成功: ${arr.length} 个账号`)
      setJsonInput('')
      setShowModal(false)
      await fetchAccounts()
    } catch (e) {
      alert(e instanceof Error ? e.message : '导入失败')
    }
  }

  const handleFileImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    try {
      const text = await file.text()
      const data = JSON.parse(text)
      const arr = Array.isArray(data) ? data : [data]
      for (const acc of arr) { await api.addAccount(acc) }
      alert(`导入成功: ${arr.length} 个账号`)
      await fetchAccounts()
    } catch (err) {
      alert(err instanceof Error ? err.message : '导入失败')
    }
    e.target.value = ''
  }

  if (loading) return <div className="text-center py-8">加载中...</div>
  if (error) return (
    <div className="text-center py-8">
      <p className="text-red-500 mb-4">{error}</p>
      <button onClick={fetchAccounts} className="px-4 py-2 bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))] rounded-md">重试</button>
    </div>
  )

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">账号管理</h2>
        <div className="flex gap-2">
          <button onClick={() => setShowModal(true)} className="px-4 py-2 bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))] rounded-md hover:opacity-80">➕ 添加账号</button>
          <label className="px-4 py-2 bg-[hsl(var(--secondary))] rounded-md hover:opacity-80 cursor-pointer">
            📁 导入文件
            <input type="file" accept=".json" onChange={handleFileImport} className="hidden" />
          </label>
          <button onClick={handleImportKiro} className="px-4 py-2 bg-[hsl(var(--secondary))] rounded-md hover:opacity-80">📥 从 Kiro 导入</button>
          <button onClick={fetchAccounts} className="px-4 py-2 bg-[hsl(var(--secondary))] rounded-md hover:opacity-80">🔄 刷新</button>
        </div>
      </div>

      {accounts.length === 0 ? (
        <p className="text-[hsl(var(--muted-foreground))]">暂无账号</p>
      ) : (
        <div className="space-y-4">
          {accounts.map((account) => {
            const quota = quotas[account.id]
            const usagePercent = quota?.usagePercent ?? 0
            
            return (
            <div key={account.id} className="p-4 bg-[hsl(var(--card))] border border-[hsl(var(--border))] rounded-lg">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-3">
                  <span className={`w-3 h-3 rounded-full ${statusColors[account.status] || 'bg-gray-500'}`} />
                  <div>
                    <p className="font-medium">{quota?.email || account.name || account.id}</p>
                    <p className="text-sm text-[hsl(var(--muted-foreground))]">
                      <span className={`inline-block px-2 py-0.5 rounded text-xs mr-2 ${account.authMethod === 'idc' ? 'bg-blue-600' : 'bg-purple-600'}`}>
                        {account.authMethod === 'idc' ? 'IdC' : 'Social'}
                      </span>
                      {account.provider && <span className="mr-2">{account.provider}</span>}
                      {statusLabels[account.status] || account.status}
                      {quota?.subscriptionType && <span className="ml-2 text-xs">· {quota.subscriptionType}</span>}
                    </p>
                  </div>
                </div>
                <div className="flex gap-2">
                  <button onClick={() => handleRefresh(account.id)} className="px-3 py-1 text-sm bg-[hsl(var(--secondary))] rounded-md hover:opacity-80">刷新</button>
                  <button onClick={() => handleToggle(account.id, account.status === 'disabled')} className={`px-3 py-1 text-sm rounded-md ${account.status === 'disabled' ? 'bg-green-600 text-white' : 'bg-orange-500 text-white'}`}>{account.status === 'disabled' ? '启用' : '禁用'}</button>
                  <button onClick={() => handleDelete(account.id)} className="px-3 py-1 text-sm bg-red-600 text-white rounded-md hover:opacity-80">删除</button>
                </div>
              </div>
              
              {quota && quota.limit > 0 && (
                <div className="space-y-2">
                  {/* 配额进度条 */}
                  <div>
                    <div className="flex justify-between text-xs text-[hsl(var(--muted-foreground))] mb-1">
                      <span>
                        配额使用
                        {quota.freeTrialStatus === 'ACTIVE' && <span className="ml-1 text-green-500">🎁 试用中</span>}
                      </span>
                      <span>{quota.used.toFixed(2)} / {quota.limit} ({usagePercent}%)</span>
                    </div>
                    <div className="h-2 bg-[hsl(var(--secondary))] rounded-full overflow-hidden">
                      <div 
                        className={`h-full transition-all ${usagePercent > 80 ? 'bg-red-500' : usagePercent > 50 ? 'bg-yellow-500' : 'bg-green-500'}`}
                        style={{ width: `${Math.min(usagePercent, 100)}%` }}
                      />
                    </div>
                  </div>

                  {/* 详细信息 */}
                  <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-[hsl(var(--muted-foreground))]">
                    {quota.freeTrialExpiry && (
                      <span>试用到期: {new Date(quota.freeTrialExpiry * 1000).toLocaleDateString()}</span>
                    )}
                    {quota.nextReset && (
                      <span>配额重置: {new Date(quota.nextReset * 1000).toLocaleDateString()}</span>
                    )}
                    {quota.overageRate && quota.currency && (
                      <span>超额费率: {quota.overageRate} {quota.currency}/次</span>
                    )}
                  </div>
                </div>
              )}
            </div>
          )})}
        </div>
      )}

      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowModal(false)}>
          <div className="bg-[hsl(var(--card))] rounded-lg p-6 w-[600px] max-h-[80vh] overflow-auto" onClick={e => e.stopPropagation()}>
            <h3 className="text-xl font-bold mb-4">添加账号</h3>
            <p className="text-sm text-[hsl(var(--muted-foreground))] mb-3">输入 JSON 对象或数组</p>
            <textarea value={jsonInput} onChange={(e) => setJsonInput(e.target.value)} placeholder={exampleJson} className="w-full h-64 px-3 py-2 bg-[hsl(var(--secondary))] rounded-md font-mono text-sm" />
            <div className="flex justify-end gap-2 mt-4">
              <button onClick={() => setShowModal(false)} className="px-4 py-2 bg-[hsl(var(--secondary))] rounded-md">取消</button>
              <button onClick={handleImportJson} disabled={!jsonInput.trim()} className="px-4 py-2 bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))] rounded-md disabled:opacity-50">导入</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
