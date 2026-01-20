import { useState, lazy, Suspense } from 'react'
import { AppShell, NavLink, Group, Text, rem, Divider, Loader, Center, Burger, Container } from '@mantine/core'
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
  const [currentPage, setCurrentPage] = useState<Page>(() => {
    // 从 localStorage 读取上次访问的页面
    const saved = localStorage.getItem('currentPage')
    return (saved as Page) || 'accounts'
  })
  const [navbarOpened, setNavbarOpened] = useState(true)

  // 切换页面并保存到 localStorage
  const handlePageChange = (page: Page) => {
    setCurrentPage(page)
    localStorage.setItem('currentPage', page)
  }

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
          minHeight: '100vh',
          paddingTop: rem(80), // 为 fixed header 留出空间
        },
        navbar: {
          background: 'var(--kiro-nav-bg)',
          backdropFilter: 'var(--kiro-glass-blur)',
          borderRight: '1px solid var(--kiro-card-border)',
          boxShadow: 'var(--kiro-card-shadow)',
        },
      }}
    >
      <div
        style={{
          position: 'fixed',
          top: rem(16),
          left: navbarOpened ? rem(220 + 24) : rem(70 + 24),
          zIndex: 1000,
          transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
          background: 'var(--kiro-card-bg)',
          borderRadius: rem(12),
          padding: rem(8),
          boxShadow: 'var(--kiro-card-shadow)',
          backdropFilter: 'var(--kiro-glass-blur)',
          border: '1px solid var(--kiro-card-border)',
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
                background: 'var(--kiro-primary-gradient)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                boxShadow: '0 8px 24px rgba(99, 102, 241, 0.3)',
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
                onClick={() => handlePageChange(item.id)}
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
                onClick={() => handlePageChange(item.id)}
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
                background: 'rgba(16, 185, 129, 0.08)',
                border: '1px solid rgba(16, 185, 129, 0.2)',
                backdropFilter: 'var(--kiro-glass-blur)',
                boxShadow: 'var(--kiro-card-shadow)',
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
          <Container size="1400px" px={rem(4)} pb="xl">
            <div
              key={currentPage}
              className="animate-fade-in"
            >
              {renderPage()}
            </div>
          </Container>
        </Suspense>
      </AppShell.Main>
    </AppShell>
  )
}
