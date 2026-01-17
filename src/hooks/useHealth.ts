// 健康检查 hooks

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getHealth, checkHealth, getAllocatorStats } from '@/api/health'

// 获取健康状态
export function useHealth() {
  return useQuery({
    queryKey: ['health'],
    queryFn: getHealth,
    refetchInterval: 10000, // 每 10 秒刷新
  })
}

// 手动触发健康检查
export function useCheckHealth() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: checkHealth,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['health'] })
      queryClient.invalidateQueries({ queryKey: ['accounts'] })
    },
  })
}

// 获取智能分配器统计
export function useAllocatorStats() {
  return useQuery({
    queryKey: ['allocator-stats'],
    queryFn: getAllocatorStats,
    refetchInterval: 10000, // 每 10 秒刷新
  })
}
