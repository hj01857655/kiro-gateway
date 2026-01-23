import { invoke } from '@tauri-apps/api/core'
import type { Account } from '../types'

export const accountsApi = {
  getAll: async (): Promise<Account[]> => {
    const accounts = await invoke<Account[]>('get_accounts')
    return accounts
  },

  add: async (account: Partial<Account>): Promise<Account> => {
    const result = await invoke<{ account: Account }>('add_account', { account })
    return result.account
  },

  update: async (id: string, updates: Partial<Account>): Promise<void> => {
    await invoke<{ success: boolean }>('update_account', { id, updates })
  },

  delete: async (id: string): Promise<void> => {
    await invoke<{ success: boolean }>('delete_account', { id })
  },

  refresh: async (id: string): Promise<void> => {
    await invoke<{ success: boolean }>('refresh_account', { id })
  },

  getQuota: async (id: string): Promise<import('../types').QuotaInfo> => {
    // 现在直接返回 JSON 对象，不需要 JSON.parse
    return await invoke<import('../types').QuotaInfo>('get_account_quota', { id })
  },
}
