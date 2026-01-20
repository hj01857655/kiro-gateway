import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { accountsApi } from '../api/accounts'
import { useAccountStore } from '../stores/accountStore'
import type { Account } from '../types'

export const useAccounts = () => {
  const queryClient = useQueryClient()
  const setAccounts = useAccountStore((state) => state.setAccounts)

  const query = useQuery({
    queryKey: ['accounts'],
    queryFn: accountsApi.getAll,
    staleTime: 0, // 数据立即过期，每次都重新请求
    gcTime: 5 * 60 * 1000, // 缓存保留 5 分钟（用于后台刷新）
  })

  // 同步数据到 store
  if (query.data) {
    setAccounts(query.data)
  }

  const addMutation = useMutation({
    mutationFn: accountsApi.add,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['accounts'] }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, updates }: { id: string; updates: Partial<Account> }) =>
      accountsApi.update(id, updates),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['accounts'] }),
  })

  const deleteMutation = useMutation({
    mutationFn: accountsApi.delete,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['accounts'] }),
  })

  const refreshMutation = useMutation({
    mutationFn: accountsApi.refresh,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['accounts'] }),
  })

  return {
    accounts: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
    addAccount: addMutation.mutate,
    updateAccount: updateMutation.mutate,
    deleteAccount: deleteMutation.mutate,
    refreshAccount: refreshMutation.mutate,
  }
}
