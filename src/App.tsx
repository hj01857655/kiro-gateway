import { useState, lazy, Suspense } from 'react'
import { AppShell, NavLink, Group, Text, rem, Divider, Loader, Center, Burger } from '@mantine/core'
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
  const [navbarOpened, setNavbarOpened] = useState(true)

  const navigation = [
    { id: 'accounts' as Page, name: '账号管理', icon: Users, color: 'violet' },
    { id: 'metrics' as Page, name: '统计监控', icon: BarChart3, color: 'blue' },
    { id: 'logs' as Page, name: '日志查看', icon: FileText, color: 'teal' },
    { id: 'chat' as Page, name: '聊天测试', icon: MessageSquare, color: 'orange' },
    { id: 'settings' as Page, name: '设置', icon: Settings, color: 'gray' },
    { id: 'about' as Page, name: '关于', icon: Info, color: 'cyan' },
  ]

  const navLinkStyles = {
    root: {
      borderRadius: rem(12),
      fontWeight: 500,
      padding: navbarOpened ? `${rem(14)} ${rem(16)}` : rem(14),
      justifyContent: navbarOpened ? 'flex-start' : 'center',
      transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
      border: '1px solid transparent',
      '&:hover': {
        transform: navbarOpened ? 'translateX(4px)' : 'scale(1.1)',
        boxShadow: colorScheme === 'dark'
          ? '0 4px 12px rgba(99, 102, 241, 0.2)'
          : '0 4px 12px rgba(0, 0, 0, 0.08)',
      },
      '&[dataActive]': {
        background: colorScheme === 'dark'
          ? 'linear-gradient(135deg, rgba(99, 102, 241, 0.2) 0%, rgba(139, 92, 246, 0.15) 100%)'
          : 'linear-gradient(135deg, rgba(99, 102, 241, 0.12) 0%, rgba(139, 92, 246, 0.08) 100%)',
        borderColor: colorScheme === 'dark'
          ? 'rgba(99, 102, 241, 0.4)'
          : 'rgba(99, 102, 241, 0.2)',
        boxShadow: colorScheme === 'dark'
          ? '0 4px 16px rgba(99, 102, 241, 0.3)'
          : '0 4px 16px rgba(99, 102, 241, 0.15)',
      },
    },
    label: {
      fontSize: rem(14),
      fontWeight: 600,
    },
  }

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
      navbar={{
        width: navbarOpened ? 220 : 70,
        breakpoint: 'sm',
      }}
      padding="md"
      styles={{
        main: {
          background: colorScheme === 'dark'
            ? 'linear-gradient(135deg, #0a0b14 0%, #13141f 50%, #1a1b2e 100%)'
            : 'linear-gradient(135deg, #f0f4f8 0%, #d9e2ec 50%, #bcccdc 100%)',
          minHeight: '100vh',
        },
        navbar: {
          background: colorScheme === 'dark'
            ? 'linear-gradient(180deg, rgba(19, 20, 31, 0.95) 0%, rgba(13, 14, 25, 0.95) 100%)'
            : 'linear-gradient(180deg, rgba(255, 255, 255, 0.95) 0%, rgba(248, 250, 252, 0.95) 100%)',
          backdropFilter: 'blur(20px)',
          borderRight: colorScheme === 'dark'
            ? '1px solid rgba(99, 102, 241, 0.2)'
            : '1px solid rgba(0, 0, 0, 0.08)',
          boxShadow: colorScheme === 'dark'
            ? '4px 0 24px rgba(0, 0, 0, 0.5)'
            : '4px 0 24px rgba(0, 0, 0, 0.08)',
        },
      }}
    >
      <div
        style={{
          position: 'fixed',
          top: rem(16),
          left: navbarOpened ? rem(220 + 16) : rem(70 + 16),
          zIndex: 1000,
          transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
          background: colorScheme === 'dark' 
            ? 'linear-gradient(135deg, rgba(30, 30, 45, 0.95) 0%, rgba(25, 25, 40, 0.95) 100%)' 
            : 'linear-gradient(135deg, rgba(255, 255, 255, 0.98) 0%, rgba(248, 250, 252, 0.98) 100%)',
          borderRadius: rem(12),
          padding: rem(10),
          boxShadow: colorScheme === 'dark' 
            ? '0 8px 32px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(99, 102, 241, 0.3)' 
            : '0 8px 32px rgba(0, 0, 0, 0.12), 0 0 0 1px rgba(0, 0, 0, 0.05)',
          backdropFilter: 'blur(20px)',
        }}
      >
        <Burger opened={navbarOpened} onClick={() => setNavbarOpened(!navbarOpened)} size="sm" />
      </div>

      <AppShell.Navbar p="md">
        <AppShell.Section>
          <Group mb="xl" justify="center">
            <div
              style={{
                width: rem(48),
                height: rem(48),
                borderRadius: rem(14),
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 50%, #a855f7 100%)'
                  : 'linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                boxShadow: colorScheme === 'dark'
                  ? '0 8px 24px rgba(99, 102, 241, 0.4)'
                  : '0 8px 24px rgba(102, 126, 234, 0.3)',
                position: 'relative',
                overflow: 'hidden',
              }}
            >
              <div
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  right: 0,
                  bottom: 0,
                  background: 'linear-gradient(45deg, transparent 30%, rgba(255, 255, 255, 0.2) 50%, transparent 70%)',
                  animation: 'shimmer 3s infinite',
                }}
              />
              <Text c="white" fw={700} size="xl" style={{ position: 'relative', zIndex: 1 }}>
                K
              </Text>
            </div>
            {navbarOpened && (
              <div>
                <Text fw={700} size="lg" className="gradient-text">
                  Kiro Gateway
                </Text>
                <Text size="xs" c="dimmed">
                  API 网关管理
                </Text>
              </div>
            )}
          </Group>
        </AppShell.Section>

        <AppShell.Section grow mt="xs">
          {navigation.slice(0, 4).map((item) => {
            const Icon = item.icon
            return (
              <NavLink
                key={item.id}
                active={currentPage === item.id}
                label={navbarOpened ? item.name : ''}
                leftSection={<Icon size={navbarOpened ? 20 : 24} />}
                onClick={() => setCurrentPage(item.id)}
                color={item.color}
                variant="subtle"
                mb={6}
                styles={navLinkStyles}
              />
            )
          })}
          <Divider my="md" opacity={0.3} />
          {navigation.slice(4).map((item) => {
            const Icon = item.icon
            return (
              <NavLink
                key={item.id}
                active={currentPage === item.id}
                label={navbarOpened ? item.name : ''}
                leftSection={<Icon size={navbarOpened ? 20 : 24} />}
                onClick={() => setCurrentPage(item.id)}
                color={item.color}
                variant="subtle"
                mb={6}
                styles={navLinkStyles}
              />
            )
          })}
        </AppShell.Section>

        <AppShell.Section>
          {navbarOpened ? (
              <div
                style={{
                  padding: rem(14),
                  borderRadius: rem(14),
                  background: colorScheme === 'dark'
                    ? 'linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(5, 150, 105, 0.1) 100%)'
                    : 'linear-gradient(135deg, rgba(16, 185, 129, 0.12) 0%, rgba(5, 150, 105, 0.08) 100%)',
                  border: `1px solid ${colorScheme === 'dark' ? 'rgba(16, 185, 129, 0.3)' : 'rgba(16, 185, 129, 0.2)'}`,
                  backdropFilter: 'blur(10px)',
                  boxShadow: colorScheme === 'dark'
                    ? '0 4px 16px rgba(16, 185, 129, 0.2)'
                    : '0 4px 16px rgba(16, 185, 129, 0.15)',
                }}
              >
                <Group gap="xs" wrap="nowrap">
                  <div
                    style={{
                      width: rem(10),
                      height: rem(10),
                      borderRadius: '50%',
                      background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                      boxShadow: '0 0 12px rgba(16, 185, 129, 0.8), 0 0 24px rgba(16, 185, 129, 0.4)',
                      animation: 'pulse 2s ease-in-out infinite',
                    }}
                  />
                  <div style={{ flex: 1 }}>
                    <Text size="xs" fw={700} c={colorScheme === 'dark' ? 'green.3' : 'green.8'}>
                      运行中
                    </Text>
                    <Text size="xs" c="dimmed" style={{ fontFamily: 'monospace', fontWeight: 500 }}>
                      127.0.0.1:8080
                    </Text>
                  </div>
                </Group>
              </div>
          ) : (
            <div style={{ textAlign: 'center' }}>
              <div
                style={{
                  width: rem(10),
                  height: rem(10),
                  borderRadius: '50%',
                  background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                  boxShadow: '0 0 12px rgba(16, 185, 129, 0.8), 0 0 24px rgba(16, 185, 129, 0.4)',
                  animation: 'pulse 2s ease-in-out infinite',
                  margin: '0 auto',
                }}
              />
            </div>
          )}
        </AppShell.Section>
      </AppShell.Navbar>

      <AppShell.Main>
        <Suspense fallback={
          <Center h={400}>
            <div style={{ textAlign: 'center' }}>
              <Loader size="lg" type="dots" color="violet" />
              <Text size="sm" c="dimmed" mt="md">加载中...</Text>
            </div>
          </Center>
        }>
          <div 
            key={currentPage} 
            className="animate-fade-in"
            style={{ 
              padding: rem(4),
            }}
          >
            {renderPage()}
          </div>
        </Suspense>
      </AppShell.Main>
    </AppShell>
  )
}
