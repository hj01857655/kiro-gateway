import { useState, useEffect } from 'react'

interface Settings {
  apiUrl: string
  apiKey: string
}

const DEFAULT_API_URL = 'http://127.0.0.1:8080'

export default function Settings() {
  const [settings, setSettings] = useState<Settings>({
    apiUrl: DEFAULT_API_URL,
    apiKey: '',
  })
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    const stored = localStorage.getItem('kirogate-settings')
    if (stored) {
      const parsed = JSON.parse(stored)
      setSettings({
        apiUrl: parsed.apiUrl || DEFAULT_API_URL,
        apiKey: parsed.apiKey || '',
      })
    }
  }, [])

  const handleSave = () => {
    localStorage.setItem('kirogate-settings', JSON.stringify(settings))
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  return (
    <div>
      <h2 className="text-2xl font-bold mb-6">设置</h2>

      <div className="max-w-md space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">API 地址</label>
          <input
            type="text"
            value={settings.apiUrl}
            disabled
            className="w-full px-4 py-2 bg-[hsl(var(--secondary))] rounded-md opacity-60 cursor-not-allowed"
          />
          <p className="text-xs text-[hsl(var(--muted-foreground))] mt-1">
            默认连接本地后端服务
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">API Key</label>
          <input
            type="password"
            value={settings.apiKey}
            onChange={(e) => setSettings({ ...settings, apiKey: e.target.value })}
            placeholder="sk-..."
            className="w-full px-4 py-2 bg-[hsl(var(--secondary))] rounded-md focus:outline-none focus:ring-2 focus:ring-[hsl(var(--ring))]"
          />
          <p className="text-xs text-[hsl(var(--muted-foreground))] mt-1">
            后端配置的 API_KEY 环境变量值
          </p>
        </div>

        <button
          onClick={handleSave}
          className="px-6 py-2 bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))] rounded-md hover:opacity-80"
        >
          {saved ? '✓ 已保存' : '保存设置'}
        </button>
      </div>

      <hr className="my-8 border-[hsl(var(--border))]" />

      <div>
        <h3 className="text-lg font-medium mb-4">关于</h3>
        <p className="text-[hsl(var(--muted-foreground))]">
          KiroGate v0.1.0 - Kiro API 兼容网关
        </p>
        <p className="text-sm text-[hsl(var(--muted-foreground))] mt-2">
          支持 OpenAI 和 Anthropic API 格式
        </p>
      </div>
    </div>
  )
}
