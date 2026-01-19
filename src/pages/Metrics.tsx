import { useMetrics } from '@/hooks/useMetrics'
import {
  Card,
  Group,
  Text,
  Stack,
  Title,
  Loader,
  Center,
  Table,
  Badge,
  SimpleGrid,
} from '@mantine/core'
import { Activity, Clock, Zap } from 'lucide-react'
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

export default function Metrics() {
  const { metrics, isLoading } = useMetrics()

  if (isLoading || !metrics) {
    return (
      <Center h={400}>
        <Loader size="lg" />
      </Center>
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
    <Stack gap="md" maw={1400} mx="auto">
      <Title order={2}>统计监控</Title>

      <SimpleGrid cols={{ base: 1, sm: 2, lg: 4 }} spacing="md">
        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Group justify="space-between">
            <div>
              <Text size="sm" c="dimmed" mb={5}>
                总请求数
              </Text>
              <Text size="xl" fw={700}>
                {metrics.total_requests}
              </Text>
            </div>
            <Activity size={32} color="#228be6" />
          </Group>
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Group justify="space-between">
            <div>
              <Text size="sm" c="dimmed" mb={5}>
                流式请求
              </Text>
              <Text size="xl" fw={700} c="green">
                {metrics.streaming_requests}
              </Text>
            </div>
            <Zap size={32} color="#40c057" />
          </Group>
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Group justify="space-between">
            <div>
              <Text size="sm" c="dimmed" mb={5}>
                非流式请求
              </Text>
              <Text size="xl" fw={700} c="blue">
                {metrics.non_streaming_requests}
              </Text>
            </div>
            <Activity size={32} color="#228be6" />
          </Group>
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Group justify="space-between">
            <div>
              <Text size="sm" c="dimmed" mb={5}>
                平均响应
              </Text>
              <Text size="xl" fw={700}>
                {avgResponseTime.toFixed(0)}ms
              </Text>
            </div>
            <Clock size={32} color="#7950f2" />
          </Group>
        </Card>
      </SimpleGrid>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Text size="lg" fw={600} mb="md">
          24 小时请求趋势
        </Text>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="time" />
            <YAxis />
            <Tooltip />
            <Line type="monotone" dataKey="count" stroke="#228be6" strokeWidth={2} />
          </LineChart>
        </ResponsiveContainer>
      </Card>

      <SimpleGrid cols={{ base: 1, md: 2 }} spacing="md">
        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Text size="lg" fw={600} mb="md">
            模型使用统计
          </Text>
          <Stack gap="xs">
            {Object.entries(metrics.requests_by_model || {}).map(([model, count]) => (
              <Group key={model} justify="space-between">
                <Text size="sm">{model}</Text>
                <Badge>{String(count)}</Badge>
              </Group>
            ))}
          </Stack>
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Text size="lg" fw={600} mb="md">
            API 类型统计
          </Text>
          <Stack gap="xs">
            {Object.entries(metrics.api_type_usage || {}).map(([type, count]) => (
              <Group key={type} justify="space-between">
                <Text size="sm">{type}</Text>
                <Badge>{String(count)}</Badge>
              </Group>
            ))}
          </Stack>
        </Card>
      </SimpleGrid>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Text size="lg" fw={600} mb="md">
          延迟百分位
        </Text>
        <SimpleGrid cols={3} spacing="md">
          <div style={{ textAlign: 'center' }}>
            <Text size="sm" c="dimmed" mb={5}>
              P50
            </Text>
            <Text size="xl" fw={700}>
              {metrics.latency_histogram.p50.toFixed(2)}s
            </Text>
          </div>
          <div style={{ textAlign: 'center' }}>
            <Text size="sm" c="dimmed" mb={5}>
              P95
            </Text>
            <Text size="xl" fw={700}>
              {metrics.latency_histogram.p95.toFixed(2)}s
            </Text>
          </div>
          <div style={{ textAlign: 'center' }}>
            <Text size="sm" c="dimmed" mb={5}>
              P99
            </Text>
            <Text size="xl" fw={700}>
              {metrics.latency_histogram.p99.toFixed(2)}s
            </Text>
          </div>
        </SimpleGrid>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Text size="lg" fw={600} mb="md">
          最近请求
        </Text>
        <Table striped highlightOnHover>
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
                <Table.Td>{format(new Date(parseInt(req.timestamp)), 'HH:mm:ss')}</Table.Td>
                <Table.Td style={{ fontFamily: 'monospace', fontSize: '0.85em' }}>
                  {req.endpoint}
                </Table.Td>
                <Table.Td>
                  <Badge color={req.status_code >= 200 && req.status_code < 300 ? 'green' : 'red'}>
                    {req.status_code}
                  </Badge>
                </Table.Td>
                <Table.Td>{req.response_time_ms.toFixed(0)}ms</Table.Td>
                <Table.Td>{req.model}</Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      </Card>
    </Stack>
  )
}
