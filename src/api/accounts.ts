import type { Account } from '../types'
import { fetchWithTimeout } from './utils'

const API_BASE = 'http://127.0.0.1:8080/admin'

export const accountsApi = {
  getAll: async (): Promise<Account[]> => {
    const res = await fetchWithTimeout(`${API_BASE}/accounts`)
    const data = await res.json()
    return data.accounts || []
  },

  add: async (account: Partial<Account>): Promise<Account> => {
    const res = await fetchWithTimeout(`${API_BASE}/accounts`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(account),
    })
    const data = await res.json()
    return data.account
  },

  update: async (id: string, updates: Partial<Account>): Promise<void> => {
    await fetchWithTimeout(`${API_BASE}/accounts/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id, updates }),
    })
  },

  delete: async (id: string): Promise<void> => {
    await fetchWithTimeout(`${API_BASE}/accounts/${id}`, { method: 'DELETE' })
  },

  refresh: async (id: string): Promise<void> => {
    await fetchWithTimeout(`${API_BASE}/accounts/${id}/refresh`, { method: 'POST' })
  },

  getQuota: async (id: string): Promise<import('../types').QuotaInfo> => {
    const res = await fetchWithTimeout(`${API_BASE}/accounts/${id}/quota`)
    return await res.json()
  },
}
