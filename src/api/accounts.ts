import { invoke } from '@tauri-apps/api/core'
import type { Account } from '../types'

export const accountsApi = {
  getAll: async (): Promise<Account[]> => {
    const accounts = await invoke<Account[]>('get_accounts')
    return accounts
  },

  add: async (account: Partial<Account>): Promise<Account> => {
    const result = await invoke<any>('add_account', { account })
    return result.account
  },

  update: async (id: string, updates: Partial<Account>): Promise<void> => {
    await invoke('update_account', { id, updates })
  },

  delete: async (id: string): Promise<void> => {
    await invoke('delete_account', { id })
  },

  refresh: async (id: string): Promise<void> => {
    await invoke('refresh_account', { id })
  },

  getQuota: async (id: string): Promise<import('../types').QuotaInfo> => {
    const result = await invoke<string>('get_account_quota', { id })
    return JSON.parse(result)
  },
}
