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
} from '@mantine/core'
import { Trash2, Search, FileText } from 'lucide-react'
import { format } from 'date-fns'
import { useState } from 'react'
import { useThemeStore } from '@/stores/themeStore'

export default function Logs() {
  const { logs, isLoading, clearLogs } = useLogs()
  const colorScheme = useThemeStore((state) => state.colorScheme)
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
        <div style={{ textAlign: 'center' }}>
          <Loader size="lg" type="dots" color="violet" />
          <Text size="sm" c="dimmed" mt="md">加载日志...</Text>
        </div>
      </Center>
    )
  }

  return (
    <Stack gap="md" maw={1400} mx="auto" className="animate-fade-in">
      <Group justify="space-between">
        <Group>
          <Title order={2}>日志查看</Title>
          <Badge size="lg" variant="light" color="blue" leftSection={<FileText size={14} />}>
            {filteredLogs.length} 条
          </Badge>
        </Group>
        <Button 
          leftSection={<Trash2 size={16} />} 
          color="red" 
          variant="light"
          onClick={() => clearLogs()}
        >
          清空日志
        </Button>
      </Group>

      <Card shadow="sm" padding="md" radius="md" withBorder className="glass-effect">
        <Group grow>
          <TextInput
            placeholder="搜索日志..."
            leftSection={<Search size={16} />}
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            styles={{
              input: {
                borderRadius: rem(10),
              }
            }}
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
            styles={{
              input: {
                borderRadius: rem(10),
              }
            }}
          />
        </Group>
      </Card>

      {filteredLogs.length === 0 ? (
        <Card shadow="sm" padding="xl" radius="md" withBorder className="glass-effect">
          <Center h={200}>
            <Stack align="center" gap="sm">
              <FileText size={48} color={colorScheme === 'dark' ? '#666' : '#aaa'} />
              <Text c="dimmed" fw={500}>
                {searchTerm || levelFilter !== 'all' ? '没有匹配的日志' : '暂无日志'}
              </Text>
              {!searchTerm && levelFilter === 'all' && (
                <Text size="sm" c="dimmed" ta="center">
                  日志会在有 API 请求通过网关时自动记录
                </Text>
              )}
            </Stack>
          </Center>
        </Card>
      ) : (
        <Stack gap="xs">
          {filteredLogs.map((log, idx) => (
            <Card 
              key={idx} 
              shadow="sm" 
              padding="md" 
              radius="md" 
              withBorder
              className="glass-effect"
              style={{
                borderLeft: `4px solid ${
                  log.level === 'ERROR' ? '#fa5252' :
                  log.level === 'WARN' ? '#fab005' :
                  log.level === 'INFO' ? '#228be6' :
                  '#868e96'
                }`,
              }}
            >
              <Group gap="md" align="flex-start" wrap="nowrap">
                <Badge 
                  color={getLevelColor(log.level)} 
                  variant="light"
                  size="lg"
                  style={{ minWidth: rem(70) }}
                >
                  {log.level}
                </Badge>
                <Stack gap={4} style={{ flex: 1 }}>
                  <Group gap="xs">
                    <Text size="sm" c="dimmed" fw={500} style={{ fontFamily: 'monospace' }}>
                      {format(new Date(log.timestamp), 'yyyy-MM-dd HH:mm:ss')}
                    </Text>
                    <Text size="sm" c="dimmed">
                      •
                    </Text>
                    <Text size="sm" c="dimmed" style={{ fontFamily: 'monospace' }}>
                      {log.target}
                    </Text>
                  </Group>
                  <Text size="sm" style={{ wordBreak: 'break-word' }}>
                    {log.message}
                  </Text>
                </Stack>
              </Group>
            </Card>
          ))}
        </Stack>
      )}

      <Card shadow="sm" padding="sm" radius="md" withBorder className="glass-effect">
        <Group justify="space-between">
          <Text size="sm" c="dimmed" fw={500}>
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
