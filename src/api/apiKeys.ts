import { fetchWithTimeout } from './utils'

const API_BASE = '/admin'

export interface ApiKey {
  id: string
  key: string
  name: string | null
  createdAt: number
  lastUsed?: number | null
  enabled: boolean
}

export const apiKeysApi = {
  list: async (): Promise<ApiKey[]> => {
    const res = await fetchWithTimeout(`${API_BASE}/api-keys`)
    const data = await res.json()
    return data.keys || []
  },

  generate: async (name: string): Promise<ApiKey> => {
    const res = await fetchWithTimeout(`${API_BASE}/api-keys`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    })
    const data = await res.json()
    return data.key
  },

  update: async (id: string, updates: Partial<ApiKey>): Promise<void> => {
    await fetchWithTimeout(`${API_BASE}/api-keys/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(updates),
    })
  },

  delete: async (id: string): Promise<void> => {
    await fetchWithTimeout(`${API_BASE}/api-keys/${id}`, { method: 'DELETE' })
  },
}
