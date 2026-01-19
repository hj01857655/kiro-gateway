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
} from '@mantine/core'
import { Trash2, Search } from 'lucide-react'
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

  const getLevelColor = (level: string) => {
    const colors: Record<string, string> = {
      ERROR: 'red',
      WARN: 'yellow',
      INFO: 'blue',
      DEBUG: 'gray',
    }
    return colors[level.toUpperCase()] || 'gray'
  }

  if (isLoading) {
    return (
      <Center h={400}>
        <Loader size="lg" />
      </Center>
    )
  }

  return (
    <Stack gap="md" maw={1400} mx="auto">
      <Group justify="space-between">
        <Title order={2}>日志查看</Title>
        <Button leftSection={<Trash2 size={16} />} color="red" onClick={() => clearLogs()}>
          清空日志
        </Button>
      </Group>

      <Card shadow="sm" padding="md" radius="md" withBorder>
        <Group grow>
          <TextInput
            placeholder="搜索日志..."
            leftSection={<Search size={16} />}
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
          <Select
            placeholder="选择级别"
            value={levelFilter}
            onChange={(value) => setLevelFilter(value || 'all')}
            data={[
              { value: 'all', label: '所有级别' },
              { value: 'ERROR', label: 'ERROR' },
              { value: 'WARN', label: 'WARN' },
              { value: 'INFO', label: 'INFO' },
              { value: 'DEBUG', label: 'DEBUG' },
            ]}
          />
        </Group>
      </Card>

      {filteredLogs.length === 0 ? (
        <Card shadow="sm" padding="xl" radius="md" withBorder>
          <Center h={200}>
            <Text c="dimmed">
              {searchTerm || levelFilter !== 'all' ? '没有匹配的日志' : '暂无日志'}
            </Text>
          </Center>
        </Card>
      ) : (
        <Stack gap="xs">
          {filteredLogs.map((log, idx) => (
            <Card key={idx} shadow="sm" padding="md" radius="md" withBorder>
              <Group gap="md" align="flex-start">
                <Badge color={getLevelColor(log.level)} variant="light">
                  {log.level}
                </Badge>
                <Stack gap={4} style={{ flex: 1 }}>
                  <Group gap="xs">
                    <Text size="sm" c="dimmed">
                      {format(new Date(log.timestamp), 'yyyy-MM-dd HH:mm:ss')}
                    </Text>
                    <Text size="sm" c="dimmed">
                      •
                    </Text>
                    <Text size="sm" c="dimmed" style={{ fontFamily: 'monospace' }}>
                      {log.target}
                    </Text>
                  </Group>
                  <Text size="sm">{log.message}</Text>
                </Stack>
              </Group>
            </Card>
          ))}
        </Stack>
      )}

      <Card shadow="sm" padding="sm" radius="md" withBorder>
        <Group justify="space-between">
          <Text size="sm" c="dimmed">
            共 {filteredLogs.length} 条日志
          </Text>
          {(searchTerm || levelFilter !== 'all') && (
            <Text size="sm" c="dimmed">
              （从 {logs.length} 条中筛选）
            </Text>
          )}
        </Group>
      </Card>
    </Stack>
  )
}
