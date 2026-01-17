import { useEffect, useState } from 'react'

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
    <div className="fixed bottom-4 right-4 px-3 py-1 rounded-full text-xs font-medium bg-muted text-muted-foreground border border-border">
      {mode === 'desktop' ? '🖥️ 桌面版' : '🌐 Web 版'}
    </div>
  )
}
