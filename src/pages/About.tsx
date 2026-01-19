import { Card, Stack, Title, Text, Group, Badge, Divider, Code, List } from '@mantine/core'
import { Info, Github, Heart, Zap, Shield, Globe } from 'lucide-react'

export default function About() {
  return (
    <Stack gap="md" maw={1200} mx="auto" w="100%">
      <Group>
        <Info size={24} color="#228be6" />
        <div>
          <Title order={2}>关于 Kiro Gateway</Title>
          <Text size="sm" c="dimmed">
            Kiro API 网关 - OpenAI/Anthropic 兼容接口
          </Text>
        </div>
      </Group>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Zap size={20} color="#fab005" />
          <Text size="lg" fw={600}>
            项目信息
          </Text>
        </Group>
        <Stack gap="sm">
          <Group>
            <Text size="sm" fw={500} w={100}>
              项目名称
            </Text>
            <Text size="sm">kiro-gateway</Text>
          </Group>
          <Group>
            <Text size="sm" fw={500} w={100}>
              版本
            </Text>
            <Badge color="blue" variant="light">
              v1.0.0
            </Badge>
          </Group>
          <Group>
            <Text size="sm" fw={500} w={100}>
              技术栈
            </Text>
            <Group gap="xs">
              <Badge color="orange" variant="light">
                Tauri 2.0
              </Badge>
              <Badge color="red" variant="light">
                Rust
              </Badge>
              <Badge color="cyan" variant="light">
                React 19
              </Badge>
              <Badge color="blue" variant="light">
                Mantine UI
              </Badge>
            </Group>
          </Group>
          <Group>
            <Text size="sm" fw={500} w={100}>
              后端框架
            </Text>
            <Badge color="grape" variant="light">
              Axum
            </Badge>
          </Group>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Shield size={20} color="#40c057" />
          <Text size="lg" fw={600}>
            核心功能
          </Text>
        </Group>
        <List spacing="sm" size="sm">
          <List.Item>
            <Text size="sm">
              <strong>OpenAI 兼容接口</strong> - 支持 <Code>/v1/chat/completions</Code> 端点
            </Text>
          </List.Item>
          <List.Item>
            <Text size="sm">
              <strong>Anthropic 兼容接口</strong> - 支持 <Code>/v1/messages</Code> 端点
            </Text>
          </List.Item>
          <List.Item>
            <Text size="sm">
              <strong>多账号管理</strong> - 支持 Social 和 IDC 账号，自动轮询和故障转移
            </Text>
          </List.Item>
          <List.Item>
            <Text size="sm">
              <strong>Token 自动刷新</strong> - 自动检测过期并刷新 Access Token
            </Text>
          </List.Item>
          <List.Item>
            <Text size="sm">
              <strong>配额监控</strong> - 实时查询账号配额使用情况
            </Text>
          </List.Item>
          <List.Item>
            <Text size="sm">
              <strong>健康检查</strong> - 定期检查账号可用性，自动标记异常账号
            </Text>
          </List.Item>
          <List.Item>
            <Text size="sm">
              <strong>流式响应</strong> - 完整支持 SSE 流式输出
            </Text>
          </List.Item>
          <List.Item>
            <Text size="sm">
              <strong>统计监控</strong> - 请求统计、延迟分析、成功率追踪
            </Text>
          </List.Item>
        </List>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Globe size={20} color="#7950f2" />
          <Text size="lg" fw={600}>
            支持的模型
          </Text>
        </Group>
        <Stack gap="xs">
          <Group>
            <Badge color="green" variant="light">
              Claude Haiku 4.5
            </Badge>
            <Text size="sm" c="dimmed">
              快速响应（0.4x 费率）
            </Text>
          </Group>
          <Group>
            <Badge color="blue" variant="light">
              Claude Sonnet 4
            </Badge>
            <Text size="sm" c="dimmed">
              常规模型（1.3x 费率）
            </Text>
          </Group>
          <Group>
            <Badge color="cyan" variant="light">
              Claude Sonnet 4.5
            </Badge>
            <Text size="sm" c="dimmed">
              最新推荐（1.3x 费率）
            </Text>
          </Group>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Github size={20} />
          <Text size="lg" fw={600}>
            开源信息
          </Text>
        </Group>
        <Stack gap="sm">
          <Text size="sm">
            本项目基于以下开源项目开发：
          </Text>
          <List spacing="xs" size="sm">
            <List.Item>
              <strong>aliom-v/KiroGate</strong> - Python + FastAPI 实现
            </List.Item>
            <List.Item>
              <strong>Jwadow/kiro-openai-gateway</strong> - KiroGate 上游项目
            </List.Item>
            <List.Item>
              <strong>justlovemaki/AIClient-2-API</strong> - 多 Provider 架构参考
            </List.Item>
            <List.Item>
              <strong>aiclientproxy/proxycast</strong> - Tauri 桌面应用参考
            </List.Item>
            <List.Item>
              <strong>hank9999/kiro.rs</strong> - Rust + React 前端参考
            </List.Item>
          </List>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Heart size={20} color="#fa5252" />
          <Text size="lg" fw={600}>
            致谢
          </Text>
        </Group>
        <Text size="sm">
          感谢所有为 Kiro API 生态做出贡献的开发者，以及 Anthropic 提供的强大 AI 能力。
        </Text>
        <Divider my="md" />
        <Text size="xs" c="dimmed" ta="center">
          Made with ❤️ by Kiro Gateway Team
        </Text>
      </Card>
    </Stack>
  )
}
