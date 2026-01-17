import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { logsApi } from '../api/logs'

export const useLogs = () => {
  const queryClient = useQueryClient()

  const query = useQuery({
    queryKey: ['logs'],
    queryFn: logsApi.getAll,
    refetchInterval: 2000, // 每 2 秒刷新
  })

  const clearMutation = useMutation({
    mutationFn: logsApi.clear,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['logs'] }),
  })

  return {
    logs: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
    clearLogs: clearMutation.mutate,
  }
}
