// API 客户端

export interface Account {
  id: string
  name?: string
  provider?: string
  authMethod: 'social' | 'idc'
  status: 'active' | 'expired' | 'throttled' | 'error' | 'disabled'
}

interface Settings {
  apiUrl: string
  apiKey: string
}

function getSettings(): Settings {
  const stored = localStorage.getItem('kirogate-settings')
  if (stored) {
    const parsed = JSON.parse(stored)
    return {
      apiUrl: parsed.apiUrl || '',  // 默认空，走 vite 代理
      apiKey: parsed.apiKey || '',
    }
  }
  return { apiUrl: '', apiKey: '' }  // 默认空，走 vite 代理
}

function getBaseUrl(): string {
  const { apiUrl } = getSettings()
  return apiUrl
}

function getHeaders(): HeadersInit {
  const { apiKey } = getSettings()
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  }
  if (apiKey) {
    headers['Authorization'] = `Bearer ${apiKey}`
  }
  return headers
}

export const api = {
  // 获取账号列表
  async getAccounts(): Promise<Account[]> {
    const res = await fetch(`${getBaseUrl()}/admin/accounts`, {
      headers: getHeaders(),
    })
    if (!res.ok) {
      throw new Error(`获取账号失败: ${res.status}`)
    }
    const data = await res.json()
    return data.accounts || []
  },

  // 刷新账号 Token
  async refreshAccount(id: string): Promise<void> {
    const res = await fetch(`${getBaseUrl()}/admin/accounts/${id}/refresh`, {
      method: 'POST',
      headers: getHeaders(),
    })
    if (!res.ok) {
      throw new Error(`刷新失败: ${res.status}`)
    }
  },

  // 启用/禁用账号
  async toggleAccount(id: string, enable: boolean): Promise<void> {
    const res = await fetch(`${getBaseUrl()}/admin/accounts/${id}/${enable ? 'enable' : 'disable'}`, {
      method: 'POST',
      headers: getHeaders(),
    })
    if (!res.ok) {
      throw new Error(`操作失败: ${res.status}`)
    }
  },

  // 从 Kiro IDE 导入账号
  async importFromKiro(): Promise<{ accountId: string }> {
    const res = await fetch(`${getBaseUrl()}/admin/accounts/import-kiro`, {
      method: 'POST',
      headers: getHeaders(),
    })
    if (!res.ok) {
      const text = await res.text()
      throw new Error(`导入失败: ${res.status} - ${text}`)
    }
    return res.json()
  },

  // 删除账号
  async deleteAccount(id: string): Promise<void> {
    const res = await fetch(`${getBaseUrl()}/admin/accounts/${id}`, {
      method: 'DELETE',
      headers: getHeaders(),
    })
    if (!res.ok) {
      throw new Error(`删除失败: ${res.status}`)
    }
  },

  // 获取账号配额（返回 Kiro API 原始响应）
  async getQuota(id: string): Promise<any> {
    const res = await fetch(`${getBaseUrl()}/admin/quota/${id}`, {
      headers: getHeaders(),
    })
    if (!res.ok) {
      throw new Error(`获取配额失败: ${res.status}`)
    }
    return res.json()
  },

  // 添加账号（JSON 导入）
  async addAccount(account: {
    provider: string
    authMethod: string
    accessToken?: string
    refreshToken: string
    region?: string
    clientId?: string
    clientSecret?: string
  }): Promise<{ accountId: string }> {
    const res = await fetch(`${getBaseUrl()}/admin/accounts`, {
      method: 'POST',
      headers: getHeaders(),
      body: JSON.stringify({
        provider: account.provider,
        authMethod: account.authMethod,
        accessToken: account.accessToken,
        refreshToken: account.refreshToken,
        region: account.region || 'us-east-1',
        clientId: account.clientId,
        clientSecret: account.clientSecret,
      }),
    })
    if (!res.ok) {
      const text = await res.text()
      throw new Error(`添加失败: ${res.status} - ${text}`)
    }
    return res.json()
  },

  // 聊天（流式）
  async chat(
    messages: { role: string; content: string }[],
    model: string,
    onChunk: (chunk: string) => void
  ): Promise<void> {
    const res = await fetch(`${getBaseUrl()}/v1/chat/completions`, {
      method: 'POST',
      headers: getHeaders(),
      body: JSON.stringify({
        model: model,  // 直接发送模型名，不加 qdev:: 前缀
        messages,
        stream: true,
      }),
    })

    if (!res.ok) {
      const text = await res.text()
      throw new Error(`请求失败: ${res.status} - ${text}`)
    }

    const reader = res.body?.getReader()
    if (!reader) {
      throw new Error('无法读取响应流')
    }

    const decoder = new TextDecoder()
    let buffer = ''

    while (true) {
      const { done, value } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })
      const lines = buffer.split('\n')
      buffer = lines.pop() || ''

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6)
          if (data === '[DONE]') continue

          try {
            const json = JSON.parse(data)
            const content = json.choices?.[0]?.delta?.content
            if (content) {
              onChunk(content)
            }
          } catch {
            // 忽略解析错误
          }
        }
      }
    }
  },
}
