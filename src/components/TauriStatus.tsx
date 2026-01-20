import { useState, useEffect } from 'react'
import { Badge, Group, Tooltip } from '@mantine/core'
import { useAccountStore } from '../stores/accountStore'

// 检测是否在 Tauri 环境中运行
const isTauri = () => {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

// 计算距离过期时间的剩余天数
const getDaysUntilExpiry = (expiresAt: string | undefined): number | null => {
  if (!expiresAt) return null
  try {
    const expireDate = new Date(expiresAt)
    const now = new Date()
    const diffMs = expireDate.getTime() - now.getTime()
    const diffDays = Math.ceil(diffMs / (1000 * 60 * 60 * 24))
    return diffDays
  } catch {
    return null
  }
}

// 获取过期状态颜色
const getExpiryColor = (daysLeft: number | null): string => {
  if (daysLeft === null) return 'gray'
  if (daysLeft <= 0) return 'red'
  if (daysLeft <= 7) return 'orange'
  if (daysLeft <= 30) return 'yellow'
  return 'green'
}

export default function TauriStatus() {
  const [mode] = useState<'web' | 'desktop'>(() => (isTauri() ? 'desktop' : 'web'))
  const [daysLeft, setDaysLeft] = useState<number | null>(null)
  const accounts = useAccountStore((state) => state.accounts)

  // 计算最近过期的账号的剩余天数
  useEffect(() => {
    if (!accounts || accounts.length === 0) {
      setDaysLeft(null)
      return
    }

    // 找到最近过期的账号
    let minDaysLeft: number | null = null
    for (const account of accounts) {
      if (account.expiresAt) {
        const days = getDaysUntilExpiry(account.expiresAt)
        if (days !== null) {
          if (minDaysLeft === null || days < minDaysLeft) {
            minDaysLeft = days
          }
        }
      }
    }
    setDaysLeft(minDaysLeft)
  }, [accounts])

  const expiryColor = getExpiryColor(daysLeft)
  const expiryText = daysLeft === null ? '无过期信息' : daysLeft <= 0 ? '已过期' : `${daysLeft}天后过期`

  return (
    <Group
      gap="xs"
      style={{
        position: 'fixed',
        bottom: '1rem',
        right: '1rem',
        zIndex: 1000,
      }}
    >
      {/* Token 过期时间显示 */}
      {daysLeft !== null && (
        <Tooltip label={`最近账号过期时间：${expiryText}`} position="top">
          <Badge
            variant="light"
            color={expiryColor}
            size="lg"
            radius="xl"
          >
            ⏱️ {expiryText}
          </Badge>
        </Tooltip>
      )}

      {/* Tauri 模式显示 */}
      <Badge
        variant="light"
        color={mode === 'desktop' ? 'violet' : 'blue'}
        size="lg"
        radius="xl"
      >
        {mode === 'desktop' ? '🖥️ 桌面版' : '🌐 Web 版'}
      </Badge>
    </Group>
  )
}
