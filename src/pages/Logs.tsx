import { useLogs } from '@/hooks/useLogs'
import {
  Button,
  Card,
  Group,
  Text,
  Stack,
  Title,
  Loader,
  Center,
  TextInput,
  Select,
  Badge,
  rem,
  Highlight,
  ActionIcon,
  Tooltip,
} from '@mantine/core'
import { Trash2, Search, FileText, Copy, Check, AlertCircle, AlertTriangle, Info, Bug } from 'lucide-react'
import { format } from 'date-fns'
import { useState } from 'react'

export default function Logs() {
  const { logs, isLoading, clearLogs } = useLogs()
  const [searchTerm, setSearchTerm] = useState('')
  const [levelFilter, setLevelFilter] = useState('all')

  const filteredLogs = logs.filter((log) => {
    const matchesSearch = log.message.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesLevel = levelFilter === 'all' || log.level === levelFilter
    return matchesSearch && matchesLevel
  })

  const getLevelInfo = (level: string) => {
    const info: Record<string, { color: string; icon: any }> = {
      ERROR: { color: 'red', icon: AlertCircle },
      WARN: { color: 'yellow', icon: AlertTriangle },
      INFO: { color: 'blue', icon: Info },
      DEBUG: { color: 'gray', icon: Bug },
    }
    return info[level.toUpperCase()] || { color: 'gray', icon: Info }
  }

  const handleClearLogs = () => {
    if (confirm('确定要清空所有日志记录吗？')) {
      clearLogs()
    }
  }

  if (isLoading) {
    return (
      <Center h={400}>
        <div style={{ textAlign: 'center' }}>
          <Loader size="lg" type="dots" color="violet" />
          <Text size="sm" c="dimmed" mt="md" fw={500}>加载日志数据...</Text>
        </div>
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
            <FileText size={24} color="white" />
          </div>
          <div>
            <Title order={2}>日志查看</Title>
            <Text size="sm" c="dimmed">实时监控网关流量与系统运行状态</Text>
          </div>
        </Group>
        <Button
          leftSection={<Trash2 size={16} />}
          color="red"
          variant="light"
          onClick={handleClearLogs}
          disabled={logs.length === 0}
        >
          清空日志
        </Button>
      </Group>

      <Card p="md" radius="md" withBorder className="glass-effect">
        <Group grow>
          <TextInput
            placeholder="搜索日志消息、端点或模型..."
            leftSection={<Search size={16} />}
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            variant="filled"
          />
          <Select
            placeholder="选择过滤级别"
            value={levelFilter}
            onChange={(value) => setLevelFilter(value || 'all')}
            data={[
              { value: 'all', label: '所有级别 (ALL)' },
              { value: 'ERROR', label: '错误 (ERROR)' },
              { value: 'WARN', label: '警告 (WARN)' },
              { value: 'INFO', label: '信息 (INFO)' },
              { value: 'DEBUG', label: '调试 (DEBUG)' },
            ]}
            variant="filled"
            style={{ maxWidth: rem(200) }}
          />
        </Group>
      </Card>

      {filteredLogs.length === 0 ? (
        <Card shadow="sm" padding="xl" radius="md" withBorder className="glass-effect" style={{ borderStyle: 'dashed' }}>
          <Center h={200}>
            <Stack align="center" gap="sm">
              <FileText size={48} color="var(--kiro-text-dimmed)" style={{ opacity: 0.4 }} />
              <Text c="dimmed" fw={600}>
                {searchTerm || levelFilter !== 'all' ? '未找到相关日志匹配项' : '暂无日志数据'}
              </Text>
              {!searchTerm && levelFilter === 'all' && (
                <Text size="xs" c="dimmed" ta="center">
                  日志会在服务处理请求时实时更新
                </Text>
              )}
            </Stack>
          </Center>
        </Card>
      ) : (
        <Stack gap="xs">
          {filteredLogs.map((log, idx) => {
            const levelInfo = getLevelInfo(log.level);
            return (
              <Card
                key={idx}
                padding="md"
                radius="md"
                withBorder
                className="glass-effect"
                style={{
                  borderLeft: `4px solid var(--mantine-color-${levelInfo.color}-filled)`,
                  overflow: 'visible'
                }}
              >
                <Group gap="md" align="flex-start" wrap="nowrap">
                  <div
                    style={{
                      width: rem(36),
                      height: rem(36),
                      borderRadius: rem(8),
                      background: `var(--mantine-color-${levelInfo.color}-light)`,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      flexShrink: 0,
                    }}
                  >
                    <levelInfo.icon size={18} color={`var(--mantine-color-${levelInfo.color}-filled)`} />
                  </div>
                  <Stack gap={4} style={{ flex: 1 }}>
                    <Group justify="space-between">
                      <Group gap="xs">
                        <Text size="xs" c="dimmed" fw={600} style={{ fontFamily: 'monospace' }}>
                          {format(new Date(log.timestamp), 'HH:mm:ss.SSS')}
                        </Text>
                        <Badge
                          color={levelInfo.color}
                          variant="dot"
                          size="sm"
                        >
                          {log.level}
                        </Badge>
                        <Text size="xs" c="dimmed" style={{ fontFamily: 'monospace' }} fw={500}>
                          {log.target}
                        </Text>
                      </Group>
                      <CopyButton value={log.message}>
                        {({ copied, copy }) => (
                          <Tooltip label={copied ? '已复制' : '复制日志'}>
                            <ActionIcon
                              size="sm"
                              variant="subtle"
                              onClick={copy}
                              color={copied ? 'teal' : 'gray'}
                            >
                              {copied ? <Check size={14} /> : <Copy size={14} />}
                            </ActionIcon>
                          </Tooltip>
                        )}
                      </CopyButton>
                    </Group>
                    <Highlight highlight={searchTerm} size="sm" style={{ wordBreak: 'break-word', lineHeight: 1.6 }}>
                      {log.message}
                    </Highlight>
                  </Stack>
                </Group>
              </Card>
            );
          })}
        </Stack>
      )}

      <Group justify="space-between" mt="xs" px="md">
        <Text size="xs" c="dimmed" fw={500}>
          显示 {filteredLogs.length} 条记录
          {(searchTerm || levelFilter !== 'all') && ` (从总共 ${logs.length} 条中筛选)`}
        </Text>
        <Text size="10px" c="dimmed">
          日志存储于网关本地内存，重启服务后将清空
        </Text>
      </Group>
    </Stack>
  )
}

function CopyButton({ value, children }: { value: string; children: (props: { copied: boolean; copy: () => void }) => React.ReactNode }) {
  const [copied, setCopied] = useState(false)
  const handleCopy = () => {
    navigator.clipboard.writeText(value)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }
  return <>{children({ copied, copy: handleCopy })}</>
}

