import {
  Card,
  Stack,
  Title,
  Text,
  TextInput,
  Button,
  Group,
  Center,
  Select,
  Loader,
  ScrollArea,
  Code,
  Alert,
  ActionIcon,
  Tooltip,
  rem,
} from '@mantine/core'
import { MessageSquare, Send, AlertCircle, Sparkles, Trash2, Copy, Check } from 'lucide-react'
import { useState, useRef, useEffect } from 'react'
import { notifications } from '@mantine/notifications'
import { modelsApi, type Model } from '@/api/models'
import ReactMarkdown from 'react-markdown'
import rehypeSanitize from 'rehype-sanitize'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus, vs } from 'react-syntax-highlighter/dist/esm/styles/prism'
import { useThemeStore } from '@/stores/themeStore'

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
  const [messages, setMessages] = useState<Message[]>(() => {
    const saved = localStorage.getItem('chat_messages')
    return saved ? JSON.parse(saved) : []
  })
  const [isLoading, setIsLoading] = useState(false)
  const [model, setModel] = useState('claude-sonnet-4.5')
  const [models, setModels] = useState<Model[]>([])
  const [modelsLoading, setModelsLoading] = useState(true)
  const [apiFormat, setApiFormat] = useState<'openai' | 'anthropic'>('openai')
  const colorScheme = useThemeStore((state) => state.colorScheme)
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
    localStorage.setItem('chat_messages', JSON.stringify(messages))
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
      // 根据 API 格式选择不同的端点和请求体
      const endpoint = apiFormat === 'openai' ? '/v1/chat/completions' : '/v1/messages'
      const requestBody = apiFormat === 'openai'
        ? {
            model,
            messages: [...messages, userMessage],
            stream: true,
          }
        : {
            model,
            messages: [...messages, userMessage].filter(m => m.role !== 'system'),
            max_tokens: 4096,
            stream: true,
          }

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'anthropic-version': '2023-06-01',
        },
        body: JSON.stringify(requestBody),
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
              
              // 根据 API 格式解析不同的响应结构
              let content = ''
              if (apiFormat === 'openai') {
                content = parsed.choices?.[0]?.delta?.content || ''
              } else {
                // Anthropic 格式
                if (parsed.type === 'content_block_delta') {
                  content = parsed.delta?.text || ''
                }
              }
              
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
    if (confirm('确定要清空所有聊天记录吗？')) {
      setMessages([])
      localStorage.removeItem('chat_messages')
    }
  }

  return (
    <Stack gap="md" style={{ height: 'calc(100vh - 120px)' }} className="animate-fade-in">
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
            <Title order={2}>聊天测试</Title>
            <Text size="sm" c="dimmed">
              测试 Kiro API 网关的 OpenAI 兼容性接口
            </Text>
          </div>
        </Group>
        <Group>
          <Select
            value={apiFormat}
            onChange={(value) => setApiFormat(value as 'openai' | 'anthropic')}
            data={[
              { value: 'openai', label: 'OpenAI 格式' },
              { value: 'anthropic', label: 'Anthropic 格式' },
            ]}
            style={{ width: 160 }}
            size="sm"
          />
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
            size="sm"
          />
          <Tooltip label="清空记录">
            <ActionIcon variant="light" color="red" size="lg" onClick={handleClear} disabled={messages.length === 0}>
              <Trash2 size={18} />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>

      <Alert
        icon={<AlertCircle size={16} />}
        color="blue"
        variant="light"
        radius="md"
        styles={{ root: { border: '1px solid rgba(34, 139, 230, 0.2)' } }}
      >
        <Text size="xs">
          当前使用 <strong>{apiFormat === 'openai' ? 'OpenAI' : 'Anthropic'}</strong> 格式，调用{' '}
          <Code style={{ fontSize: rem(12) }}>
            {apiFormat === 'openai' ? '/v1/chat/completions' : '/v1/messages'}
          </Code>{' '}
          接口。已支持 <strong>Markdown</strong> 解析与代码高亮。
        </Text>
      </Alert>

      <Card
        p={0}
        radius="md"
        withBorder
        className="glass-effect"
        style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}
      >
        <ScrollArea style={{ flex: 1 }} viewportRef={viewport} type="auto">
          {messages.length === 0 ? (
            <Center h="100%" style={{ minHeight: 400 }}>
              <Stack align="center" gap="md">
                <div style={{ position: 'relative' }}>
                  <Sparkles size={64} color="var(--kiro-primary)" style={{ opacity: 0.3 }} />
                  <Sparkles size={32} color="var(--kiro-primary)" style={{ position: 'absolute', top: -10, right: -10, animation: 'pulse 2s infinite' }} />
                </div>
                <Text fw={600} c="dimmed">即刻开始对话</Text>
                <Text size="xs" c="dimmed" ta="center">
                  支持流式响应与 Markdown 渲染
                </Text>
              </Stack>
            </Center>
          ) : (
            <Stack gap="xl" p="xl">
              {messages.map((msg, idx) => (
                <Group key={idx} justify={msg.role === 'user' ? 'flex-end' : 'flex-start'} align="flex-start" wrap="nowrap">
                  {msg.role === 'assistant' && (
                    <div
                      style={{
                        minWidth: rem(32),
                        height: rem(32),
                        borderRadius: '50%',
                        background: 'var(--kiro-primary-gradient)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        marginTop: rem(4)
                      }}
                    >
                      <Sparkles size={16} color="white" />
                    </div>
                  )}
                  <div className={`chat-bubble ${msg.role === 'user' ? 'chat-bubble-user' : 'chat-bubble-assistant'}`}>
                    <div className="markdown-content">
                      <ReactMarkdown
                        rehypePlugins={[rehypeSanitize]}
                        components={{
                          code({ node, inline, className, children, ...props }: any) {
                            const match = /language-(\w+)/.exec(className || '')
                            return !inline && match ? (
                              <div style={{ position: 'relative', marginTop: rem(12), marginBottom: rem(12) }}>
                                <SyntaxHighlighter
                                  {...props}
                                  children={String(children).replace(/\n$/, '')}
                                  style={colorScheme === 'dark' ? vscDarkPlus : vs}
                                  language={match[1]}
                                  PreTag="div"
                                  customStyle={{
                                    borderRadius: rem(8),
                                    fontSize: '0.85rem',
                                    margin: 0,
                                    background: colorScheme === 'dark' ? 'rgba(0,0,0,0.3)' : 'rgba(0,0,0,0.03)',
                                  }}
                                />
                                <div style={{ position: 'absolute', top: rem(8), right: rem(8) }}>
                                  <CopyButton value={String(children)}>
                                    {({ copied, copy }) => (
                                      <ActionIcon size="sm" variant="subtle" onClick={copy} color={copied ? 'teal' : 'gray'}>
                                        {copied ? <Check size={14} /> : <Copy size={14} />}
                                      </ActionIcon>
                                    )}
                                  </CopyButton>
                                </div>
                              </div>
                            ) : (
                              <code className={className} {...props}>
                                {children}
                              </code>
                            )
                          }
                        }}
                      >
                        {msg.content}
                      </ReactMarkdown>
                    </div>
                  </div>
                </Group>
              ))}
              {isLoading && messages[messages.length - 1]?.role === 'user' && (
                <Group justify="flex-start" align="flex-start" wrap="nowrap">
                  <div style={{ minWidth: rem(32) }}><Loader size="sm" type="dots" /></div>
                </Group>
              )}
            </Stack>
          )}
        </ScrollArea>
      </Card>

      <Card padding="md" radius="md" withBorder className="glass-effect">
        <Group>
          <TextInput
            placeholder="输入消息，开启精彩对话..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
            style={{ flex: 1 }}
            disabled={isLoading}
            variant="unstyled"
            styles={{ input: { paddingLeft: rem(12), fontSize: rem(15) } }}
          />
          <Button
            leftSection={isLoading ? <Loader size={16} color="white" /> : <Send size={16} />}
            onClick={handleSend}
            disabled={!message.trim() || isLoading}
            loading={isLoading}
            variant="filled"
          >
            发送
          </Button>
        </Group>
        <Group justify="flex-end" mt={4}>
          <Text size="10px" c="dimmed">
            按 Enter 发送，Shift+Enter 换行
          </Text>
        </Group>
      </Card>
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


