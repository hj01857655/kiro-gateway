import {
  Card,
  Stack,
  Title,
  Text,
  TextInput,
  Button,
  Group,
  Paper,
  Center,
  Select,
  Loader,
  ScrollArea,
  Code,
  Alert,
} from '@mantine/core'
import { MessageSquare, Send, AlertCircle, Sparkles } from 'lucide-react'
import { useState, useRef, useEffect } from 'react'
import { notifications } from '@mantine/notifications'
import { modelsApi, type Model } from '@/api/models'

interface Message {
  role: 'user' | 'assistant' | 'system'
  content: string
}

// 模型显示名称映射
const MODEL_LABELS: Record<string, string> = {
  'claude-haiku-4.5': 'Claude Haiku 4.5 (快速)',
  'claude-sonnet-4': 'Claude Sonnet 4',
  'claude-sonnet-4.5': 'Claude Sonnet 4.5 (推荐)',
  'claude-opus-4.5': 'Claude Opus 4.5',
}

export default function Chat() {
  const [message, setMessage] = useState('')
  const [messages, setMessages] = useState<Message[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [model, setModel] = useState('claude-sonnet-4.5')
  const [models, setModels] = useState<Model[]>([])
  const [modelsLoading, setModelsLoading] = useState(true)
  const viewport = useRef<HTMLDivElement>(null)

  // 加载模型列表
  useEffect(() => {
    loadModels()
  }, [])

  const loadModels = async () => {
    setModelsLoading(true)
    try {
      const modelList = await modelsApi.list()
      setModels(modelList)
    } catch (error) {
      console.error('加载模型列表失败:', error)
    } finally {
      setModelsLoading(false)
    }
  }

  // 自动滚动到底部
  useEffect(() => {
    if (viewport.current) {
      viewport.current.scrollTo({ top: viewport.current.scrollHeight, behavior: 'smooth' })
    }
  }, [messages])

  const handleSend = async () => {
    if (!message.trim() || isLoading) return

    const userMessage: Message = { role: 'user', content: message.trim() }
    setMessages((prev) => [...prev, userMessage])
    setMessage('')
    setIsLoading(true)

    try {
      const response = await fetch('/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          model,
          messages: [...messages, userMessage],
          stream: true,
        }),
      })

      if (!response.ok) {
        const error = await response.json()
        throw new Error(error.error?.message || '请求失败')
      }

      const reader = response.body?.getReader()
      const decoder = new TextDecoder()
      let assistantMessage = ''

      // 添加空的助手消息
      setMessages((prev) => [...prev, { role: 'assistant', content: '' }])

      while (reader) {
        const { done, value } = await reader.read()
        if (done) break

        const chunk = decoder.decode(value)
        const lines = chunk.split('\n')

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6)
            if (data === '[DONE]') continue

            try {
              const parsed = JSON.parse(data)
              const content = parsed.choices?.[0]?.delta?.content
              if (content) {
                assistantMessage += content
                // 更新最后一条消息
                setMessages((prev) => {
                  const newMessages = [...prev]
                  newMessages[newMessages.length - 1] = {
                    role: 'assistant',
                    content: assistantMessage,
                  }
                  return newMessages
                })
              }
            } catch (e) {
              // 忽略解析错误
            }
          }
        }
      }
    } catch (error) {
      notifications.show({
        title: '发送失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
      // 移除空的助手消息
      setMessages((prev) => prev.slice(0, -1))
    } finally {
      setIsLoading(false)
    }
  }

  const handleClear = () => {
    setMessages([])
  }

  return (
    <Stack gap="md" style={{ height: 'calc(100vh - 120px)' }}>
      <Group justify="space-between">
        <Group>
          <MessageSquare size={24} color="#228be6" />
          <div>
            <Title order={2}>聊天测试</Title>
            <Text size="sm" c="dimmed">
              测试 Kiro API 网关的聊天功能
            </Text>
          </div>
        </Group>
        <Group>
          <Select
            value={model}
            onChange={(value) => setModel(value || 'claude-sonnet-4.5')}
            data={models.map((m) => ({
              value: m.id,
              label: MODEL_LABELS[m.id] || m.id,
            }))}
            style={{ width: 220 }}
            disabled={modelsLoading || models.length === 0}
            placeholder={modelsLoading ? '加载中...' : '选择模型'}
          />
          <Button variant="light" onClick={handleClear} disabled={messages.length === 0}>
            清空对话
          </Button>
        </Group>
      </Group>

      <Alert icon={<AlertCircle size={16} />} color="blue" variant="light">
        此功能调用本地网关的 <Code>/v1/chat/completions</Code> 接口，用于测试 OpenAI 兼容性
      </Alert>

      <Card
        shadow="sm"
        padding="lg"
        radius="md"
        withBorder
        style={{ flex: 1, display: 'flex', flexDirection: 'column' }}
      >
        <ScrollArea style={{ flex: 1 }} viewportRef={viewport}>
          {messages.length === 0 ? (
            <Center h="100%">
              <Stack align="center" gap="md">
                <Sparkles size={64} color="#adb5bd" />
                <Text c="dimmed">开始对话吧</Text>
                <Text size="xs" c="dimmed">
                  支持流式响应，实时显示 AI 回复
                </Text>
              </Stack>
            </Center>
          ) : (
            <Stack gap="md" p="md">
              {messages.map((msg, idx) => (
                <Group key={idx} justify={msg.role === 'user' ? 'flex-end' : 'flex-start'}>
                  <Paper
                    p="md"
                    radius="md"
                    style={{
                      maxWidth: '70%',
                      background:
                        msg.role === 'user'
                          ? 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)'
                          : '#f1f3f5',
                    }}
                  >
                    <Text
                      size="sm"
                      style={{
                        color: msg.role === 'user' ? 'white' : 'black',
                        whiteSpace: 'pre-wrap',
                        wordBreak: 'break-word',
                      }}
                    >
                      {msg.content}
                    </Text>
                  </Paper>
                </Group>
              ))}
              {isLoading && messages[messages.length - 1]?.role === 'user' && (
                <Group justify="flex-start">
                  <Paper p="md" radius="md" style={{ background: '#f1f3f5' }}>
                    <Loader size="sm" />
                  </Paper>
                </Group>
              )}
            </Stack>
          )}
        </ScrollArea>
      </Card>

      <Card shadow="sm" padding="md" radius="md" withBorder>
        <Group>
          <TextInput
            placeholder="输入消息..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
            style={{ flex: 1 }}
            disabled={isLoading}
          />
          <Button
            leftSection={isLoading ? <Loader size={16} /> : <Send size={16} />}
            onClick={handleSend}
            disabled={!message.trim() || isLoading}
            loading={isLoading}
          >
            发送
          </Button>
        </Group>
        <Text size="xs" c="dimmed" mt="xs">
          按 Enter 发送，Shift+Enter 换行
        </Text>
      </Card>
    </Stack>
  )
}

