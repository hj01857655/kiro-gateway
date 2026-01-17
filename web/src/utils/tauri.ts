// Tauri 环境检测和 API 封装

// 检测是否在 Tauri 环境中运行
export const isTauri = () => {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

// 获取后端 API 基础 URL
export const getApiBaseUrl = (): string => {
  if (isTauri()) {
    // Tauri 桌面应用：使用本地后端
    return 'http://127.0.0.1:8080'
  } else {
    // Web 应用：使用当前域名
    return window.location.origin
  }
}

// Tauri 命令调用封装
export const tauriInvoke = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
  if (!isTauri()) {
    throw new Error('Tauri API is not available in web mode')
  }
  
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}

// 启动后端服务器（仅 Tauri）
export const startBackendServer = async (): Promise<string> => {
  if (!isTauri()) {
    return 'Web mode: backend server not needed'
  }
  return tauriInvoke<string>('start_backend_server')
}

// 停止后端服务器（仅 Tauri）
export const stopBackendServer = async (): Promise<string> => {
  if (!isTauri()) {
    return 'Web mode: backend server not needed'
  }
  return tauriInvoke<string>('stop_backend_server')
}

// 检查后端服务器状态（仅 Tauri）
export const checkBackendStatus = async (): Promise<boolean> => {
  if (!isTauri()) {
    return true // Web 模式假设后端始终运行
  }
  return tauriInvoke<boolean>('check_backend_status')
}

// 读取 Kiro 凭证文件（仅 Tauri）
export const readKiroCredentials = async (): Promise<string> => {
  if (!isTauri()) {
    throw new Error('Credentials reading is only available in desktop mode')
  }
  return tauriInvoke<string>('read_kiro_credentials')
}

// 检查凭证文件是否存在（仅 Tauri）
export const checkCredentialsExists = async (): Promise<boolean> => {
  if (!isTauri()) {
    return false
  }
  return tauriInvoke<boolean>('check_credentials_exists')
}

// 监听凭证文件变更事件（仅 Tauri）
export const onCredentialsChanged = async (callback: () => void) => {
  if (!isTauri()) {
    return
  }
  
  const { listen } = await import('@tauri-apps/api/event')
  return listen('credentials-changed', () => {
    callback()
  })
}
