import { Card, Stack, Title, Text, TextInput, Group, Code } from '@mantine/core'
import { Settings as SettingsIcon, Server, Key, Globe } from 'lucide-react'

export default function Settings() {
  return (
    <Stack gap="md">
      <Title order={2}>设置</Title>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Server size={20} color="#228be6" />
          <Text size="lg" fw={600}>
            服务器配置
          </Text>
        </Group>
        <Stack gap="md">
          <TextInput label="监听地址" value="127.0.0.1" disabled />
          <TextInput label="监听端口" value="8080" disabled />
          <Text size="sm" c="dimmed">
            服务器配置需要通过环境变量设置，重启后生效
          </Text>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Key size={20} color="#40c057" />
          <Text size="lg" fw={600}>
            API Key 管理
          </Text>
        </Group>
        <Stack gap="md">
          <Text size="sm" c="dimmed">
            API Key 用于保护网关接口，客户端需要在请求头中携带 Authorization: Bearer YOUR_API_KEY
          </Text>
          <TextInput
            label="当前 API Key"
            type="password"
            value="sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
            disabled
          />
          <Text size="sm" c="dimmed">
            API Key 通过环境变量 API_KEY 设置
          </Text>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Globe size={20} color="#7950f2" />
          <Text size="lg" fw={600}>
            API 端点
          </Text>
        </Group>
        <Stack gap="sm">
          <div>
            <Text size="sm" fw={500} mb={4}>
              OpenAI 兼容接口
            </Text>
            <Code block>POST http://127.0.0.1:8080/v1/chat/completions</Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={4}>
              Anthropic 兼容接口
            </Text>
            <Code block>POST http://127.0.0.1:8080/v1/messages</Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={4}>
              模型列表
            </Text>
            <Code block>GET http://127.0.0.1:8080/v1/models</Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={4}>
              健康检查
            </Text>
            <Code block>GET http://127.0.0.1:8080/health</Code>
          </div>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <SettingsIcon size={20} color="#868e96" />
          <Text size="lg" fw={600}>
            关于
          </Text>
        </Group>
        <Stack gap="xs">
          <Group>
            <Text size="sm" fw={500}>
              项目名称:
            </Text>
            <Text size="sm">kiro-gateway</Text>
          </Group>
          <Group>
            <Text size="sm" fw={500}>
              版本:
            </Text>
            <Text size="sm">1.0.0</Text>
          </Group>
          <Group>
            <Text size="sm" fw={500}>
              技术栈:
            </Text>
            <Text size="sm">Tauri 2.0 + Rust + React + Mantine</Text>
          </Group>
          <Group>
            <Text size="sm" fw={500}>
              描述:
            </Text>
            <Text size="sm">Kiro API 网关，提供 OpenAI/Anthropic 兼容接口</Text>
          </Group>
        </Stack>
      </Card>
    </Stack>
  )
}
