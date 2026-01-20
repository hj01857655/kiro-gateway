import { useState, useRef, useEffect } from 'react'
import { useAccounts } from '@/hooks/useAccounts'
import { useHealth, useCheckHealth } from '@/hooks/useHealth'
import { accountsApi } from '@/api/accounts'
import { fetchWithTimeout } from '@/api/utils'
import { logger } from '@/lib/logger'
import { getAccountKey } from '@/lib/utils'
import { maskToken } from '@/lib/masking'
import {
  Button,
  Card,
  Group,
  Text,
  Badge,
  Stack,
  Modal,
  TextInput,
  Select,
  Textarea,
  ActionIcon,
  Loader,
  Center,
  Title,
  Tabs,
  FileButton,
  Menu,
  Progress,
  Tooltip,
  PasswordInput,
  Code,
  rem,
} from '@mantine/core'
import { notifications } from '@mantine/notifications'
import {
  Plus,
  RefreshCw,
  Trash2,
  Power,
  PowerOff,
  Upload,
  ChevronDown,
  Activity,
  CreditCard,
  Users,
} from 'lucide-react'
import { format } from 'date-fns'
import type { Account, QuotaInfo } from '@/types'

export default function Accounts() {
  const { accounts, isLoading, addAccount, updateAccount, deleteAccount, refreshAccount } =
    useAccounts()
  const { data: healthData } = useHealth()
  const checkHealthMutation = useCheckHealth()
  const [showAddModal, setShowAddModal] = useState(false)
  const [activeTab, setActiveTab] = useState<string | null>('form')
  const resetRef = useRef<() => void>(null)
  const [quotaCache, setQuotaCache] = useState<Record<string, QuotaInfo>>({})
  const [loadingQuotas, setLoadingQuotas] = useState<Record<string, boolean>>({})

  const [formData, setFormData] = useState({
    name: '',
    authMethod: 'social',
    refreshToken: '',
    profileArn: '',
    region: 'us-east-1',
    clientId: '',
    clientSecret: '',
  })

  const [jsonInput, setJsonInput] = useState(`[
  {
    "id": "account-1",
    "name": "我的 Social 账号",
    "authMethod": "social",
    "refreshToken": "粘贴你的 refresh token",
    "profileArn": "",
    "region": "us-east-1",
    "enabled": true
  },
  {
    "id": "account-2",
    "name": "我的 IDC 账号",
    "authMethod": "idc",
    "refreshToken": "粘贴你的 refresh token",
    "profileArn": "arn:aws:codewhisperer:...",
    "clientId": "粘贴 client id",
    "clientSecret": "粘贴 client secret",
    "region": "us-east-1",
    "enabled": true
  }
]`)

  // 页面加载时自动获取所有账号的配额（只在账号 ID 列表变化时触发）
  useEffect(() => {
    if (accounts && accounts.length > 0) {
      // 批量获取配额，避免重复请求
      const accountsToFetch = accounts.filter(
        (account) => !quotaCache[account.id] && !loadingQuotas[account.id]
      )

      if (accountsToFetch.length > 0) {
        accountsToFetch.forEach((account) => {
          fetchQuota(account.id)
        })
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [accounts?.map(a => a.id).join(',')])

  // 获取配额（静默失败，使用缓存）
  const fetchQuota = async (accountId: string) => {
    if (loadingQuotas[accountId]) return

    setLoadingQuotas((prev) => ({ ...prev, [accountId]: true }))
    try {
      const quota = await accountsApi.getQuota(accountId)
      setQuotaCache((prev) => ({ ...prev, [accountId]: quota }))
    } catch (error) {
      // 静默失败，使用缓存的配额信息
      logger.warn(`获取账号 ${accountId} 配额失败:`, error)
    } finally {
      setLoadingQuotas((prev) => ({ ...prev, [accountId]: false }))
    }
  }

  // 刷新账号（同时刷新配额）
  const handleRefresh = async (accountId: string) => {
    try {
      await refreshAccount(accountId)
      // 刷新成功后立即获取配额
      await fetchQuota(accountId)
    } catch (error) {
      // refreshAccount 已经显示了错误通知
    }
  }

  const handleFormSubmit = async () => {
    if (!formData.name || !formData.refreshToken) {
      notifications.show({
        title: '错误',
        message: '请填写必填字段',
        color: 'red',
      })
      return
    }
    // 构造符合 Partial<Account> 类型的对象
    const accountData: Partial<Account> = {
      id: `${formData.authMethod}-${Date.now()}`, // 自动生成 ID
      name: formData.name,
      authMethod: formData.authMethod,
      refreshToken: formData.refreshToken,
      profileArn: formData.profileArn || '',
      region: formData.region || 'us-east-1',
      clientId: formData.clientId || undefined,
      clientSecret: formData.clientSecret || undefined,
      enabled: true,
    }
    
    try {
      await addAccount(accountData)
      setShowAddModal(false)
      setFormData({
        name: '',
        authMethod: 'social',
        refreshToken: '',
        profileArn: '',
        region: 'us-east-1',
        clientId: '',
        clientSecret: '',
      })
      notifications.show({
        title: '成功',
        message: '账号添加成功',
        color: 'green',
      })
    } catch (error) {
      notifications.show({
        title: '添加失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
    }
  }

  const handleJsonSubmit = async () => {
    try {
      const data = JSON.parse(jsonInput)

      if (!data || (Array.isArray(data) && data.length === 0)) {
        notifications.show({
          title: '错误',
          message: '导入数据不能为空',
          color: 'red',
        })
        return
      }

      // 批量导入（让后端处理去重和格式转换）
      if (Array.isArray(data)) {
        let successCount = 0
        let failCount = 0
        const errors: string[] = []

        for (const account of data) {
          try {
            await addAccount(account)
            successCount++
          } catch (error) {
            failCount++
            const errorMsg = error instanceof Error ? error.message : '未知错误'
            // 只记录前 3 个错误，避免通知过多
            if (errors.length < 3) {
              errors.push(errorMsg)
            }
          }
        }

        if (successCount > 0) {
          notifications.show({
            title: '导入完成',
            message: `成功导入 ${successCount} 个账号${failCount > 0 ? `，跳过 ${failCount} 个重复或无效账号` : ''}`,
            color: successCount === data.length ? 'green' : 'yellow',
          })
        } else {
          notifications.show({
            title: '导入失败',
            message: errors.length > 0 ? errors[0] : '所有账号都已存在或无效',
            color: 'red',
          })
        }
      } else {
        // 单个导入
        try {
          await addAccount(data)
          notifications.show({
            title: '成功',
            message: 'JSON 账号添加成功',
            color: 'green',
          })
        } catch (error) {
          notifications.show({
            title: '导入失败',
            message: error instanceof Error ? error.message : '未知错误',
            color: 'red',
          })
        }
      }
      setShowAddModal(false)
    } catch (error) {
      notifications.show({
        title: '错误',
        message: 'JSON 格式错误: ' + (error as Error).message,
        color: 'red',
      })
    }
  }

  const handleFileImport = async (file: File | null) => {
    if (!file) return

    const reader = new FileReader()
    reader.onload = async (e) => {
      try {
        const content = e.target?.result as string
        const data = JSON.parse(content)

        if (!data || (Array.isArray(data) && data.length === 0)) {
          notifications.show({
            title: '错误',
            message: '导入数据不能为空',
            color: 'red',
          })
          return
        }

        // 批量导入（让后端处理去重和格式转换）
        if (Array.isArray(data)) {
          let successCount = 0
          let failCount = 0
          const errors: string[] = []

          for (const account of data) {
            try {
              await addAccount(account)
              successCount++
            } catch (error) {
              failCount++
              const errorMsg = error instanceof Error ? error.message : '未知错误'
              // 只记录前 3 个错误，避免通知过多
              if (errors.length < 3) {
                errors.push(errorMsg)
              }
            }
          }

          if (successCount > 0) {
            notifications.show({
              title: '导入完成',
              message: `成功导入 ${successCount} 个账号${failCount > 0 ? `，跳过 ${failCount} 个重复或无效账号` : ''}`,
              color: successCount === data.length ? 'green' : 'yellow',
            })
          } else {
            notifications.show({
              title: '导入失败',
              message: errors.length > 0 ? errors[0] : '所有账号都已存在或无效',
              color: 'red',
            })
          }
        } else {
          // 单个导入
          try {
            await addAccount(data)
            notifications.show({
              title: '成功',
              message: '从文件导入账号成功',
              color: 'green',
            })
          } catch (error) {
            notifications.show({
              title: '导入失败',
              message: error instanceof Error ? error.message : '未知错误',
              color: 'red',
            })
          }
        }
        setShowAddModal(false)
      } catch (error) {
        notifications.show({
          title: '错误',
          message: '文件格式错误: ' + (error as Error).message,
          color: 'red',
        })
      }
    }
    reader.readAsText(file)
  }

  const handleToggle = (account: Account) => {
    const newStatus: Account['status'] = account.status === 'disabled' ? 'active' : 'disabled'
    updateAccount({
      id: account.id,
      updates: { status: newStatus },
    })
  }

  const getStatusColor = (status: string) => {
    const colors: Record<string, string> = {
      active: 'green',
      expired: 'red',
      throttled: 'yellow',
      error: 'red',
      disabled: 'gray',
      banned: 'red',
    }
    return colors[status] || 'gray'
  }

  if (isLoading) {
    return (
      <Center h={400}>
        <Loader size="lg" />
      </Center>
    )
  }

  // 获取账号的健康信息
  const getAccountHealth = (accountId: string) => {
    return healthData?.accounts.find((h) => h.id === accountId)
  }

  // 手动触发健康检查
  const handleCheckHealth = async () => {
    try {
      const result = await checkHealthMutation.mutateAsync()
      notifications.show({
        title: '健康检查完成',
        message: `检查了 ${result.checked} 个账号，有效 ${result.valid} 个，无效 ${result.invalid} 个`,
        color: 'green',
      })
    } catch (error) {
      notifications.show({
        title: '健康检查失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
    }
  }

  // 格式化配额显示
  const formatQuota = (quota: QuotaInfo | undefined) => {
    if (!quota || !quota.usageBreakdownList || quota.usageBreakdownList.length === 0) {
      return null
    }

    const usage = quota.usageBreakdownList[0]
    const total = usage.usageLimit + (usage.freeTrialInfo?.usageLimit || 0)
    const used = usage.currentUsage + (usage.freeTrialInfo?.currentUsage || 0)
    const percentage = total > 0 ? (used / total) * 100 : 0

    return { total, used, percentage, unit: usage.displayName }
  }

  // 从 Kiro IDE 导入账号
  const handleImportFromKiro = async () => {
    try {
      const res = await fetchWithTimeout('/admin/accounts/import', { method: 'POST' })

      const data = await res.json()
      if (!data.success || !data.accounts || data.accounts.length === 0) {
        notifications.show({
          title: '导入失败',
          message: data.message || '未找到 Kiro IDE 缓存文件',
          color: 'orange',
        })
        return
      }

      // 去重：使用账号唯一标识（只检查有 email+provider 的账号）
      const existingKeys = new Set(
        (accounts || [])
          .map(a => getAccountKey(a))
          .filter((key): key is string => key !== null)
      )
      const newAccounts = data.accounts.filter((account: Account) => {
        const key = getAccountKey(account)
        return key === null || !existingKeys.has(key)
      })

      if (newAccounts.length === 0) {
        notifications.show({
          title: '无需导入',
          message: '所有账号已存在，无需重复导入',
          color: 'blue',
        })
        return
      }

      // 添加新账号
      for (const account of newAccounts) {
        await addAccount(account)
      }

      const skippedCount = data.accounts.length - newAccounts.length
      notifications.show({
        title: '导入成功',
        message: `成功导入 ${newAccounts.length} 个账号${skippedCount > 0 ? `，跳过 ${skippedCount} 个重复账号` : ''}`,
        color: 'green',
      })
    } catch (error) {
      notifications.show({
        title: '导入失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
    }
  }

  return (
    <Stack gap="md" className="animate-fade-in">
      <Group justify="space-between">
        <Group>
          <div
            style={{
              width: rem(44),
              height: rem(44),
              borderRadius: rem(12),
              background: 'var(--kiro-primary-gradient)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 8px 16px rgba(99, 102, 241, 0.25)',
            }}
          >
            <Users size={24} color="white" />
          </div>
          <div>
            <Title order={2}>账号管理</Title>
            <Text size="sm" c="dimmed">
              管理你的 AI 账号权限与配额状态
            </Text>
          </div>
        </Group>
        <Group gap="sm">
          <Button
            variant="light"
            leftSection={<RefreshCw size={16} />}
            onClick={handleCheckHealth}
            loading={checkHealthMutation.isPending}
          >
            检查健康
          </Button>
          <Menu shadow="md" width={200}>
            <Menu.Target>
              <Button leftSection={<Plus size={16} />} rightSection={<ChevronDown size={16} />}>
                添加账号
              </Button>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item
                leftSection={<Plus size={16} />}
                onClick={() => {
                  setActiveTab('form')
                  setShowAddModal(true)
                }}
              >
                手动添加
              </Menu.Item>
              <Menu.Item
                leftSection={<Upload size={16} />}
                onClick={() => {
                  setActiveTab('import')
                  setShowAddModal(true)
                }}
              >
                批量导入
              </Menu.Item>
              <Menu.Item leftSection={<Upload size={16} />} onClick={handleImportFromKiro}>
                从 Kiro IDE 导入
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Group>
      </Group>

      {/* 健康状态概览 */}
      {healthData && (
        <Card withBorder className="glass-effect">
          <Group mb="md">
            <Activity size={20} color="var(--kiro-primary)" />
            <Text fw={600}>系统概览</Text>
          </Group>
          <Group grow>
            <Card
              withBorder
              p="sm"
              className="glass-effect"
              style={{ background: 'rgba(99, 102, 241, 0.05) !important' }}
            >
              <Text size="xs" c="dimmed">
                总账号数
              </Text>
              <Text size="xl" fw={700}>
                {healthData.total}
              </Text>
            </Card>
            <Card
              withBorder
              p="sm"
              className="glass-effect"
              style={{ background: 'rgba(16, 185, 129, 0.05) !important' }}
            >
              <Text size="xs" c="dimmed">
                可用账号
              </Text>
              <Text size="xl" fw={700} c="green">
                {healthData.available}
              </Text>
            </Card>
            <Card
              withBorder
              p="sm"
              className="glass-effect"
              style={{ background: 'rgba(250, 82, 82, 0.05) !important' }}
            >
              <Text size="xs" c="dimmed">
                不可用账号
              </Text>
              <Text size="xl" fw={700} c="red">
                {healthData.total - healthData.available}
              </Text>
            </Card>
            <Card withBorder p="sm" className="glass-effect">
              <Text size="xs" c="dimmed">
                可用率
              </Text>
              <Text size="xl" fw={700}>
                {healthData.total > 0 ? Math.round((healthData.available / healthData.total) * 100) : 0}%
              </Text>
              <Progress
                value={healthData.total > 0 ? (healthData.available / healthData.total) * 100 : 0}
                size="xs"
                mt="xs"
                color={healthData.available / healthData.total > 0.7 ? 'green' : 'orange'}
              />
            </Card>
          </Group>
        </Card>
      )}

      {(accounts || []).length === 0 ? (
        <Card shadow="sm" padding="xl" radius="md" withBorder>
          <Center h={200}>
            <Stack align="center" gap="md">
              <Text c="dimmed">暂无账号</Text>
              <Button leftSection={<Plus size={16} />} onClick={() => setShowAddModal(true)}>
                添加第一个账号
              </Button>
            </Stack>
          </Center>
        </Card>
      ) : (
        <Stack gap="md">
          {(accounts || []).map((account: Account) => {
            const health = getAccountHealth(account.id)
            const quota = formatQuota(quotaCache[account.id])
            const isLoadingQuota = loadingQuotas[account.id]

            return (
              <Card key={account.id} shadow="sm" padding="lg" radius="md" withBorder>
                <Stack gap="md">
                  {/* 顶部：名称和状态 */}
                  <Group justify="space-between" wrap="nowrap">
                    <Group gap="sm">
                      <Text fw={600} size="lg">
                        {account.name || account.id}
                      </Text>
                      <Badge color={getStatusColor(account.status)} variant="light" size="lg">
                        {account.status}
                      </Badge>
                      <Badge color="gray" variant="outline" size="sm">
                        {account.authMethod === 'social' ? 'social' : 'IDC'}
                      </Badge>
                    </Group>
                    <Group gap="xs">
                      <Tooltip label={account.status !== 'disabled' ? '禁用账号' : '启用账号'}>
                        <ActionIcon
                          variant="light"
                          color={account.status !== 'disabled' ? 'orange' : 'green'}
                          size="lg"
                          onClick={() => handleToggle(account)}
                        >
                          {account.status !== 'disabled' ? <PowerOff size={18} /> : <Power size={18} />}
                        </ActionIcon>
                      </Tooltip>
                      <Tooltip label="刷新 Token 和配额">
                        <ActionIcon
                          variant="light"
                          color="blue"
                          size="lg"
                          onClick={() => handleRefresh(account.id)}
                          loading={isLoadingQuota}
                        >
                          <RefreshCw size={18} />
                        </ActionIcon>
                      </Tooltip>
                      <Tooltip label="删除账号">
                        <ActionIcon
                          variant="light"
                          color="red"
                          size="lg"
                          onClick={() => deleteAccount(account.id)}
                        >
                          <Trash2 size={18} />
                        </ActionIcon>
                      </Tooltip>
                    </Group>
                  </Group>

                  {/* 中部：配额和健康状态 */}
                  <Group grow>
                    {/* 配额卡片 */}
                    {quota ? (
                      <Card
                        withBorder
                        p="sm"
                        radius="sm"
                        className="glass-effect"
                        style={{ background: 'rgba(99, 102, 241, 0.03) !important' }}
                      >
                        <Group gap="xs" mb={4}>
                          <CreditCard size={16} color="var(--kiro-primary)" />
                          <Text size="sm" fw={500}>
                            配额使用
                          </Text>
                        </Group>
                        <Group justify="space-between" align="flex-end">
                          <div>
                            <Text size="xl" fw={700}>
                              {quota.used}
                            </Text>
                            <Text size="xs" c="dimmed">
                              / {quota.total} {quota.unit}
                            </Text>
                          </div>
                          <Badge
                            size="lg"
                            color={quota.percentage > 80 ? 'red' : quota.percentage > 50 ? 'yellow' : 'green'}
                          >
                            {quota.percentage.toFixed(0)}%
                          </Badge>
                        </Group>
                        <Progress
                          value={quota.percentage}
                          size="sm"
                          mt="xs"
                          color={quota.percentage > 80 ? 'red' : quota.percentage > 50 ? 'yellow' : 'green'}
                        />
                      </Card>
                    ) : isLoadingQuota ? (
                      <Card withBorder p="sm" radius="sm" className="glass-effect">
                        <Center h={80}>
                          <Stack align="center" gap={4}>
                            <Loader size="sm" />
                            <Text size="xs" c="dimmed">
                              加载配额...
                            </Text>
                          </Stack>
                        </Center>
                      </Card>
                    ) : (
                      <Card withBorder p="sm" radius="sm" className="glass-effect">
                        <Group gap="xs" mb={4}>
                          <CreditCard size={16} />
                          <Text size="sm" fw={500}>
                            配额使用
                          </Text>
                        </Group>
                        <Center h={60}>
                          <Text size="xs" c="dimmed">
                            暂无数据
                          </Text>
                        </Center>
                      </Card>
                    )}

                    {/* 健康状态卡片 */}
                    {health ? (
                      <Card
                        withBorder
                        p="sm"
                        radius="sm"
                        className="glass-effect"
                        style={{ background: 'rgba(16, 185, 129, 0.03) !important' }}
                      >
                        <Group gap="xs" mb={4}>
                          <Activity size={16} color="#10b981" />
                          <Text size="sm" fw={500}>
                            健康状态
                          </Text>
                        </Group>
                        <Group justify="space-between" align="flex-end">
                          <div>
                            <Badge size="lg" color={health.is_available ? 'green' : 'red'} variant="dot">
                              {health.is_available ? '可用' : '不可用'}
                            </Badge>
                            <Text size="xs" c="dimmed" mt={4}>
                              成功率: {(health.success_rate * 100).toFixed(1)}%
                            </Text>
                          </div>
                          <Stack gap={0} align="flex-end">
                            <Text size="xs" c="dimmed">
                              成功: {health.success_count}
                            </Text>
                            <Text size="xs" c="dimmed">
                              失败: {health.fail_count}
                            </Text>
                          </Stack>
                        </Group>
                        {health.last_used && (
                          <Text size="xs" c="dimmed" mt="xs">
                            最后使用: {format(new Date(health.last_used), 'HH:mm:ss')}
                          </Text>
                        )}
                      </Card>
                    ) : (
                      <Card withBorder p="sm" radius="sm" className="glass-effect">
                        <Group gap="xs" mb={4}>
                          <Activity size={16} />
                          <Text size="sm" fw={500}>
                            健康状态
                          </Text>
                        </Group>
                        <Center h={60}>
                          <Text size="xs" c="dimmed">
                            暂无数据
                          </Text>
                        </Center>
                      </Card>
                    )}
                  </Group>

                  {/* 底部：详细信息 */}
                  <Stack gap={4}>
                    <Group gap="xs">
                      <Text size="xs" c="dimmed" style={{ fontFamily: 'monospace' }}>
                        ID: {account.id}
                      </Text>
                    </Group>
                    <Group gap="xs">
                      <Text size="xs" c="dimmed">Token:</Text>
                      <Tooltip label="点击复制完整 Token (功能待完善)">
                        <Code style={{ cursor: 'pointer', fontSize: rem(11) }}>{maskToken(account.refreshToken)}</Code>
                      </Tooltip>
                    </Group>
                    {account.clientSecret && (
                      <Group gap="xs">
                        <Text size="xs" c="dimmed">Secret:</Text>
                        <Code style={{ fontSize: rem(11) }}>{maskToken(account.clientSecret)}</Code>
                      </Group>
                    )}
                    {account.expiresAt && (
                      <Text size="xs" c="dimmed">
                        Token 过期: {format(new Date(account.expiresAt), 'yyyy-MM-dd HH:mm:ss')}
                      </Text>
                    )}
                  </Stack>
                </Stack>
              </Card>
            )
          })}
        </Stack>
      )}

      <Modal
        opened={showAddModal}
        onClose={() => setShowAddModal(false)}
        title="添加账号"
        size="lg"
        radius="md"
        styles={{
          title: { fontSize: rem(18), fontWeight: 600 },
        }}
      >
        <Tabs value={activeTab} onChange={setActiveTab} variant="pills">
          <Tabs.List grow mb="lg">
            <Tabs.Tab value="form" leftSection={<Plus size={16} />}>
              手动添加
            </Tabs.Tab>
            <Tabs.Tab value="import" leftSection={<Upload size={16} />}>
              批量导入
            </Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel value="form">
            <Stack gap="md">
              <TextInput
                label="账号名称"
                placeholder="例如: Google 账号 1"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                required
              />
              <Select
                label="账号类型"
                value={formData.authMethod}
                onChange={(value) => setFormData({ ...formData, authMethod: value || 'social' })}
                data={[
                  { value: 'social', label: 'Social (Google/GitHub)' },
                  { value: 'idc', label: 'IDC (Builder ID)' },
                ]}
                required
              />
              <PasswordInput
                label="Refresh Token"
                placeholder="粘贴 Refresh Token"
                value={formData.refreshToken}
                onChange={(e) => setFormData({ ...formData, refreshToken: e.target.value })}
                required
              />
              {formData.authMethod === 'idc' && (
                <>
                  <Select
                    label="Region"
                    placeholder="选择区域"
                    value={formData.region}
                    onChange={(value) => setFormData({ ...formData, region: value || 'us-east-1' })}
                    data={[
                      { value: 'us-east-1', label: 'US East (N. Virginia)' },
                      { value: 'us-west-2', label: 'US West (Oregon)' },
                      { value: 'eu-west-1', label: 'Europe (Ireland)' },
                      { value: 'ap-southeast-1', label: 'Asia Pacific (Singapore)' },
                      { value: 'ap-northeast-1', label: 'Asia Pacific (Tokyo)' },
                    ]}
                    required
                  />
                  <TextInput
                    label="Client ID"
                    placeholder="客户端 ID"
                    value={formData.clientId}
                    onChange={(e) => setFormData({ ...formData, clientId: e.target.value })}
                    required
                  />
                  <PasswordInput
                    label="Client Secret"
                    placeholder="客户端密钥"
                    value={formData.clientSecret}
                    onChange={(e) => setFormData({ ...formData, clientSecret: e.target.value })}
                    required
                  />
                </>
              )}
              <Group justify="flex-end" mt="md">
                <Button variant="light" onClick={() => setShowAddModal(false)}>
                  取消
                </Button>
                <Button onClick={handleFormSubmit}>添加</Button>
              </Group>
            </Stack>
          </Tabs.Panel>

          <Tabs.Panel value="import">
            <Stack gap="md">
              <Text size="sm" c="dimmed">
                支持 JSON 输入或文件上传，可导入单个账号或批量导入
              </Text>
              <FileButton resetRef={resetRef} onChange={handleFileImport} accept="application/json,.json">
                {(props) => (
                  <Button {...props} leftSection={<Upload size={16} />} variant="light" fullWidth>
                    选择 JSON 文件
                  </Button>
                )}
              </FileButton>
              <Text size="xs" c="dimmed" ta="center">
                或者直接粘贴 JSON 配置
              </Text>
              <Textarea
                placeholder="粘贴 JSON 配置..."
                value={jsonInput}
                onChange={(e) => setJsonInput(e.target.value)}
                minRows={12}
                maxRows={16}
                autosize
                styles={{
                  input: {
                    fontFamily: 'monospace',
                    fontSize: '0.85em',
                    lineHeight: '1.5',
                  },
                }}
              />
              <Group justify="flex-end" mt="md">
                <Button variant="light" onClick={() => setShowAddModal(false)}>
                  取消
                </Button>
                <Button onClick={handleJsonSubmit}>导入</Button>
              </Group>
            </Stack>
          </Tabs.Panel>
        </Tabs>
      </Modal>
    </Stack>
  )
}
