import { Routes, Route, NavLink } from 'react-router-dom'
import { useState, useEffect } from 'react'
import Accounts from './pages/Accounts'
import Chat from './pages/Chat'
import Settings from './pages/Settings'
import Metrics from './pages/Metrics'
import Logs from './pages/Logs'
import TauriStatus from './components/TauriStatus'
import BackendStatus from './components/BackendStatus'

function App() {
  const [darkMode, setDarkMode] = useState(() => {
    return localStorage.getItem('theme') === 'dark'
  })

  useEffect(() => {
    document.documentElement.classList.toggle('dark', darkMode)
    localStorage.setItem('theme', darkMode ? 'dark' : 'light')
  }, [darkMode])

  return (
    <div className="min-h-screen flex">
      {/* 侧边栏 */}
      <aside className="w-56 bg-[hsl(var(--card))] border-r border-[hsl(var(--border))] p-4 flex flex-col">
        <h1 className="text-xl font-bold mb-6 text-[hsl(var(--primary))]">kiro-gateway</h1>
        <nav className="flex-1 space-y-2">
          <NavLink
            to="/"
            className={({ isActive }) =>
              `block px-3 py-2 rounded-md transition-colors ${
                isActive
                  ? 'bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]'
                  : 'hover:bg-[hsl(var(--muted))]'
              }`
            }
          >
            📊 账号管理
          </NavLink>
          <NavLink
            to="/metrics"
            className={({ isActive }) =>
              `block px-3 py-2 rounded-md transition-colors ${
                isActive
                  ? 'bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]'
                  : 'hover:bg-[hsl(var(--muted))]'
              }`
            }
          >
            📈 统计监控
          </NavLink>
          <NavLink
            to="/logs"
            className={({ isActive }) =>
              `block px-3 py-2 rounded-md transition-colors ${
                isActive
                  ? 'bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]'
                  : 'hover:bg-[hsl(var(--muted))]'
              }`
            }
          >
            📝 日志查看
          </NavLink>
          <NavLink
            to="/chat"
            className={({ isActive }) =>
              `block px-3 py-2 rounded-md transition-colors ${
                isActive
                  ? 'bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]'
                  : 'hover:bg-[hsl(var(--muted))]'
              }`
            }
          >
            💬 聊天测试
          </NavLink>
          <NavLink
            to="/settings"
            className={({ isActive }) =>
              `block px-3 py-2 rounded-md transition-colors ${
                isActive
                  ? 'bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]'
                  : 'hover:bg-[hsl(var(--muted))]'
              }`
            }
          >
            ⚙️ 设置
          </NavLink>
        </nav>
        <button
          onClick={() => setDarkMode(!darkMode)}
          className="mt-auto px-3 py-2 rounded-md hover:bg-[hsl(var(--muted))] transition-colors"
        >
          {darkMode ? '🌞 浅色模式' : '🌙 深色模式'}
        </button>
      </aside>

      {/* 主内容区 */}
      <main className="flex-1 p-6 overflow-auto">
        <Routes>
          <Route path="/" element={<Accounts />} />
          <Route path="/metrics" element={<Metrics />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/chat" element={<Chat />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>

      {/* 状态指示器 */}
      <BackendStatus />
      <TauriStatus />
    </div>
  )
}

export default App
