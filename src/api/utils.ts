export async function fetchWithTimeout(
  url: string,
  options: RequestInit = {},
  timeout = 10000
): Promise<Response> {
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
