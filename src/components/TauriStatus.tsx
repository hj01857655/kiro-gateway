import { useEffect, useState } from 'react'
import { Badge } from '@mantine/core'

// 检测是否在 Tauri 环境中运行
const isTauri = () => {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

export default function TauriStatus() {
  const [mode, setMode] = useState<'web' | 'desktop'>('web')

  useEffect(() => {
    setMode(isTauri() ? 'desktop' : 'web')
  }, [])

  return (
    <Badge
      variant="light"
      color={mode === 'desktop' ? 'violet' : 'blue'}
      size="lg"
      radius="xl"
      style={{
        position: 'fixed',
        bottom: '1rem',
        right: '1rem',
      }}
    >
      {mode === 'desktop' ? '🖥️ 桌面版' : '🌐 Web 版'}
    </Badge>
  )
}
