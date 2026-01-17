import { useEffect, useState } from 'react'
import { isTauri, checkBackendStatus } from '../utils/tauri'

export default function BackendStatus() {
  const [isRunning, setIsRunning] = useState(false)
  const [checking, setChecking] = useState(true)

  useEffect(() => {
    if (!isTauri()) {
      setChecking(false)
      setIsRunning(true) // Web 模式假设后端始终运行
      return
    }

    // 定期检查后端状态
    const checkStatus = async () => {
      try {
        const status = await checkBackendStatus()
        setIsRunning(status)
      } catch (error) {
        console.error('Failed to check backend status:', error)
        setIsRunning(false)
      } finally {
        setChecking(false)
      }
    }

    checkStatus()
    const interval = setInterval(checkStatus, 5000) // 每 5 秒检查一次

    return () => clearInterval(interval)
  }, [])

  if (!isTauri()) {
    return null // Web 模式不显示后端状态
  }

  if (checking) {
    return (
      <div className="fixed top-4 right-4 px-3 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400">
        🔄 检查中...
      </div>
    )
  }

  return (
    <div className={`fixed top-4 right-4 px-3 py-1 rounded-full text-xs font-medium ${
      isRunning 
        ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' 
        : 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'
    }`}>
      {isRunning ? '✅ 后端运行中' : '❌ 后端未运行'}
    </div>
  )
}
