import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

interface UpdateInfo {
  available: boolean
  currentVersion: string
  newVersion?: string
  body?: string
}

export function useAutoUpdate() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null)
  const [isChecking, setIsChecking] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // 检查更新
  const checkForUpdates = async () => {
    setIsChecking(true)
    setError(null)

    try {
      // 开发版本不检查更新
      if (import.meta.env.DEV) {
        setUpdateInfo({
          available: false,
          currentVersion: 'dev',
        })
        return false
      }

      // 获取当前版本
      const currentVersion = await invoke<string>('get_app_version')

      // 检查是否有新版本
      const update = await check()

      // update 可能为 null（latest.json 不存在或格式错误）
      if (update && update.available) {
        setUpdateInfo({
          available: true,
          currentVersion,
          newVersion: update.version,
          body: update.body,
        })
        return true
      } else {
        setUpdateInfo({
          available: false,
          currentVersion,
        })
        return false
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      setError(errorMsg)
      console.error('检查更新失败:', err)
      return false
    } finally {
      setIsChecking(false)
    }
  }

  // 安装更新
  const installUpdate = async () => {
    try {
      const update = await check()
      // update 可能为 null，需要检查
      if (update && update.available) {
        await update.downloadAndInstall()
        // 安装完成后重启应用
        await relaunch()
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      setError(errorMsg)
      console.error('安装更新失败:', err)
    }
  }

  // 组件挂载时自动检查更新（可选）
  useEffect(() => {
    // 延迟 3 秒后检查更新，避免应用启动时卡顿
    const timer = setTimeout(() => {
      checkForUpdates()
    }, 3000)

    return () => clearTimeout(timer)
  }, [])

  return {
    updateInfo,
    isChecking,
    error,
    checkForUpdates,
    installUpdate,
  }
}
