import { useState, useEffect } from 'react'
import { fetchAccounts, deleteAccount, refreshAccount, enableAccount, disableAccount, importKiroAccount, addAccount, fetchQuota } from '../api/accounts'

interface Account {
  id: string
  name: string
  provider: string
  authMethod: string
  enabled: boolean
  status: string
  isExpired: boolean
  isThrottled: boolean
}

interface QuotaInfo {
  usage: number
  limit: number
  percentage: number
}

export default function Accounts() {
  const [accounts, setAccounts] = useState<Account[]>([])
  const [loading, setLoading] = useState(true)
  const [showAddModal, setShowAddModal] = useState(false)
  const [quotas, setQuotas] = useState<Record<string, QuotaInfo>>({})
  const [loadingQuotas, setLoadingQuotas] = useState<Record<string, boolean>>({})
  const [newAccount, setNewAccount] = useState({
    refreshToken: '',
    authMethod: 'social',
    clientId: '',
    clientSecret: '',
    provider: 'Google',
  })

  const loadAccounts = async () => {
    try {
      const data = await fetchAccounts()
      setAccounts(data.accounts)
      
      // 自动查询所有账号的配额（静默模式）
      for (const account of data.accounts) {
        handleCheckQuota(account.id, true)
      }
    } catch (error) {
      console.error('加载账号失败:', error)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadAccounts()
  }, [])

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除此账号？')) return
    try {
      await deleteAccount(id)
      await loadAccounts()
    } catch (error) {
      alert('删除失败: ' + error)
    }
  }

  const handleRefresh = async (id: string) => {
    try {
      await refreshAccount(id)
      await loadAccounts()
      alert('刷新成功')
    } catch (error) {
      alert('刷新失败: ' + error)
    }
  }

  const handleCheckQuota = async (id: string, silent = false) => {
    setLoadingQuotas(prev => ({ ...prev, [id]: true }))
    try {
      const rawQuota = await fetchQuota(id)
      
      // 解析 Kiro API 返回的配额数据（驼峰格式）
      const breakdown = rawQuota.usageBreakdownList?.[0]
      if (!breakdown) {
        if (!silent) alert('配额数据格式错误')
        return
      }
      
      // 优先使用免费试用配额
      let usage: number, limit: number
      if (breakdown.freeTrialInfo?.freeTrialStatus === 'ACTIVE') {
        usage = breakdown.freeTrialInfo.currentUsageWithPrecision ?? 0
        limit = breakdown.freeTrialInfo.usageLimitWithPrecision ?? 0
      } else {
        usage = breakdown.currentUsageWithPrecision ?? 0
        limit = breakdown.usageLimitWithPrecision ?? 0
      }
      
      const percentage = limit > 0 ? (usage / limit) * 100 : 0
      
      setQuotas(prev => ({ ...prev, [id]: { usage, limit, percentage } }))
    } catch (error) {
      if (!silent) alert('查询配额失败: ' + error)
      console.error(`账号 ${id} 配额查询失败:`, error)
    } finally {
      setLoadingQuotas(prev => ({ ...prev, [id]: false }))
    }
  }

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      if (enabled) {
        await disableAccount(id)
      } else {
        await enableAccount(id)
      }
      await loadAccounts()
    } catch (error) {
      alert('操作失败: ' + error)
    }
  }

  const handleImportKiro = async () => {
    try {
      await importKiroAccount()
      await loadAccounts()
      alert('导入成功')
    } catch (error) {
      alert('导入失败: ' + error)
    }
  }

  const handleAddAccount = async () => {
    try {
      await addAccount(newAccount)
      await loadAccounts()
      setShowAddModal(false)
      setNewAccount({
        refreshToken: '',
        authMethod: 'social',
        clientId: '',
        clientSecret: '',
        provider: 'Google',
      })
      alert('添加成功')
    } catch (error) {
      alert('添加失败: ' + error)
    }
  }

  const getStatusColor = (account: Account) => {
    if (!account.enabled) return 'text-gray-500'
    if (account.isExpired) return 'text-red-500'
    if (account.isThrottled) return 'text-yellow-500'
    if (account.status === 'active') return 'text-green-500'
    return 'text-gray-500'
  }

  const getStatusText = (account: Account) => {
    if (!account.enabled) return '已禁用'
    if (account.isExpired) return '已过期'
    if (account.isThrottled) return '限流中'
    if (account.status === 'active') return '正常'
    return account.status
  }

  if (loading) {
    return <div className="text-center py-8">加载中...</div>
  }

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">账号管理</h2>
        <div className="space-x-2">
          <button
            onClick={handleImportKiro}
            className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600"
          >
            从 Kiro IDE 导入
          </button>
          <button
            onClick={() => setShowAddModal(true)}
            className="px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600"
          >
            添加账号
          </button>
        </div>
      </div>

      {accounts.length === 0 ? (
        <div className="text-center py-12 text-gray-500">
          <p className="mb-4">暂无账号</p>
          <button
            onClick={handleImportKiro}
            className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600"
          >
            从 Kiro IDE 导入
          </button>
        </div>
      ) : (
        <div className="grid gap-4">
          {accounts.map((account) => (
            <div
              key={account.id}
              className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]"
            >
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <h3 className="text-lg font-semibold">{account.name}</h3>
                    <span className={`text-sm font-medium ${getStatusColor(account)}`}>
                      {getStatusText(account)}
                    </span>
                    <span className="text-sm text-gray-500">
                      {account.authMethod === 'social' ? 'Social' : 'IDC'}
                    </span>
                  </div>
                  <p className="text-sm text-gray-600">
                    Provider: {account.provider} | ID: {account.id}
                  </p>
                  {quotas[account.id] && (
                    <div className="mt-2 text-sm">
                      <span className="text-gray-600">配额: </span>
                      <span className="font-medium">
                        {quotas[account.id].usage?.toFixed(2) || '?'} / {quotas[account.id].limit?.toFixed(2) || '?'}
                      </span>
                      {quotas[account.id].percentage !== undefined && (
                        <span className={`ml-2 ${
                          quotas[account.id].percentage! > 80 ? 'text-red-500' : 
                          quotas[account.id].percentage! > 50 ? 'text-yellow-500' : 
                          'text-green-500'
                        }`}>
                          ({quotas[account.id].percentage!.toFixed(1)}%)
                        </span>
                      )}
                    </div>
                  )}
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleCheckQuota(account.id)}
                    disabled={loadingQuotas[account.id]}
                    className="px-3 py-1 text-sm bg-purple-500 text-white rounded hover:bg-purple-600 disabled:opacity-50"
                  >
                    {loadingQuotas[account.id] ? '查询中...' : '查配额'}
                  </button>
                  <button
                    onClick={() => handleRefresh(account.id)}
                    className="px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
                  >
                    刷新
                  </button>
                  <button
                    onClick={() => handleToggle(account.id, account.enabled)}
                    className={`px-3 py-1 text-sm rounded ${
                      account.enabled
                        ? 'bg-yellow-500 text-white hover:bg-yellow-600'
                        : 'bg-green-500 text-white hover:bg-green-600'
                    }`}
                  >
                    {account.enabled ? '禁用' : '启用'}
                  </button>
                  <button
                    onClick={() => handleDelete(account.id)}
                    className="px-3 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600"
                  >
                    删除
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* 添加账号模态框 */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-[hsl(var(--card))] p-6 rounded-lg w-full max-w-md">
            <h3 className="text-xl font-bold mb-4">添加账号</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">认证方式</label>
                <select
                  value={newAccount.authMethod}
                  onChange={(e) => setNewAccount({ ...newAccount, authMethod: e.target.value })}
                  className="w-full px-3 py-2 border border-[hsl(var(--border))] rounded-md bg-[hsl(var(--background))]"
                >
                  <option value="social">Social (Google/GitHub)</option>
                  <option value="idc">IDC (Builder ID)</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Provider</label>
                <input
                  type="text"
                  value={newAccount.provider}
                  onChange={(e) => setNewAccount({ ...newAccount, provider: e.target.value })}
                  className="w-full px-3 py-2 border border-[hsl(var(--border))] rounded-md bg-[hsl(var(--background))]"
                  placeholder="Google / GitHub / BuilderId"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Refresh Token</label>
                <textarea
                  value={newAccount.refreshToken}
                  onChange={(e) => setNewAccount({ ...newAccount, refreshToken: e.target.value })}
                  className="w-full px-3 py-2 border border-[hsl(var(--border))] rounded-md bg-[hsl(var(--background))] font-mono text-sm"
                  rows={3}
                  placeholder="粘贴 Refresh Token"
                />
              </div>
              {newAccount.authMethod === 'idc' && (
                <>
                  <div>
                    <label className="block text-sm font-medium mb-1">Client ID</label>
                    <input
                      type="text"
                      value={newAccount.clientId}
                      onChange={(e) => setNewAccount({ ...newAccount, clientId: e.target.value })}
                      className="w-full px-3 py-2 border border-[hsl(var(--border))] rounded-md bg-[hsl(var(--background))] font-mono text-sm"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-1">Client Secret</label>
                    <input
                      type="password"
                      value={newAccount.clientSecret}
                      onChange={(e) => setNewAccount({ ...newAccount, clientSecret: e.target.value })}
                      className="w-full px-3 py-2 border border-[hsl(var(--border))] rounded-md bg-[hsl(var(--background))] font-mono text-sm"
                    />
                  </div>
                </>
              )}
            </div>
            <div className="flex gap-2 mt-6">
              <button
                onClick={handleAddAccount}
                className="flex-1 px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600"
              >
                添加
              </button>
              <button
                onClick={() => setShowAddModal(false)}
                className="flex-1 px-4 py-2 bg-gray-500 text-white rounded-md hover:bg-gray-600"
              >
                取消
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
