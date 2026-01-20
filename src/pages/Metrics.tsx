import { useMetrics } from '@/hooks/useMetrics'
import {
  Card,
  Group,
  Text,
  Stack,
  Table,
  Badge,
  SimpleGrid,
  rem,
  Skeleton,
  Title,
} from '@mantine/core'
import { Activity, Clock, Zap, TrendingUp, BarChart3 } from 'lucide-react'
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'
import { format } from 'date-fns'
import { useThemeStore } from '@/stores/themeStore'

export default function Metrics() {
  const { metrics, isLoading } = useMetrics()
  const colorScheme = useThemeStore((state) => state.colorScheme)

  if (isLoading || !metrics) {
    return (
      <Stack gap="md" className="animate-fade-in">
        <Group justify="space-between">
          <Skeleton h={36} w={150} radius="md" />
          <Skeleton h={32} w={100} radius="sm" />
        </Group>

        <SimpleGrid cols={{ base: 1, sm: 2, lg: 4 }} spacing="md">
          {[1, 2, 3, 4].map((i) => (
            <Card key={i} withBorder radius="md" p="lg" className="glass-effect">
              <Group justify="space-between" mb="xs">
                <Skeleton circle h={48} w={48} />
              </Group>
              <Skeleton h={14} w="40%" mb={8} />
              <Skeleton h={28} w="60%" />
            </Card>
          ))}
        </SimpleGrid>

        <Card withBorder radius="md" p="lg" className="glass-effect">
          <Skeleton h={24} w={200} mb="md" />
          <Skeleton h={300} w="100%" />
        </Card>

        <SimpleGrid cols={{ base: 1, md: 2 }} spacing="md">
          <Card withBorder radius="md" p="lg" className="glass-effect"><Skeleton h={24} w={150} mb="md" /><Stack gap="xs"><Skeleton h={40} /><Skeleton h={40} /><Skeleton h={40} /></Stack></Card>
          <Card withBorder radius="md" p="lg" className="glass-effect"><Skeleton h={24} w={150} mb="md" /><Stack gap="xs"><Skeleton h={40} /><Skeleton h={40} /><Skeleton h={40} /></Stack></Card>
        </SimpleGrid>
      </Stack>
    )
  }

  const avgResponseTime =
    metrics.response_times.length > 0
      ? metrics.response_times.reduce((a: number, b: number) => a + b, 0) /
      metrics.response_times.length
      : 0

  const chartData = (metrics.hourly_stats || []).map((item) => ({
    time: format(new Date(parseInt(item.hour)), 'HH:mm'),
    count: item.count,
  }))

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
            <BarChart3 size={24} color="white" />
          </div>
          <div>
            <Title order={2}>统计监控</Title>
            <Text size="sm" c="dimmed">实时监控 API 调用频次与性能指标</Text>
          </div>
        </Group>
        <Badge size="lg" variant="light" color="violet" leftSection={<TrendingUp size={14} />}>
          实时数据
        </Badge>
      </Group>

      <SimpleGrid cols={{ base: 1, sm: 2, lg: 4 }} spacing="md">
        <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
          <Group justify="space-between" mb="xs">
            <div
              style={{
                width: rem(48),
                height: rem(48),
                borderRadius: rem(12),
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, rgba(34, 139, 230, 0.2) 0%, rgba(34, 139, 230, 0.1) 100%)'
                  : 'linear-gradient(135deg, rgba(34, 139, 230, 0.15) 0%, rgba(34, 139, 230, 0.08) 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Activity size={24} color="#228be6" />
            </div>
          </Group>
          <Text size="sm" c="dimmed" mb={4} fw={500} tt="uppercase" style={{ letterSpacing: '0.05em' }}>
            总请求数
          </Text>
          <Text size="xl" fw={700} className="gradient-text">
            {metrics.total_requests.toLocaleString()}
          </Text>
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
          <Group justify="space-between" mb="xs">
            <div
              style={{
                width: rem(48),
                height: rem(48),
                borderRadius: rem(12),
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, rgba(64, 192, 87, 0.2) 0%, rgba(64, 192, 87, 0.1) 100%)'
                  : 'linear-gradient(135deg, rgba(64, 192, 87, 0.15) 0%, rgba(64, 192, 87, 0.08) 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Zap size={24} color="#40c057" />
            </div>
          </Group>
          <Text size="sm" c="dimmed" mb={4} fw={500} tt="uppercase" style={{ letterSpacing: '0.05em' }}>
            流式请求
          </Text>
          <Text size="xl" fw={700} c="green">
            {metrics.streaming_requests.toLocaleString()}
          </Text>
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
          <Group justify="space-between" mb="xs">
            <div
              style={{
                width: rem(48),
                height: rem(48),
                borderRadius: rem(12),
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, rgba(34, 139, 230, 0.2) 0%, rgba(34, 139, 230, 0.1) 100%)'
                  : 'linear-gradient(135deg, rgba(34, 139, 230, 0.15) 0%, rgba(34, 139, 230, 0.08) 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Activity size={24} color="#228be6" />
            </div>
          </Group>
          <Text size="sm" c="dimmed" mb={4} fw={500} tt="uppercase" style={{ letterSpacing: '0.05em' }}>
            非流式请求
          </Text>
          <Text size="xl" fw={700} c="blue">
            {metrics.non_streaming_requests.toLocaleString()}
          </Text>
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
          <Group justify="space-between" mb="xs">
            <div
              style={{
                width: rem(48),
                height: rem(48),
                borderRadius: rem(12),
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, rgba(121, 80, 242, 0.2) 0%, rgba(121, 80, 242, 0.1) 100%)'
                  : 'linear-gradient(135deg, rgba(121, 80, 242, 0.15) 0%, rgba(121, 80, 242, 0.08) 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Clock size={24} color="#7950f2" />
            </div>
          </Group>
          <Text size="sm" c="dimmed" mb={4} fw={500} tt="uppercase" style={{ letterSpacing: '0.05em' }}>
            平均响应
          </Text>
          <Text size="xl" fw={700} c="violet">
            {avgResponseTime.toFixed(0)}ms
          </Text>
        </Card>
      </SimpleGrid>

      <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
        <Group mb="md">
          <TrendingUp size={20} color="#228be6" />
          <Text size="lg" fw={600}>
            24 小时请求趋势
          </Text>
        </Group>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" stroke={colorScheme === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)'} />
            <XAxis dataKey="time" stroke={colorScheme === 'dark' ? '#aaa' : '#666'} />
            <YAxis stroke={colorScheme === 'dark' ? '#aaa' : '#666'} />
            <Tooltip
              contentStyle={{
                backgroundColor: colorScheme === 'dark' ? 'rgba(30, 30, 40, 0.95)' : 'rgba(255, 255, 255, 0.95)',
                border: `1px solid ${colorScheme === 'dark' ? 'rgba(99, 102, 241, 0.3)' : 'rgba(0, 0, 0, 0.1)'} `,
                borderRadius: '8px',
                backdropFilter: 'blur(10px)',
              }}
            />
            <Line type="monotone" dataKey="count" stroke="#6366f1" strokeWidth={3} dot={{ fill: '#6366f1', r: 4 }} />
          </LineChart>
        </ResponsiveContainer>
      </Card>

      <SimpleGrid cols={{ base: 1, md: 2 }} spacing="md">
        <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
          <Text size="lg" fw={600} mb="md">
            模型使用统计
          </Text>
          <Stack gap="sm">
            {Object.entries(metrics.requests_by_model || {}).map(([model, count]) => (
              <Group key={model} justify="space-between" p="xs" style={{
                borderRadius: rem(8),
                background: colorScheme === 'dark' ? 'rgba(99, 102, 241, 0.05)' : 'rgba(99, 102, 241, 0.03)',
                transition: 'all 0.2s ease',
              }}>
                <Text size="sm" fw={500}>{model}</Text>
                <Badge size="lg" variant="light" color="violet">{String(count)}</Badge>
              </Group>
            ))}
          </Stack>
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
          <Text size="lg" fw={600} mb="md">
            API 类型统计
          </Text>
          <Stack gap="sm">
            {Object.entries(metrics.api_type_usage || {}).map(([type, count]) => (
              <Group key={type} justify="space-between" p="xs" style={{
                borderRadius: rem(8),
                background: colorScheme === 'dark' ? 'rgba(99, 102, 241, 0.05)' : 'rgba(99, 102, 241, 0.03)',
                transition: 'all 0.2s ease',
              }}>
                <Text size="sm" fw={500}>{type}</Text>
                <Badge size="lg" variant="light" color="blue">{String(count)}</Badge>
              </Group>
            ))}
          </Stack>
        </Card>
      </SimpleGrid>

      <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
        <Text size="lg" fw={600} mb="md">
          延迟百分位
        </Text>
        <SimpleGrid cols={3} spacing="md">
          <Card withBorder p="md" radius="md" style={{ textAlign: 'center', background: colorScheme === 'dark' ? 'rgba(16, 185, 129, 0.08)' : 'rgba(16, 185, 129, 0.05)' }}>
            <Text size="sm" c="dimmed" mb={8} fw={500} tt="uppercase" style={{ letterSpacing: '0.05em' }}>
              P50
            </Text>
            <Text size="xl" fw={700} c="green">
              {metrics.latency_histogram.p50.toFixed(2)}s
            </Text>
          </Card>
          <Card withBorder p="md" radius="md" style={{ textAlign: 'center', background: colorScheme === 'dark' ? 'rgba(245, 159, 0, 0.08)' : 'rgba(245, 159, 0, 0.05)' }}>
            <Text size="sm" c="dimmed" mb={8} fw={500} tt="uppercase" style={{ letterSpacing: '0.05em' }}>
              P95
            </Text>
            <Text size="xl" fw={700} c="orange">
              {metrics.latency_histogram.p95.toFixed(2)}s
            </Text>
          </Card>
          <Card withBorder p="md" radius="md" style={{ textAlign: 'center', background: colorScheme === 'dark' ? 'rgba(250, 82, 82, 0.08)' : 'rgba(250, 82, 82, 0.05)' }}>
            <Text size="sm" c="dimmed" mb={8} fw={500} tt="uppercase" style={{ letterSpacing: '0.05em' }}>
              P99
            </Text>
            <Text size="xl" fw={700} c="red">
              {metrics.latency_histogram.p99.toFixed(2)}s
            </Text>
          </Card>
        </SimpleGrid>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
        <Text size="lg" fw={600} mb="md">
          最近请求
        </Text>
        <Table striped highlightOnHover withTableBorder={false}>
          <Table.Thead>
            <Table.Tr>
              <Table.Th>时间</Table.Th>
              <Table.Th>端点</Table.Th>
              <Table.Th>状态</Table.Th>
              <Table.Th>耗时</Table.Th>
              <Table.Th>模型</Table.Th>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            {(metrics.recent_requests || []).map((req, idx) => (
              <Table.Tr key={idx}>
                <Table.Td>
                  <Text size="sm" fw={500} style={{ fontFamily: 'monospace' }}>
                    {format(new Date(parseInt(req.timestamp)), 'HH:mm:ss')}
                  </Text>
                </Table.Td>
                <Table.Td>
                  <Text size="sm" style={{ fontFamily: 'monospace', fontSize: '0.85em' }}>
                    {req.endpoint}
                  </Text>
                </Table.Td>
                <Table.Td>
                  <Badge
                    size="lg"
                    variant="light"
                    color={req.status_code >= 200 && req.status_code < 300 ? 'green' : 'red'}
                  >
                    {req.status_code}
                  </Badge>
                </Table.Td>
                <Table.Td>
                  <Text size="sm" fw={500}>
                    {req.response_time_ms.toFixed(0)}ms
                  </Text>
                </Table.Td>
                <Table.Td>
                  <Badge size="sm" variant="outline">
                    {req.model}
                  </Badge>
                </Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      </Card>
    </Stack>
  )
}
