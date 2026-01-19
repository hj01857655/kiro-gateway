import { useState } from 'react'
import { AppShell, NavLink, Group, Text, Badge, rem } from '@mantine/core'
import { Users, BarChart3, FileText, Settings, MessageSquare, Info } from 'lucide-react'
import Accounts from './pages/Accounts'
import Metrics from './pages/Metrics'
import Logs from './pages/Logs'
import SettingsPage from './pages/Settings'
import Chat from './pages/Chat'
import About from './pages/About'

type Page = 'accounts' | 'metrics' | 'logs' | 'settings' | 'chat' | 'about'

export default function App() {
  const [currentPage, setCurrentPage] = useState<Page>('accounts')

  const navigation = [
    { id: 'accounts' as Page, name: '账号管理', icon: Users, color: 'violet' },
    { id: 'metrics' as Page, name: '统计监控', icon: BarChart3, color: 'blue' },
    { id: 'logs' as Page, name: '日志查看', icon: FileText, color: 'teal' },
    { id: 'chat' as Page, name: '聊天测试', icon: MessageSquare, color: 'orange' },
    { id: 'settings' as Page, name: '设置', icon: Settings, color: 'gray' },
    { id: 'about' as Page, name: '关于', icon: Info, color: 'cyan' },
  ]

  const renderPage = () => {
    switch (currentPage) {
      case 'accounts':
        return <Accounts />
      case 'metrics':
        return <Metrics />
      case 'logs':
        return <Logs />
      case 'settings':
        return <SettingsPage />
      case 'chat':
        return <Chat />
      case 'about':
        return <About />
      default:
        return <Accounts />
    }
  }

  return (
    <AppShell
      navbar={{ width: 280, breakpoint: 'sm' }}
      padding="md"
      styles={{
        main: {
          background: 'linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%)',
        },
      }}
    >
      <AppShell.Navbar p="md">
        <AppShell.Section>
          <Group mb="xl">
            <div
              style={{
                width: rem(40),
                height: rem(40),
                borderRadius: rem(12),
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Text c="white" fw={700} size="xl">
                K
              </Text>
            </div>
            <div>
              <Text fw={700} size="lg">
                Kiro Gateway
              </Text>
              <Text size="xs" c="dimmed">
                API 网关管理
              </Text>
            </div>
          </Group>
        </AppShell.Section>

        <AppShell.Section grow>
          {navigation.map((item) => {
            const Icon = item.icon
            return (
              <NavLink
                key={item.id}
                active={currentPage === item.id}
                label={item.name}
                leftSection={<Icon size={20} />}
                onClick={() => setCurrentPage(item.id)}
                color={item.color}
                variant="filled"
                mb="xs"
              />
            )
          })}
        </AppShell.Section>

        <AppShell.Section>
          <Group
            p="sm"
            style={{
              borderRadius: rem(8),
              background: 'linear-gradient(135deg, #d4fc79 0%, #96e6a1 100%)',
            }}
          >
            <Badge color="green" variant="dot" size="lg">
              运行中
            </Badge>
            <Text size="xs" fw={500}>
              127.0.0.1:8080
            </Text>
          </Group>
        </AppShell.Section>
      </AppShell.Navbar>

      <AppShell.Main>{renderPage()}</AppShell.Main>
    </AppShell>
  )
}
