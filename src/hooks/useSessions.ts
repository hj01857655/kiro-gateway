import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { sessionsApi } from '@/api/sessions'
import type { Session } from '@/types'
import { notifications } from '@mantine/notifications'

export function useSessions(workspaceDir?: string) {
  const queryClient = useQueryClient()

  const { data: sessions, isLoading } = useQuery({
    queryKey: ['sessions', workspaceDir],
    queryFn: () => sessionsApi.list(workspaceDir),
  })

  const createMutation = useMutation({
    mutationFn: (session: Partial<Session>) => sessionsApi.create(session),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] })
      notifications.show({
        title: '成功',
        message: '会话创建成功',
        color: 'green',
      })
    },
    onError: (error: Error) => {
      notifications.show({
        title: '创建失败',
        message: error.message,
        color: 'red',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, updates }: { id: string; updates: Partial<Session> }) =>
      sessionsApi.update(id, updates),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] })
      notifications.show({
        title: '成功',
        message: '会话更新成功',
        color: 'green',
      })
    },
    onError: (error: Error) => {
      notifications.show({
        title: '更新失败',
        message: error.message,
        color: 'red',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: ({ id, workspaceDir }: { id: string; workspaceDir?: string }) =>
      sessionsApi.delete(id, workspaceDir),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] })
      notifications.show({
        title: '成功',
        message: '会话删除成功',
        color: 'green',
      })
    },
    onError: (error: Error) => {
      notifications.show({
        title: '删除失败',
        message: error.message,
        color: 'red',
      })
    },
  })

  return {
    sessions,
    isLoading,
    createSession: createMutation.mutate,
    updateSession: updateMutation.mutate,
    deleteSession: deleteMutation.mutate,
  }
}

export function useSession(id: string, workspaceDir?: string) {
  return useQuery({
    queryKey: ['session', id, workspaceDir],
    queryFn: () => sessionsApi.get(id, workspaceDir),
    enabled: !!id,
  })
}

export function useSearchSessions(query: string, workspaceDir?: string) {
  return useQuery({
    queryKey: ['sessions', 'search', query, workspaceDir],
    queryFn: () => sessionsApi.search(query, workspaceDir),
    enabled: query.length > 0,
  })
}
