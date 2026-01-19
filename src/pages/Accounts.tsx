import { useState, useRef, useEffect } from 'react'
import { useAccounts } from '@/hooks/useAccounts'
import { useHealth, useCheckHealth } from '@/hooks/useHealth'
import { accountsApi } from '@/api/accounts'
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
  Code,
  Menu,
  Progress,
  Tooltip,
} from '@mantine/core'
import { notifications } from '@mantine/notifications'
import {
  Plus,
  RefreshCw,
  Trash2,
  Power,
  PowerOff,
  Upload,
  FileJson,
  ChevronDown,
  Activity,
  CreditCard,
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

  const [jsonInput, setJsonInput] = useState(`{
  "id": "account-1",
  "name": "我的账号",
  "authMethod": "social",
  "refreshToken": "粘贴你的 refresh token",
  "profileArn": "",
  "region": "us-east-1",
  "enabled": true
}

// 或者批量导入（数组格式）：
[
  {
    "id": "account-1",
    "name": "账号 1",
    "authMethod": "social",
    "refreshToken": "token1",
    "enabled": true
  },
  {
    "id": "account-2",
    "name": "账号 2",
    "authMethod": "idc",
    "refreshToken": "token2",
    "profileArn": "arn:aws:...",
    "clientId": "client-id",
    "clientSecret": "client-secret",
    "enabled": true
  }
]`)

  // 页面加载时自动获取所有账号的配额
  useEffect(() => {
    if (accounts && accounts.length > 0) {
      accounts.forEach((account) => {
        fetchQuota(account.id)
      })
    }
  }, [accounts?.length])

  // 获取配额（静默失败，使用缓存）
  const fetchQuota = async (accountId: string) => {
    if (loadingQuotas[accountId]) return
    
    setLoadingQuotas((prev) => ({ ...prev, [accountId]: true }))
    try {
      const quota = await accountsApi.getQuota(accountId)
      setQuotaCache((prev) => ({ ...prev, [accountId]: quota }))
    } catch (error) {
      // 静默失败，使用缓存的配额信息
      console.warn(`获取账号 ${accountId} 配额失败:`, error)
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

  const handleFormSubmit = () => {
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
      name: formData.name,
      authMethod: formData.authMethod,
      refreshToken: formData.refreshToken,
      profileArn: formData.profileArn,
      region: formData.region,
      clientId: formData.clientId,
      clientSecret: formData.clientSecret,
    }
    addAccount(accountData)
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
  }

  const handleJsonSubmit = () => {
    try {
      const data = JSON.parse(jsonInput)
      
      // 自动检测是单个对象还是数组
      if (Array.isArray(data)) {
        // 批量导入
        data.forEach((account) => addAccount(account))
        setShowAddModal(false)
        notifications.show({
          title: '成功',
          message: `批量导入 ${data.length} 个账号成功`,
          color: 'green',
        })
      } else {
        // 单个导入
        addAccount(data)
        setShowAddModal(false)
        notifications.show({
          title: '成功',
          message: 'JSON 账号添加成功',
          color: 'green',
        })
      }
    } catch (error) {
      notifications.show({
        title: '错误',
        message: 'JSON 格式错误: ' + (error as Error).message,
        color: 'red',
      })
    }
  }

  const handleFileImport = (file: File | null) => {
    if (!file) return

    const reader = new FileReader()
    reader.onload = (e) => {
      try {
        const content = e.target?.result as string
        const data = JSON.parse(content)

        if (Array.isArray(data)) {
          data.forEach((account) => addAccount(account))
          notifications.show({
            title: '成功',
            message: `从文件导入 ${data.length} 个账号成功`,
            color: 'green',
          })
        } else {
          addAccount(data)
          notifications.show({
            title: '成功',
            message: '从文件导入账号成功',
            color: 'green',
          })
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
      const res = await fetch('/admin/accounts/import', { method: 'POST' })
      if (!res.ok) throw new Error('导入失败')
      
      const data = await res.json()
      if (!data.success || !data.accounts || data.accounts.length === 0) {
        notifications.show({
          title: '导入失败',
          message: data.message || '未找到 Kiro IDE 缓存文件',
          color: 'orange',
        })
        return
      }

      // 添加导入的账号
      for (const account of data.accounts) {
        await addAccount(account)
      }

      notifications.show({
        title: '导入成功',
        message: `成功导入 ${data.accounts.length} 个账号`,
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
    <Stack gap="md">
      {/* 健康状态概览 */}
      {healthData && (
        <Card withBorder>
          <Group justify="space-between" mb="md">
            <Group>
              <Activity size={20} />
              <Text fw={600}>健康状态</Text>
            </Group>
            <Button
              size="xs"
              variant="light"
              leftSection={<RefreshCw size={14} />}
              onClick={handleCheckHealth}
              loading={checkHealthMutation.isPending}
            >
              检查健康
            </Button>
          </Group>
          <Group grow>
            <Card withBorder p="sm">
              <Text size="xs" c="dimmed">
                总账号数
              </Text>
              <Text size="xl" fw={700}>
                {healthData.total}
              </Text>
            </Card>
            <Card withBorder p="sm">
              <Text size="xs" c="dimmed">
                可用账号
              </Text>
              <Text size="xl" fw={700} c="green">
                {healthData.available}
              </Text>
            </Card>
            <Card withBorder p="sm">
              <Text size="xs" c="dimmed">
                不可用账号
              </Text>
              <Text size="xl" fw={700} c="red">
                {healthData.total - healthData.available}
              </Text>
            </Card>
            <Card withBorder p="sm">
              <Text size="xs" c="dimmed">
                可用率
              </Text>
              <Text size="xl" fw={700}>
                {healthData.total > 0
                  ? Math.round((healthData.available / healthData.total) * 100)
                  : 0}
                %
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

      <Group justify="space-between">
        <Title order={2}>账号管理</Title>
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
              表单添加
            </Menu.Item>
            <Menu.Item
              leftSection={<FileJson size={16} />}
              onClick={() => {
                setActiveTab('json')
                setShowAddModal(true)
              }}
            >
              JSON 导入
            </Menu.Item>
            <Menu.Item
              leftSection={<Upload size={16} />}
              onClick={() => {
                setActiveTab('file')
                setShowAddModal(true)
              }}
            >
              文件导入
            </Menu.Item>
            <Menu.Item
              leftSection={<Upload size={16} />}
              onClick={handleImportFromKiro}
            >
              从 Kiro IDE 导入
            </Menu.Item>
          </Menu.Dropdown>
        </Menu>
      </Group>

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
                <Group justify="space-between" wrap="nowrap">
                  <Stack gap="xs" style={{ flex: 1 }}>
                    <Group gap="sm">
                      <Text fw={600} size="lg">
                        {account.name || account.id}
                      </Text>
                      <Badge color={getStatusColor(account.status)} variant="light">
                        {account.status}
                      </Badge>
                      <Badge color="gray" variant="outline">
                        {account.authMethod === 'social' ? 'Social' : 'IDC'}
                      </Badge>
                      {health && (
                        <Tooltip label={`成功率: ${(health.success_rate * 100).toFixed(1)}%`}>
                          <Badge color={health.is_available ? 'green' : 'red'} variant="dot">
                            {health.is_available ? '可用' : '不可用'}
                          </Badge>
                        </Tooltip>
                      )}
                    </Group>
                    <Text
                      size="sm"
                      c="dimmed"
                      style={{ fontFamily: 'monospace', fontSize: '0.8em' }}
                    >
                      ID: {account.id}
                    </Text>
                    
                    <Group gap="md">
                      {/* 配额显示 - 紧凑版 */}
                      {quota && (
                        <Group gap="xs">
                          <CreditCard size={14} />
                          <Text size="sm" c="dimmed">
                            配额: {quota.used}/{quota.total}
                          </Text>
                          <Badge
                            size="xs"
                            color={quota.percentage > 80 ? 'red' : quota.percentage > 50 ? 'yellow' : 'green'}
                          >
                            {quota.percentage.toFixed(0)}%
                          </Badge>
                        </Group>
                      )}
                      {isLoadingQuota && (
                        <Group gap="xs">
                          <Loader size="xs" />
                          <Text size="xs" c="dimmed">加载配额...</Text>
                        </Group>
                      )}
                      
                      {/* 健康统计 */}
                      {health && (
                        <>
                          <Text size="sm" c="dimmed">
                            成功: {health.success_count} | 失败: {health.fail_count}
                          </Text>
                          <Text size="sm" c="dimmed">
                            成功率: {(health.success_rate * 100).toFixed(1)}%
                          </Text>
                          {health.last_used && (
                            <Text size="sm" c="dimmed">
                              最后使用: {format(new Date(health.last_used), 'HH:mm:ss')}
                            </Text>
                          )}
                        </>
                      )}
                    </Group>
                    
                    {account.expiresAt && (
                      <Text size="sm" c="dimmed">
                        过期: {format(new Date(account.expiresAt), 'yyyy-MM-dd HH:mm:ss')}
                      </Text>
                    )}
                  </Stack>
                  <Group gap="xs">
                    <ActionIcon
                      variant="light"
                      color="blue"
                      size="lg"
                      onClick={() => handleRefresh(account.id)}
                      loading={isLoadingQuota}
                    >
                      <RefreshCw size={18} />
                    </ActionIcon>
                    <ActionIcon
                      variant="light"
                      color={account.status !== 'disabled' ? 'orange' : 'green'}
                      size="lg"
                      onClick={() => handleToggle(account)}
                    >
                      {account.status !== 'disabled' ? <PowerOff size={18} /> : <Power size={18} />}
                    </ActionIcon>
                    <ActionIcon
                      variant="light"
                      color="red"
                      size="lg"
                      onClick={() => deleteAccount(account.id)}
                    >
                      <Trash2 size={18} />
                    </ActionIcon>
                  </Group>
                </Group>
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
        styles={{
          body: {
            minHeight: '500px',
            maxHeight: '80vh',
            overflow: 'auto',
          },
        }}
      >
        <Tabs value={activeTab} onChange={setActiveTab}>
          <Tabs.List>
            <Tabs.Tab value="form" leftSection={<Plus size={16} />}>
              表单
            </Tabs.Tab>
            <Tabs.Tab value="json" leftSection={<FileJson size={16} />}>
              JSON
            </Tabs.Tab>
            <Tabs.Tab value="file" leftSection={<Upload size={16} />}>
              文件
            </Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel value="form" pt="md">
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
              <Textarea
                label="Refresh Token"
                placeholder="粘贴 Refresh Token"
                value={formData.refreshToken}
                onChange={(e) => setFormData({ ...formData, refreshToken: e.target.value })}
                minRows={4}
                autosize
                maxRows={8}
                required
              />
              {formData.authMethod === 'idc' && (
                <>
                  <TextInput
                    label="Profile ARN"
                    placeholder="arn:aws:..."
                    value={formData.profileArn}
                    onChange={(e) => setFormData({ ...formData, profileArn: e.target.value })}
                  />
                  <TextInput
                    label="Region"
                    placeholder="us-east-1"
                    value={formData.region}
                    onChange={(e) => setFormData({ ...formData, region: e.target.value })}
                  />
                  <TextInput
                    label="Client ID"
                    value={formData.clientId}
                    onChange={(e) => setFormData({ ...formData, clientId: e.target.value })}
                  />
                  <TextInput
                    label="Client Secret"
                    type="password"
                    value={formData.clientSecret}
                    onChange={(e) => setFormData({ ...formData, clientSecret: e.target.value })}
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

          <Tabs.Panel value="json" pt="md">
            <Stack gap="md">
              <Text size="sm" c="dimmed">
                支持单个对象或数组格式，自动识别
              </Text>
              <Textarea
                value={jsonInput}
                onChange={(e) => setJsonInput(e.target.value)}
                minRows={15}
                autosize
                maxRows={25}
                styles={{ input: { fontFamily: 'monospace', fontSize: '0.9em' } }}
              />
              <Group justify="flex-end">
                <Button variant="light" onClick={() => setShowAddModal(false)}>
                  取消
                </Button>
                <Button onClick={handleJsonSubmit}>导入</Button>
              </Group>
            </Stack>
          </Tabs.Panel>

          <Tabs.Panel value="file" pt="md">
            <Stack gap="md">
              <Text size="sm" c="dimmed">
                选择 JSON 文件导入账号（支持单个对象或数组）
              </Text>
              <Code block>
                {`// 单个账号
{
  "id": "account-1",
  "name": "我的账号",
  "authMethod": "social",
  "refreshToken": "...",
  "enabled": true
}

// 或多个账号
[
  { "id": "account-1", ... },
  { "id": "account-2", ... }
]`}
              </Code>
              <FileButton
                resetRef={resetRef}
                onChange={handleFileImport}
                accept="application/json,.json"
              >
                {(props) => (
                  <Button {...props} leftSection={<Upload size={16} />} fullWidth>
                    选择 JSON 文件
                  </Button>
                )}
              </FileButton>
              <Group justify="flex-end">
                <Button variant="light" onClick={() => setShowAddModal(false)}>
                  取消
                </Button>
              </Group>
            </Stack>
          </Tabs.Panel>
        </Tabs>
      </Modal>
    </Stack>
  )
}
