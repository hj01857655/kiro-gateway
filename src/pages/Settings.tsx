import {
  Card,
  Stack,
  Title,
  Text,
  TextInput,
  Group,
  Code,
  Button,
  ActionIcon,
  Badge,
  Modal,
  CopyButton,
  Tooltip,
  Tabs,
  Textarea,
  Alert,
  Switch,
  Center,
  rem,
} from '@mantine/core'
import { Server, Key, Globe, Plus, Trash2, Copy, Check, Power, PowerOff, Download, Wand2, AlertCircle, Moon } from 'lucide-react'
import { useState, useEffect } from 'react'
import { notifications } from '@mantine/notifications'
import { apiKeysApi, type ApiKey } from '@/api/apiKeys'
import { useThemeStore } from '@/stores/themeStore'

export default function Settings() {
  const colorScheme = useThemeStore((state) => state.colorScheme)
  const toggleColorScheme = useThemeStore((state) => state.toggleColorScheme)
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([])
  const [showAddModal, setShowAddModal] = useState(false)
  const [newKeyName, setNewKeyName] = useState('')
  const [generatedKey, setGeneratedKey] = useState<ApiKey | null>(null)
  const [loading, setLoading] = useState(false)
  const [showConfigModal, setShowConfigModal] = useState(false)
  const [configPackage, setConfigPackage] = useState<any>(null)
  const [configLoading, setConfigLoading] = useState(false)
  
  // 服务器配置
  const [serverConfig, setServerConfig] = useState({ host: '127.0.0.1', port: 8080 })
  const [editingServer, setEditingServer] = useState(false)
  const [serverConfigDirty, setServerConfigDirty] = useState(false)

  useEffect(() => {
    loadApiKeys()
    loadServerConfig()
  }, [])

  const loadServerConfig = async () => {
    try {
      const res = await fetch('/admin/config/server')
      if (!res.ok) throw new Error('加载配置失败')
      const data = await res.json()
      setServerConfig(data)
    } catch (error) {
      console.error('加载服务器配置失败:', error)
    }
  }

  const loadApiKeys = async () => {
    try {
      const keys = await apiKeysApi.list()
      setApiKeys(keys)
    } catch (error) {
      console.error('加载 API Keys 失败:', error)
    }
  }

  const handleGenerateKey = async () => {
    if (!newKeyName.trim()) {
      notifications.show({
        title: '错误',
        message: '请输入 API Key 名称',
        color: 'red',
      })
      return
    }

    setLoading(true)
    try {
      const key = await apiKeysApi.generate(newKeyName.trim())
      setGeneratedKey(key)
      setNewKeyName('')
      await loadApiKeys()
      notifications.show({
        title: '成功',
        message: 'API Key 生成成功',
        color: 'green',
      })
    } catch (error) {
      notifications.show({
        title: '生成失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
    } finally {
      setLoading(false)
    }
  }

  const handleToggleKey = async (key: ApiKey) => {
    try {
      await apiKeysApi.update(key.id, { enabled: !key.enabled })
      await loadApiKeys()
      notifications.show({
        title: '成功',
        message: `API Key 已${key.enabled ? '禁用' : '启用'}`,
        color: 'green',
      })
    } catch (error) {
      notifications.show({
        title: '操作失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
    }
  }

  const handleDeleteKey = async (id: string) => {
    if (!confirm('确定要删除这个 API Key 吗？')) return

    try {
      await apiKeysApi.delete(id)
      await loadApiKeys()
      notifications.show({
        title: '成功',
        message: 'API Key 已删除',
        color: 'green',
      })
    } catch (error) {
      notifications.show({
        title: '删除失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
    }
  }

  // 生成配置包
  const handleGenerateConfig = async () => {
    setConfigLoading(true)
    try {
      const res = await fetch('/admin/config/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
      })
      if (!res.ok) throw new Error('生成配置失败')
      const data = await res.json()
      setConfigPackage(data)
      setShowConfigModal(true)
      notifications.show({
        title: '成功',
        message: '配置已生成',
        color: 'green',
      })
    } catch (error) {
      notifications.show({
        title: '生成失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
    } finally {
      setConfigLoading(false)
    }
  }

  // 保存服务器配置
  const handleSaveServerConfig = async () => {
    // 验证端口范围
    if (serverConfig.port < 1024 || serverConfig.port > 65535) {
      notifications.show({
        title: '验证失败',
        message: '端口号必须在 1024-65535 之间',
        color: 'red',
      })
      return
    }

    setLoading(true)
    try {
      const res = await fetch('/admin/config/server', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(serverConfig),
      })
      if (!res.ok) throw new Error('保存配置失败')
      const data = await res.json()
      
      notifications.show({
        title: '成功',
        message: data.message,
        color: 'green',
      })
      
      setEditingServer(false)
      setServerConfigDirty(false)
      
      // 询问是否重启
      if (data.needRestart && confirm('配置已保存，是否立即重启应用使其生效？')) {
        await handleRestartApp()
      }
    } catch (error) {
      notifications.show({
        title: '保存失败',
        message: error instanceof Error ? error.message : '未知错误',
        color: 'red',
      })
    } finally {
      setLoading(false)
    }
  }

  // 重启应用
  const handleRestartApp = async () => {
    try {
      const { relaunch } = await import('@tauri-apps/plugin-process')
      await relaunch()
    } catch (error) {
      notifications.show({
        title: '重启失败',
        message: error instanceof Error ? error.message : '请手动重启应用',
        color: 'red',
      })
    }
  }

  return (
    <Stack gap="md" maw={1200} mx="auto" className="animate-fade-in">
      <Title order={2}>设置</Title>

      <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
        <Group mb="md">
          <div
            style={{
              width: rem(40),
              height: rem(40),
              borderRadius: rem(10),
              background: colorScheme === 'dark'
                ? 'linear-gradient(135deg, rgba(121, 80, 242, 0.2) 0%, rgba(121, 80, 242, 0.1) 100%)'
                : 'linear-gradient(135deg, rgba(121, 80, 242, 0.15) 0%, rgba(121, 80, 242, 0.08) 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Moon size={20} color="#7950f2" />
          </div>
          <Text size="lg" fw={600}>
            外观
          </Text>
        </Group>
        <Group justify="space-between" p="md" style={{ 
          borderRadius: rem(10),
          background: colorScheme === 'dark' ? 'rgba(99, 102, 241, 0.05)' : 'rgba(99, 102, 241, 0.03)',
        }}>
          <div>
            <Text size="sm" fw={500}>深色模式</Text>
            <Text size="xs" c="dimmed">切换浅色/深色主题</Text>
          </div>
          <Switch
            checked={colorScheme === 'dark'}
            onChange={toggleColorScheme}
            size="md"
            color="violet"
          />
        </Group>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
        <Group justify="space-between" mb="md">
          <Group>
            <div
              style={{
                width: rem(40),
                height: rem(40),
                borderRadius: rem(10),
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, rgba(34, 139, 230, 0.2) 0%, rgba(34, 139, 230, 0.1) 100%)'
                  : 'linear-gradient(135deg, rgba(34, 139, 230, 0.15) 0%, rgba(34, 139, 230, 0.08) 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Server size={20} color="#228be6" />
            </div>
            <Text size="lg" fw={600}>
              服务器配置
            </Text>
          </Group>
          {!editingServer ? (
            <Button
              size="sm"
              variant="light"
              onClick={() => setEditingServer(true)}
            >
              编辑
            </Button>
          ) : (
            <Group gap="xs">
              <Button
                size="sm"
                variant="light"
                onClick={() => {
                  setEditingServer(false)
                  loadServerConfig()
                }}
              >
                取消
              </Button>
              <Button
                size="sm"
                onClick={handleSaveServerConfig}
                loading={loading}
                disabled={!serverConfigDirty}
              >
                保存
              </Button>
            </Group>
          )}
        </Group>
        <Stack gap="md">
          <TextInput 
            label="监听地址" 
            value={serverConfig.host}
            onChange={(e) => {
              setServerConfig({ ...serverConfig, host: e.target.value })
              setServerConfigDirty(true)
            }}
            disabled={!editingServer}
            styles={{
              input: {
                borderRadius: rem(10),
              }
            }}
          />
          <TextInput 
            label="监听端口" 
            value={serverConfig.port.toString()}
            onChange={(e) => {
              const port = parseInt(e.target.value) || 8080
              setServerConfig({ ...serverConfig, port })
              setServerConfigDirty(true)
            }}
            disabled={!editingServer}
            type="number"
            min={1024}
            max={65535}
            styles={{
              input: {
                borderRadius: rem(10),
              }
            }}
          />
          <Alert icon={<AlertCircle size={16} />} color="blue" variant="light">
            {editingServer ? '修改后需要保存并重启应用才能生效' : '点击"编辑"按钮修改服务器配置'}
          </Alert>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
        <Group justify="space-between" mb="md">
          <Group>
            <div
              style={{
                width: rem(40),
                height: rem(40),
                borderRadius: rem(10),
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, rgba(64, 192, 87, 0.2) 0%, rgba(64, 192, 87, 0.1) 100%)'
                  : 'linear-gradient(135deg, rgba(64, 192, 87, 0.15) 0%, rgba(64, 192, 87, 0.08) 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Key size={20} color="#40c057" />
            </div>
            <Text size="lg" fw={600}>
              API Key 管理
            </Text>
          </Group>
          <Button
            leftSection={<Plus size={16} />}
            onClick={() => setShowAddModal(true)}
            size="sm"
            variant="light"
          >
            生成新 Key
          </Button>
        </Group>
        <Text size="sm" c="dimmed" mb="md">
          API Key 用于保护网关接口，客户端需要在请求头中携带 Authorization: Bearer YOUR_API_KEY
        </Text>
        
        {apiKeys.length === 0 ? (
          <Card withBorder p="xl" radius="md" style={{ background: colorScheme === 'dark' ? 'rgba(99, 102, 241, 0.03)' : 'rgba(99, 102, 241, 0.02)' }}>
            <Center>
              <Stack align="center" gap="sm">
                <Key size={48} color={colorScheme === 'dark' ? '#666' : '#aaa'} />
                <Text size="sm" c="dimmed" ta="center">
                  暂无 API Key，点击上方按钮生成
                </Text>
              </Stack>
            </Center>
          </Card>
        ) : (
          <Stack gap="sm">
            {apiKeys.map((key) => (
              <Card key={key.id} withBorder p="md" radius="md" className="glass-effect">
                <Group justify="space-between">
                  <Stack gap="xs" style={{ flex: 1 }}>
                    <Group gap="sm">
                      <Text size="sm" fw={500}>
                        {key.name}
                      </Text>
                      <Badge color={key.enabled ? 'green' : 'gray'} size="sm" variant="light">
                        {key.enabled ? '启用' : '禁用'}
                      </Badge>
                    </Group>
                    <Group gap="xs">
                      <Code style={{ fontSize: '0.75em', padding: '4px 8px', borderRadius: rem(6) }}>
                        {key.key.slice(0, 20)}...{key.key.slice(-8)}
                      </Code>
                      <CopyButton value={key.key}>
                        {({ copied, copy }) => (
                          <Tooltip label={copied ? '已复制' : '复制'}>
                            <ActionIcon
                              size="sm"
                              variant="subtle"
                              onClick={copy}
                              color={copied ? 'teal' : 'gray'}
                            >
                              {copied ? <Check size={14} /> : <Copy size={14} />}
                            </ActionIcon>
                          </Tooltip>
                        )}
                      </CopyButton>
                    </Group>
                    <Text size="xs" c="dimmed">
                      创建于: {new Date(key.createdAt).toLocaleString('zh-CN', {
                        year: 'numeric',
                        month: '2-digit',
                        day: '2-digit',
                        hour: '2-digit',
                        minute: '2-digit',
                        second: '2-digit',
                      })}
                      {key.lastUsed && ` · 最后使用: ${new Date(key.lastUsed).toLocaleString('zh-CN', {
                        month: '2-digit',
                        day: '2-digit',
                        hour: '2-digit',
                        minute: '2-digit',
                      })}`}
                    </Text>
                  </Stack>
                  <Group gap="xs">
                    <ActionIcon
                      variant="light"
                      color={key.enabled ? 'orange' : 'green'}
                      onClick={() => handleToggleKey(key)}
                      size="lg"
                    >
                      {key.enabled ? <PowerOff size={16} /> : <Power size={16} />}
                    </ActionIcon>
                    <ActionIcon
                      variant="light"
                      color="red"
                      onClick={() => handleDeleteKey(key.id)}
                      size="lg"
                    >
                      <Trash2 size={16} />
                    </ActionIcon>
                  </Group>
                </Group>
              </Card>
            ))}
          </Stack>
        )}
      </Card>

      <Modal
        opened={showAddModal}
        onClose={() => {
          setShowAddModal(false)
          setGeneratedKey(null)
        }}
        title={generatedKey ? 'API Key 生成成功' : '生成新 API Key'}
      >
        {generatedKey ? (
          <Stack gap="md">
            <Text size="sm" c="orange" fw={500}>
              ⚠️ 请立即复制保存，关闭后将无法再次查看完整 Key
            </Text>
            <TextInput
              label="API Key"
              value={generatedKey.key}
              readOnly
              rightSection={
                <CopyButton value={generatedKey.key}>
                  {({ copied, copy }) => (
                    <ActionIcon onClick={copy} color={copied ? 'teal' : 'gray'}>
                      {copied ? <Check size={16} /> : <Copy size={16} />}
                    </ActionIcon>
                  )}
                </CopyButton>
              }
            />
            <Button
              fullWidth
              onClick={() => {
                setShowAddModal(false)
                setGeneratedKey(null)
              }}
            >
              我已保存
            </Button>
          </Stack>
        ) : (
          <Stack gap="md">
            <TextInput
              label="名称"
              placeholder="例如: 生产环境 Key"
              value={newKeyName}
              onChange={(e) => setNewKeyName(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleGenerateKey()}
            />
            <Group justify="flex-end">
              <Button variant="light" onClick={() => setShowAddModal(false)}>
                取消
              </Button>
              <Button onClick={handleGenerateKey} loading={loading}>
                生成
              </Button>
            </Group>
          </Stack>
        )}
      </Modal>

      <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
        <Group mb="md">
          <div
            style={{
              width: rem(40),
              height: rem(40),
              borderRadius: rem(10),
              background: colorScheme === 'dark'
                ? 'linear-gradient(135deg, rgba(121, 80, 242, 0.2) 0%, rgba(121, 80, 242, 0.1) 100%)'
                : 'linear-gradient(135deg, rgba(121, 80, 242, 0.15) 0%, rgba(121, 80, 242, 0.08) 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Globe size={20} color="#7950f2" />
          </div>
          <Text size="lg" fw={600}>
            API 端点
          </Text>
        </Group>
        <Stack gap="md">
          <div>
            <Text size="sm" fw={500} mb={8}>
              OpenAI 兼容接口
            </Text>
            <Code block style={{ borderRadius: rem(8), padding: rem(12) }}>
              POST http://127.0.0.1:8080/v1/chat/completions
            </Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={8}>
              Anthropic 兼容接口
            </Text>
            <Code block style={{ borderRadius: rem(8), padding: rem(12) }}>
              POST http://127.0.0.1:8080/v1/messages
            </Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={8}>
              模型列表
            </Text>
            <Code block style={{ borderRadius: rem(8), padding: rem(12) }}>
              GET http://127.0.0.1:8080/v1/models
            </Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={8}>
              健康检查
            </Text>
            <Code block style={{ borderRadius: rem(8), padding: rem(12) }}>
              GET http://127.0.0.1:8080/health
            </Code>
          </div>
        </Stack>
      </Card>

      {/* 一键配置 Claude */}
      <Card shadow="sm" padding="lg" radius="md" withBorder className="glass-effect">
        <Group justify="space-between" mb="md">
          <Group>
            <div
              style={{
                width: rem(40),
                height: rem(40),
                borderRadius: rem(10),
                background: colorScheme === 'dark'
                  ? 'linear-gradient(135deg, rgba(245, 159, 0, 0.2) 0%, rgba(245, 159, 0, 0.1) 100%)'
                  : 'linear-gradient(135deg, rgba(245, 159, 0, 0.15) 0%, rgba(245, 159, 0, 0.08) 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Wand2 size={20} color="#f59f00" />
            </div>
            <Text size="lg" fw={600}>
              一键配置 Claude
            </Text>
          </Group>
          <Button
            leftSection={<Download size={16} />}
            onClick={handleGenerateConfig}
            loading={configLoading}
            size="sm"
            variant="light"
          >
            生成配置
          </Button>
        </Group>
        <Text size="sm" c="dimmed" mb="md">
          自动生成 Claude CLI 和 OpenAI 兼容工具的配置脚本
        </Text>
        <Alert icon={<AlertCircle size={16} />} color="blue" variant="light">
          点击"生成配置"后，可以选择：
          <ul style={{ marginTop: 8, marginBottom: 0 }}>
            <li>复制配置脚本手动配置 Claude CLI</li>
            <li>复制 OpenAI 兼容配置供其他工具使用（Continue、Cursor 等）</li>
          </ul>
        </Alert>
      </Card>

      {/* 配置生成模态框 */}
      <Modal
        opened={showConfigModal}
        onClose={() => setShowConfigModal(false)}
        title="Claude 配置"
        size="lg"
      >
        {configPackage && (
          <Tabs defaultValue="cli">
            <Tabs.List>
              <Tabs.Tab value="cli">Claude CLI</Tabs.Tab>
              <Tabs.Tab value="openai">OpenAI 兼容</Tabs.Tab>
            </Tabs.List>

            <Tabs.Panel value="cli" pt="md">
              <Stack gap="md">
                <Textarea
                  label="配置脚本"
                  value={configPackage.claude_cli_script}
                  readOnly
                  minRows={18}
                  autosize
                  styles={{ input: { fontFamily: 'monospace', fontSize: '0.85em' } }}
                />
                <CopyButton value={configPackage.claude_cli_script}>
                  {({ copied, copy }) => (
                    <Button
                      leftSection={copied ? <Check size={16} /> : <Copy size={16} />}
                      onClick={copy}
                      fullWidth
                      variant="light"
                      color={copied ? 'teal' : 'blue'}
                    >
                      {copied ? '已复制' : '复制脚本'}
                    </Button>
                  )}
                </CopyButton>
                <Alert icon={<AlertCircle size={16} />} color="blue" variant="light" title="使用方法">
                  复制上方脚本到终端执行，或手动设置环境变量
                </Alert>
              </Stack>
            </Tabs.Panel>

            <Tabs.Panel value="openai" pt="md">
              <Stack gap="md">
                <Textarea
                  label="OpenAI 兼容配置"
                  value={configPackage.openai_config}
                  readOnly
                  minRows={18}
                  autosize
                  styles={{ input: { fontFamily: 'monospace', fontSize: '0.85em' } }}
                />
                <CopyButton value={configPackage.openai_config}>
                  {({ copied, copy }) => (
                    <Button
                      leftSection={copied ? <Check size={16} /> : <Copy size={16} />}
                      onClick={copy}
                      fullWidth
                      variant="light"
                      color={copied ? 'teal' : 'blue'}
                    >
                      {copied ? '已复制' : '复制配置'}
                    </Button>
                  )}
                </CopyButton>
                <Alert icon={<AlertCircle size={16} />} color="blue" variant="light" title="适用工具">
                  适用于 Continue、Cursor、LangChain 等支持 OpenAI API 的工具
                </Alert>
              </Stack>
            </Tabs.Panel>
          </Tabs>
        )}
      </Modal>
    </Stack>
  )
}
