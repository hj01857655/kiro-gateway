import { useState, useRef } from 'react'
import { useAccounts } from '@/hooks/useAccounts'
import { useHealth, useCheckHealth } from '@/hooks/useHealth'
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
import { Plus, RefreshCw, Trash2, Power, PowerOff, Upload, FileJson, ChevronDown, Activity } from 'lucide-react'
import { format } from 'date-fns'
import type { Account } from '@/types'

export default function Accounts() {
  const { accounts, isLoading, addAccount, updateAccount, deleteAccount, refreshAccount } = useAccounts()
  const { data: healthData } = useHealth()
  const checkHealthMutation = useCheckHealth()
  const [showAddModal, setShowAddModal] = useState(false)
  const [activeTab, setActiveTab] = useState<string | null>('form')
  const resetRef = useRef<() => void>(null)
  
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
}`)
  
  const [batchJsonInput, setBatchJsonInput] = useState(`[
  {
    "id": "account-1",
    "name": "账号 1",
    "authMethod": "social",
    "refreshToken": "token1",
    "profileArn": "",
    "region": "us-east-1",
    "enabled": true
  },
  {
    "id": "account-2",
    "name": "账号 2",
    "authMethod": "idc",
    "refreshToken": "token2",
    "profileArn": "arn:aws:...",
    "region": "us-east-1",
    "clientId": "client-id",
    "clientSecret": "client-secret",
    "enabled": true
  }
]`)

  const handleFormSubmit = () => {
    if (!formData.name || !formData.refreshToken) {
      notifications.show({
        title: '错误',
        message: '请填写必填字段',
        color: 'red',
      })
      return
    }
    addAccount(formData as any)
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
      const account = JSON.parse(jsonInput)
      addAccount(account)
      setShowAddModal(false)
      notifications.show({
        title: '成功',
        message: 'JSON 账号添加成功',
        color: 'green',
      })
    } catch (error) {
      notifications.show({
        title: '错误',
        message: 'JSON 格式错误: ' + (error as Error).message,
        color: 'red',
      })
    }
  }

  const handleBatchJsonSubmit = () => {
    try {
      const accountsArray = JSON.parse(batchJsonInput)
      if (!Array.isArray(accountsArray)) {
        throw new Error('必须是数组格式')
      }
      accountsArray.forEach((account) => addAccount(account))
      setShowAddModal(false)
      notifications.show({
        title: '成功',
        message: `批量导入 ${accountsArray.length} 个账号成功`,
        color: 'green',
      })
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
    updateAccount({
      id: account.id,
      updates: { status: account.status === 'disabled' ? 'active' : 'disabled' } as any,
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
    return healthData?.accounts.find(h => h.id === accountId)
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
              <Text size="xs" c="dimmed">总账号数</Text>
              <Text size="xl" fw={700}>{healthData.total}</Text>
            </Card>
            <Card withBorder p="sm">
              <Text size="xs" c="dimmed">可用账号</Text>
              <Text size="xl" fw={700} c="green">{healthData.available}</Text>
            </Card>
            <Card withBorder p="sm">
              <Text size="xs" c="dimmed">不可用账号</Text>
              <Text size="xl" fw={700} c="red">{healthData.total - healthData.available}</Text>
            </Card>
            <Card withBorder p="sm">
              <Text size="xs" c="dimmed">可用率</Text>
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
              JSON 编辑
            </Menu.Item>
            <Menu.Item
              leftSection={<FileJson size={16} />}
              onClick={() => {
                setActiveTab('batch')
                setShowAddModal(true)
              }}
            >
              批量导入
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
                          <Badge
                            color={health.is_available ? 'green' : 'red'}
                            variant="dot"
                          >
                            {health.is_available ? '可用' : '不可用'}
                          </Badge>
                        </Tooltip>
                      )}
                    </Group>
                    <Text size="sm" c="dimmed" style={{ fontFamily: 'monospace', fontSize: '0.8em' }}>
                      ID: {account.id}
                    </Text>
                    {health && (
                      <Group gap="md">
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
                      </Group>
                    )}
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
                      onClick={() => refreshAccount(account.id)}
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
      >
        <Tabs value={activeTab} onChange={setActiveTab}>
          <Tabs.List>
            <Tabs.Tab value="form" leftSection={<Plus size={16} />}>
              表单
            </Tabs.Tab>
            <Tabs.Tab value="json" leftSection={<FileJson size={16} />}>
              JSON
            </Tabs.Tab>
            <Tabs.Tab value="batch" leftSection={<FileJson size={16} />}>
              批量
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
                minRows={3}
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
                粘贴或编辑 JSON 格式的账号信息
              </Text>
              <Textarea
                value={jsonInput}
                onChange={(e) => setJsonInput(e.target.value)}
                minRows={12}
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

          <Tabs.Panel value="batch" pt="md">
            <Stack gap="md">
              <Text size="sm" c="dimmed">
                粘贴 JSON 数组格式的多个账号
              </Text>
              <Textarea
                value={batchJsonInput}
                onChange={(e) => setBatchJsonInput(e.target.value)}
                minRows={12}
                styles={{ input: { fontFamily: 'monospace', fontSize: '0.9em' } }}
              />
              <Group justify="flex-end">
                <Button variant="light" onClick={() => setShowAddModal(false)}>
                  取消
                </Button>
                <Button onClick={handleBatchJsonSubmit}>批量导入</Button>
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
