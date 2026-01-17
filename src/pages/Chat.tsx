import { Card, Stack, Title, Text, TextInput, Button, Group, Paper, Center } from '@mantine/core'
import { MessageSquare, Send } from 'lucide-react'
import { useState } from 'react'

export default function Chat() {
  const [message, setMessage] = useState('')
  const [messages, setMessages] = useState<Array<{ role: string; content: string }>>([])

  const handleSend = () => {
    if (!message.trim()) return
    setMessages([...messages, { role: 'user', content: message }])
    setMessage('')
  }

  return (
    <Stack gap="md" style={{ height: 'calc(100vh - 100px)' }}>
      <Group>
        <MessageSquare size={24} color="#228be6" />
        <div>
          <Title order={2}>聊天测试</Title>
          <Text size="sm" c="dimmed">
            测试 Kiro API 网关的聊天功能
          </Text>
        </div>
      </Group>

      <Card shadow="sm" padding="lg" radius="md" withBorder style={{ flex: 1, overflow: 'auto' }}>
        {messages.length === 0 ? (
          <Center h="100%">
            <Stack align="center" gap="md">
              <MessageSquare size={64} color="#adb5bd" />
              <Text c="dimmed">开始对话吧</Text>
            </Stack>
          </Center>
        ) : (
          <Stack gap="md">
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
                    }}
                  >
                    {msg.content}
                  </Text>
                </Paper>
              </Group>
            ))}
          </Stack>
        )}
      </Card>

      <Card shadow="sm" padding="md" radius="md" withBorder>
        <Group>
          <TextInput
            placeholder="输入消息..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSend()}
            style={{ flex: 1 }}
          />
          <Button leftSection={<Send size={16} />} onClick={handleSend} disabled={!message.trim()}>
            发送
          </Button>
        </Group>
        <Text size="xs" c="dimmed" mt="xs">
          此功能仅用于测试，实际使用请通过 API 端点调用
        </Text>
      </Card>
    </Stack>
  )
}
