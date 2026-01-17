import { useEffect, useState } from 'react'
import { isTauri } from '../utils/tauri'

export default function TauriStatus() {
  const [mode, setMode] = useState<'web' | 'desktop'>('web')

  useEffect(() => {
    setMode(isTauri() ? 'desktop' : 'web')
  }, [])

  return (
    <div className="fixed bottom-4 right-4 px-3 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400">
      {mode === 'desktop' ? '🖥️ 桌面版' : '🌐 Web 版'}
    </div>
  )
}
