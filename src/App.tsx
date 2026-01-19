import { useState, lazy, Suspense } from 'react'
import { AppShell, NavLink, Group, Text, rem, Divider, Loader, Center } from '@mantine/core'
import { Users, BarChart3, FileText, Settings, MessageSquare, Info } from 'lucide-react'
import { useThemeStore } from './stores/themeStore'

const Accounts = lazy(() => import('./pages/Accounts'))
const Metrics = lazy(() => import('./pages/Metrics'))
const Logs = lazy(() => import('./pages/Logs'))
const SettingsPage = lazy(() => import('./pages/Settings'))
const Chat = lazy(() => import('./pages/Chat'))
const About = lazy(() => import('./pages/About'))

type Page = 'accounts' | 'metrics' | 'logs' | 'settings' | 'chat' | 'about'

export default function App() {
  const colorScheme = useThemeStore((state) => state.colorScheme)
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
          background: colorScheme === 'dark'
            ? 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)'
            : 'linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%)',
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
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, #7c3aed 0%, #a855f7 100%)'
                  : 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
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
          {navigation.slice(0, 4).map((item) => {
            const Icon = item.icon
            return (
              <NavLink
                key={item.id}
                active={currentPage === item.id}
                label={item.name}
                leftSection={<Icon size={20} />}
                onClick={() => setCurrentPage(item.id)}
                color={item.color}
                variant="subtle"
                mb={4}
                styles={{
                  root: {
                    borderRadius: rem(8),
                    fontWeight: 500,
                    padding: `${rem(10)} ${rem(12)}`,
                  },
                  label: {
                    fontSize: rem(14),
                  },
                }}
              />
            )
          })}
          <Divider my="sm" />
          {navigation.slice(4).map((item) => {
            const Icon = item.icon
            return (
              <NavLink
                key={item.id}
                active={currentPage === item.id}
                label={item.name}
                leftSection={<Icon size={20} />}
                onClick={() => setCurrentPage(item.id)}
                color={item.color}
                variant="subtle"
                mb={4}
                styles={{
                  root: {
                    borderRadius: rem(8),
                    fontWeight: 500,
                    padding: `${rem(10)} ${rem(12)}`,
                  },
                  label: {
                    fontSize: rem(14),
                  },
                }}
              />
            )
          })}
        </AppShell.Section>

        <AppShell.Section>
          <div
            style={{
              padding: rem(12),
              borderRadius: rem(12),
              background: colorScheme === 'dark'
                ? 'rgba(16, 185, 129, 0.1)'
                : 'rgba(16, 185, 129, 0.08)',
              border: `1px solid ${colorScheme === 'dark' ? 'rgba(16, 185, 129, 0.2)' : 'rgba(16, 185, 129, 0.15)'}`,
              backdropFilter: 'blur(10px)',
            }}
          >
            <Group gap="xs" wrap="nowrap">
              <div
                style={{
                  width: rem(8),
                  height: rem(8),
                  borderRadius: '50%',
                  background: '#10b981',
                  boxShadow: '0 0 8px rgba(16, 185, 129, 0.6)',
                  animation: 'pulse 2s ease-in-out infinite',
                }}
              />
              <div style={{ flex: 1 }}>
                <Text size="xs" fw={600} c={colorScheme === 'dark' ? 'green.4' : 'green.7'}>
                  运行中
                </Text>
                <Text size="xs" c="dimmed" style={{ fontFamily: 'monospace' }}>
                  127.0.0.1:8080
                </Text>
              </div>
            </Group>
          </div>
        </AppShell.Section>
      </AppShell.Navbar>

      <AppShell.Main>
        <Suspense fallback={<Center h={400}><Loader size="lg" /></Center>}>
          <div key={currentPage} style={{ animation: 'fadeIn 0.2s ease-in' }}>
            {renderPage()}
          </div>
        </Suspense>
      </AppShell.Main>
    </AppShell>
  )
}
