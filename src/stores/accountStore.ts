import { create } from 'zustand'
import type { Account } from '../types'

interface AccountStore {
  accounts: Account[]
  setAccounts: (accounts: Account[]) => void
  addAccount: (account: Account) => void
  updateAccount: (id: string, updates: Partial<Account>) => void
  deleteAccount: (id: string) => void
  toggleAccount: (id: string) => void
}

export const useAccountStore = create<AccountStore>((set) => ({
  accounts: [],
  setAccounts: (accounts) => set({ accounts }),
  addAccount: (account) => set((state) => ({ accounts: [...state.accounts, account] })),
  updateAccount: (id, updates) =>
    set((state) => ({
      accounts: state.accounts.map((acc) => (acc.id === id ? { ...acc, ...updates } : acc)),
    })),
  deleteAccount: (id) =>
    set((state) => ({ accounts: state.accounts.filter((acc) => acc.id !== id) })),
  toggleAccount: (id) =>
    set((state) => ({
      accounts: state.accounts.map((acc) =>
        acc.id === id ? { ...acc, status: acc.status === 'disabled' ? 'active' : 'disabled' } : acc
      ),
    })),
}))
