const API_BASE = '/admin'

export interface ApiKey {
  id: string
  key: string
  name: string
  created_at: number
  enabled: boolean
}

export const apiKeysApi = {
  list: async (): Promise<ApiKey[]> => {
    const res = await fetch(`${API_BASE}/api-keys`)
    if (!res.ok) throw new Error('获取 API Keys 失败')
    const data = await res.json()
    return data.keys || []
  },

  generate: async (name: string): Promise<ApiKey> => {
    const res = await fetch(`${API_BASE}/api-keys`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    })
    if (!res.ok) throw new Error('生成 API Key 失败')
    const data = await res.json()
    return data.key
  },

  update: async (id: string, updates: Partial<ApiKey>): Promise<void> => {
    const res = await fetch(`${API_BASE}/api-keys/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(updates),
    })
    if (!res.ok) throw new Error('更新 API Key 失败')
  },

  delete: async (id: string): Promise<void> => {
    const res = await fetch(`${API_BASE}/api-keys/${id}`, { method: 'DELETE' })
    if (!res.ok) throw new Error('删除 API Key 失败')
  },
}
