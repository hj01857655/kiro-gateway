import { useState } from 'react'
import { useSessions, useSession } from '@/hooks/useSessions'
import {
  Card,
  Group,
  Text,
  Badge,
  Stack,
  TextInput,
  ActionIcon,
  Loader,
  Center,
  Title,
  Tooltip,
  rem,
  ScrollArea,
  Code,
} from '@mantine/core'
import { MessageSquare, Trash2, Eye, Search, Calendar, Folder } from 'lucide-react'
import { format } from 'date-fns'
import type { SessionInfo } from '@/types'

export default function Sessions() {
  const { sessions, isLoading, deleteSession } = useSessions()
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const { data: selectedSession } = useSession(selectedSessionId || '', undefined)

  // 过滤会话
  const filteredSessions = sessions?.filter((session) =>
    session.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    session.sessionId.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const handleDelete = (id: string, workspaceDir?: string) => {
    if (confirm('确定要删除这个会话吗？')) {
      deleteSession({ id, workspaceDir })
      if (selectedSessionId === id) {
        setSelectedSessionId(null)
      }
    }
  }

  if (isLoading) {
    return (
      <Center h={400}>
        <Loader size="lg" />
      </Center>
    )
  }

  return (
    <Stack gap="md" className="animate-fade-in">
      <Group justify="space-between">
        <Group>
          <div
            style={{
              width: rem(44),
              height: rem(44),
              borderRadius: rem(12),
              background: 'var(--kiro-primary-gradient)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 8px 16px rgba(99, 102, 241, 0.25)',
            }}
          >
            <MessageSquare size={24} color="white" />
          </div>
          <div>
            <Title order={2}>会话管理</Title>
            <Text size="sm" c="dimmed">
              查看和管理你的聊天会话历史
            </Text>
          </div>
        </Group>
      </Group>

      {/* 搜索框 */}
      <TextInput
        placeholder="搜索会话标题或 ID..."
        leftSection={<Search size={16} />}
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        size="md"
      />

      {/* 统计卡片 */}
      <Card withBorder className="glass-effect">
        <Group grow>
          <Card
            withBorder
            p="sm"
            className="glass-effect"
            style={{ background: 'rgba(99, 102, 241, 0.05) !important' }}
          >
            <Text size="xs" c="dimmed">
              总会话数
            </Text>
            <Text size="xl" fw={700}>
              {sessions?.length || 0}
            </Text>
          </Card>
          <Card
            withBorder
            p="sm"
            className="glass-effect"
            style={{ background: 'rgba(16, 185, 129, 0.05) !important' }}
          >
            <Text size="xs" c="dimmed">
              已选中
            </Text>
            <Text size="xl" fw={700} c="green">
              {selectedSessionId ? 1 : 0}
            </Text>
          </Card>
          <Card
            withBorder
            p="sm"
            className="glass-effect"
            style={{ background: 'rgba(250, 82, 82, 0.05) !important' }}
          >
            <Text size="xs" c="dimmed">
              搜索结果
            </Text>
            <Text size="xl" fw={700}>
              {filteredSessions?.length || 0}
            </Text>
          </Card>
        </Group>
      </Card>

      {(filteredSessions || []).length === 0 ? (
        <Card shadow="sm" padding="xl" radius="md" withBorder>
          <Center h={200}>
            <Stack align="center" gap="md">
              <Text c="dimmed">
                {searchQuery ? '没有找到匹配的会话' : '暂无会话'}
              </Text>
            </Stack>
          </Center>
        </Card>
      ) : (
        <Group align="flex-start" gap="md">
          {/* 左侧：会话列表 */}
          <Stack gap="md" style={{ flex: 1 }}>
            {(filteredSessions || []).map((session: SessionInfo) => (
              <Card
                key={session.sessionId}
                shadow="sm"
                padding="lg"
                radius="md"
                withBorder
                style={{
                  cursor: 'pointer',
                  border:
                    selectedSessionId === session.sessionId
                      ? '2px solid var(--kiro-primary)'
                      : undefined,
                }}
                onClick={() => setSelectedSessionId(session.sessionId)}
              >
                <Stack gap="sm">
                  {/* 顶部：标题和操作 */}
                  <Group justify="space-between" wrap="nowrap">
                    <Group gap="sm">
                      <MessageSquare size={20} color="var(--kiro-primary)" />
                      <Text fw={600} size="md" lineClamp={1}>
                        {session.title}
                      </Text>
                      {session.hidden && (
                        <Badge color="gray" variant="outline" size="sm">
                          隐藏
                        </Badge>
                      )}
                    </Group>
                    <Group gap="xs">
                      <Tooltip label="查看详情">
                        <ActionIcon
                          variant="light"
                          color="blue"
                          size="md"
                          onClick={(e) => {
                            e.stopPropagation()
                            setSelectedSessionId(session.sessionId)
                          }}
                        >
                          <Eye size={16} />
                        </ActionIcon>
                      </Tooltip>
                      <Tooltip label="删除会话">
                        <ActionIcon
                          variant="light"
                          color="red"
                          size="md"
                          onClick={(e) => {
                            e.stopPropagation()
                            handleDelete(session.sessionId, session.workspaceDirectory)
                          }}
                        >
                          <Trash2 size={16} />
                        </ActionIcon>
                      </Tooltip>
                    </Group>
                  </Group>

                  {/* 底部：元数据 */}
                  <Stack gap={4}>
                    <Group gap="xs">
                      <Calendar size={14} />
                      <Text size="xs" c="dimmed">
                        创建时间: {format(new Date(parseInt(session.dateCreated)), 'yyyy-MM-dd HH:mm:ss')}
                      </Text>
                    </Group>
                    {session.workspaceDirectory && (
                      <Group gap="xs">
                        <Folder size={14} />
                        <Text size="xs" c="dimmed" lineClamp={1}>
                          工作区: {session.workspaceDirectory}
                        </Text>
                      </Group>
                    )}
                    <Group gap="xs">
                      <Text size="xs" c="dimmed" style={{ fontFamily: 'monospace' }}>
                        ID: {session.sessionId.slice(0, 8)}...
                      </Text>
                    </Group>
                  </Stack>
                </Stack>
              </Card>
            ))}
          </Stack>

          {/* 右侧：会话详情 */}
          {selectedSessionId && (
            <Card
              shadow="sm"
              padding="lg"
              radius="md"
              withBorder
              style={{ flex: 1, minWidth: 400 }}
            >
              <Stack gap="md">
                <Group justify="space-between">
                  <Text fw={600} size="lg">
                    会话详情
                  </Text>
                  <ActionIcon
                    variant="light"
                    color="gray"
                    onClick={() => setSelectedSessionId(null)}
                  >
                    ✕
                  </ActionIcon>
                </Group>

                {selectedSession ? (
                  <Stack gap="md">
                    <div>
                      <Text size="sm" c="dimmed" mb={4}>
                        标题
                      </Text>
                      <Text fw={500}>{selectedSession.title}</Text>
                    </div>

                    <div>
                      <Text size="sm" c="dimmed" mb={4}>
                        会话 ID
                      </Text>
                      <Code block>{selectedSession.sessionId}</Code>
                    </div>

                    {selectedSession.workspaceDirectory && (
                      <div>
                        <Text size="sm" c="dimmed" mb={4}>
                          工作区目录
                        </Text>
                        <Code block>{selectedSession.workspaceDirectory}</Code>
                      </div>
                    )}

                    <div>
                      <Text size="sm" c="dimmed" mb={4}>
                        对话历史 ({selectedSession.history.length} 条消息)
                      </Text>
                      <ScrollArea h={300} type="auto">
                        <Stack gap="xs">
                          {selectedSession.history.map((msg, idx) => (
                            <Card key={idx} withBorder p="sm" radius="sm">
                              <Code block style={{ whiteSpace: 'pre-wrap', fontSize: rem(11) }}>
                                {JSON.stringify(msg, null, 2)}
                              </Code>
                            </Card>
                          ))}
                        </Stack>
                      </ScrollArea>
                    </div>
                  </Stack>
                ) : (
                  <Center h={200}>
                    <Loader size="md" />
                  </Center>
                )}
              </Stack>
            </Card>
          )}
        </Group>
      )}
    </Stack>
  )
}
