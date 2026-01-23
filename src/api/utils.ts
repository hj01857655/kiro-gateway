import { tauriFetch } from '../lib/tauri'

// 检测是否在 Tauri 环境
const isTauri = '__TAURI_INTERNALS__' in window

export async function fetchWithTimeout(
  url: string,
  options: RequestInit = {},
  timeout = 10000
): Promise<Response> {
  // Tauri 环境使用 invoke 代理请求
  if (isTauri) {
    const path = url.replace(/^https?:\/\/[^/]+/, '')
    return tauriFetch(path, options)
  }

  // 开发环境直接访问 Axum 服务器
  const API_BASE_URL = 'http://127.0.0.1:8080'
  const fullUrl = url.startsWith('http') ? url : `${API_BASE_URL}${url}`
  
  const controller = new AbortController()
  const id = setTimeout(() => controller.abort(), timeout)

  try {
    const response = await fetch(fullUrl, {
      ...options,
      signal: controller.signal,
    })
    clearTimeout(id)

    if (!response.ok) {
      let errorMessage = `请求失败 (${response.status})`
      try {
        const data = await response.json()
        if (data.error || data.message) {
          errorMessage = data.error || data.message
        }
      } catch {
        // 无法解析错误响应，使用默认消息
      }
      throw new Error(errorMessage)
    }

    return response
  } catch (error) {
    clearTimeout(id)
    if (error instanceof Error && error.name === 'AbortError') {
      throw new Error('请求超时')
    }
    throw error
  }
}
