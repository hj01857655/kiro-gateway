import { tauriFetch } from '../lib/tauri'

// 检测是否在 Tauri 环境
const isTauri = '__TAURI_INTERNALS__' in window

export async function fetchWithTimeout(
  url: string,
  options: RequestInit = {},
  timeout = 10000
): Promise<Response> {
  // 生产环境使用 Tauri invoke
  if (isTauri) {
    // 提取路径（移除 base URL）
    const path = url.replace(/^https?:\/\/[^/]+/, '')
    return tauriFetch(path, options)
  }

  // 开发环境使用原生 fetch
  const controller = new AbortController()
  const id = setTimeout(() => controller.abort(), timeout)

  try {
    const response = await fetch(url, {
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
