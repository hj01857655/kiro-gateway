import type { Account } from '../types'

const API_BASE = '/admin'

export const accountsApi = {
  getAll: async (): Promise<Account[]> => {
    const res = await fetch(`${API_BASE}/accounts`)
    if (!res.ok) throw new Error('获取账号列表失败')
    const data = await res.json()
    return data.accounts || []
  },

  add: async (account: Partial<Account>): Promise<Account> => {
    const res = await fetch(`${API_BASE}/accounts`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(account),
    })
    if (!res.ok) throw new Error('添加账号失败')
    const data = await res.json()
    return data.account
  },

  update: async (id: string, updates: Partial<Account>): Promise<void> => {
    const res = await fetch(`${API_BASE}/accounts/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id, updates }),
    })
    if (!res.ok) throw new Error('更新账号失败')
  },

  delete: async (id: string): Promise<void> => {
    const res = await fetch(`${API_BASE}/accounts/${id}`, { method: 'DELETE' })
    if (!res.ok) throw new Error('删除账号失败')
  },

  refresh: async (id: string): Promise<void> => {
    const res = await fetch(`${API_BASE}/accounts/${id}/refresh`, { method: 'POST' })
    if (!res.ok) throw new Error('刷新 Token 失败')
  },

  getQuota: async (id: string): Promise<import('../types').QuotaInfo> => {
    const res = await fetch(`${API_BASE}/accounts/${id}/quota`)
    if (!res.ok) throw new Error('获取配额失败')
    return await res.json()
  },
}
