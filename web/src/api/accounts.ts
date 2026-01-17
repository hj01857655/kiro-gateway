const API_BASE = ''

async function request(url: string, options?: RequestInit) {
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  })
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }))
    throw new Error(error.error || error.message || 'Request failed')
  }
  return response.json()
}

export async function fetchAccounts() {
  return request(`${API_BASE}/admin/accounts`)
}

export async function deleteAccount(id: string) {
  return request(`${API_BASE}/admin/accounts/${id}`, { method: 'DELETE' })
}

export async function refreshAccount(id: string) {
  return request(`${API_BASE}/admin/accounts/${id}/refresh`, { method: 'POST' })
}

export async function enableAccount(id: string) {
  return request(`${API_BASE}/admin/accounts/${id}/enable`, { method: 'POST' })
}

export async function disableAccount(id: string) {
  return request(`${API_BASE}/admin/accounts/${id}/disable`, { method: 'POST' })
}

export async function importKiroAccount() {
  return request(`${API_BASE}/admin/accounts/import-kiro`, { method: 'POST' })
}

export async function addAccount(data: {
  refreshToken: string
  authMethod: string
  clientId?: string
  clientSecret?: string
  provider?: string
}) {
  return request(`${API_BASE}/admin/accounts`, {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

export async function fetchStats() {
  return request(`${API_BASE}/admin/stats`)
}

export async function fetchMetrics() {
  return request(`${API_BASE}/admin/metrics`)
}

export async function fetchLogs() {
  return request(`${API_BASE}/admin/logs`)
}

export async function clearLogs() {
  return request(`${API_BASE}/admin/logs/clear`, { method: 'POST' })
}

export async function fetchQuota(accountId: string) {
  return request(`${API_BASE}/admin/quota/${accountId}`)
}
