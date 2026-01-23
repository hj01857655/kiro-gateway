import { invoke } from '@tauri-apps/api/core'
import type { Account } from '../types'

// 检测是否在 Tauri 环境
const isTauri = '__TAURI_INTERNALS__' in window

export const accountsApi = {
  getAll: async (): Promise<Account[]> => {
    if (isTauri) {
      const accounts = await invoke<Account[]>('get_accounts')
      return accounts
    }
    // 开发环境使用 HTTP
    const res = await fetch('http://127.0.0.1:8080/admin/accounts')
    const data = await res.json()
    return data.accounts || []
  },

  add: async (account: Partial<Account>): Promise<Account> => {
    if (isTauri) {
      const result = await invoke<any>('add_account', { account })
      return result.account
    }
    // 开发环境使用 HTTP
    const res = await fetch('http://127.0.0.1:8080/admin/accounts', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(account),
    })
    const data = await res.json()
    return data.account
  },

  update: async (id: string, updates: Partial<Account>): Promise<void> => {
    if (isTauri) {
      await invoke('update_account', { id, updates })
      return
    }
    // 开发环境使用 HTTP
    await fetch(`http://127.0.0.1:8080/admin/accounts/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id, updates }),
    })
  },

  delete: async (id: string): Promise<void> => {
    if (isTauri) {
      await invoke('delete_account', { id })
      return
    }
    // 开发环境使用 HTTP
    await fetch(`http://127.0.0.1:8080/admin/accounts/${id}`, { method: 'DELETE' })
  },

  refresh: async (id: string): Promise<void> => {
    if (isTauri) {
      await invoke('refresh_account', { id })
      return
    }
    // 开发环境使用 HTTP
    await fetch(`http://127.0.0.1:8080/admin/accounts/${id}/refresh`, { method: 'POST' })
  },

  getQuota: async (id: string): Promise<import('../types').QuotaInfo> => {
    if (isTauri) {
      const result = await invoke<string>('get_account_quota', { id })
      return JSON.parse(result)
    }
    // 开发环境使用 HTTP
    const res = await fetch(`http://127.0.0.1:8080/admin/accounts/${id}/quota`)
    return await res.json()
  },
}
