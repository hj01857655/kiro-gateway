import { useState, useRef, useEffect } from 'react'
import { api } from '../api/client'

interface Message {
  role: 'user' | 'assistant'
  content: string
}

const models = [
  { id: 'auto', name: '自动选择' },
  { id: 'claude-haiku-4.5', name: 'Claude Haiku 4.5' },
  { id: 'claude-sonnet-4', name: 'Claude Sonnet 4' },
  { id: 'claude-sonnet-4.5', name: 'Claude Sonnet 4.5' },
  { id: 'claude-opus-4.5', name: 'Claude Opus 4.5' },
]

export default function Chat() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [model, setModel] = useState('auto')
  const [loading, setLoading] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const handleSend = async () => {
    if (!input.trim() || loading) return

    const userMessage = input.trim()
    setInput('')
    
    // 先添加用户消息
    const newMessages = [...messages, { role: 'user' as const, content: userMessage }]
    setMessages(newMessages)
    setLoading(true)

    // 用于累积 assistant 响应
    let assistantContent = ''

    try {
      // 添加空的 assistant 消息用于流式更新
      setMessages([...newMessages, { role: 'assistant', content: '' }])

      await api.chat(
        newMessages,
        model,
        (chunk) => {
          // 累积内容
          assistantContent += chunk
          // 更新最后一条消息
          setMessages([...newMessages, { role: 'assistant', content: assistantContent }])
        }
      )
    } catch (e) {
      setMessages([...newMessages, { 
        role: 'assistant', 
        content: `错误: ${e instanceof Error ? e.message : '请求失败'}` 
      }])
    } finally {
      setLoading(false)
    }
  }

  const handleClear = () => {
    setMessages([])
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-2xl font-bold">聊天测试</h2>
        <div className="flex gap-2">
          <select
            value={model}
            onChange={(e) => setModel(e.target.value)}
            className="px-3 py-2 bg-[hsl(var(--secondary))] rounded-md"
          >
            {models.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name}
              </option>
            ))}
          </select>
          <button
            onClick={handleClear}
            className="px-4 py-2 bg-[hsl(var(--secondary))] rounded-md hover:opacity-80"
          >
            清空
          </button>
        </div>
      </div>

      {/* 消息列表 */}
      <div className="flex-1 overflow-auto space-y-4 mb-4 p-4 bg-[hsl(var(--card))] border border-[hsl(var(--border))] rounded-lg">
        {messages.length === 0 ? (
          <p className="text-center text-[hsl(var(--muted-foreground))]">开始对话吧 👋</p>
        ) : (
          messages.map((msg, i) => (
            <div
              key={i}
              className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-[80%] px-4 py-2 rounded-lg whitespace-pre-wrap ${
                  msg.role === 'user'
                    ? 'bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]'
                    : 'bg-[hsl(var(--muted))]'
                }`}
              >
                {msg.content || (loading && msg.role === 'assistant' ? '思考中...' : '')}
              </div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* 输入框 */}
      <div className="flex gap-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
          placeholder="输入消息..."
          className="flex-1 px-4 py-2 bg-[hsl(var(--secondary))] rounded-md focus:outline-none focus:ring-2 focus:ring-[hsl(var(--ring))]"
          disabled={loading}
        />
        <button
          onClick={handleSend}
          disabled={loading || !input.trim()}
          className="px-6 py-2 bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))] rounded-md hover:opacity-80 disabled:opacity-50"
        >
          发送
        </button>
      </div>
    </div>
  )
}
