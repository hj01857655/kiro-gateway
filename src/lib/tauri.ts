import { invoke } from '@tauri-apps/api/core'

export async function tauriFetch(
  path: string,
  options: RequestInit = {}
): Promise<Response> {
  const method = options.method || 'GET'
  const body = options.body ? String(options.body) : undefined

  try {
    const responseText = await invoke<string>('proxy_request', {
      method,
      path,
      body,
    })

    // 将字符串响应转换为 Response 对象
    return new Response(responseText, {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    })
  } catch (error) {
    // Tauri invoke 错误
    throw new Error(String(error))
  }
}
