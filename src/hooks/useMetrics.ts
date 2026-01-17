import { useQuery } from '@tanstack/react-query'
import { metricsApi } from '../api/metrics'

export const useMetrics = () => {
  const query = useQuery({
    queryKey: ['metrics'],
    queryFn: metricsApi.get,
    refetchInterval: 5000, // 每 5 秒刷新
  })

  return {
    metrics: query.data,
    isLoading: query.isLoading,
    error: query.error,
  }
}
