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
} from '@mantine/core'
import { Settings as SettingsIcon, Server, Key, Globe, Plus, Trash2, Copy, Check, Power, PowerOff } from 'lucide-react'
import { useState, useEffect } from 'react'
import { notifications } from '@mantine/notifications'
import { apiKeysApi, type ApiKey } from '@/api/apiKeys'

export default function Settings() {
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([])
  const [showAddModal, setShowAddModal] = useState(false)
  const [newKeyName, setNewKeyName] = useState('')
  const [generatedKey, setGeneratedKey] = useState<ApiKey | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    loadApiKeys()
  }, [])

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
  return (
    <Stack gap="md">
      <Title order={2}>设置</Title>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Server size={20} color="#228be6" />
          <Text size="lg" fw={600}>
            服务器配置
          </Text>
        </Group>
        <Stack gap="md">
          <TextInput label="监听地址" value="127.0.0.1" disabled />
          <TextInput label="监听端口" value="8080" disabled />
          <Text size="sm" c="dimmed">
            服务器配置需要通过环境变量设置，重启后生效
          </Text>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group justify="space-between" mb="md">
          <Group>
            <Key size={20} color="#40c057" />
            <Text size="lg" fw={600}>
              API Key 管理
            </Text>
          </Group>
          <Button
            leftSection={<Plus size={16} />}
            onClick={() => setShowAddModal(true)}
            size="sm"
          >
            生成新 Key
          </Button>
        </Group>
        <Text size="sm" c="dimmed" mb="md">
          API Key 用于保护网关接口，客户端需要在请求头中携带 Authorization: Bearer YOUR_API_KEY
        </Text>
        
        {apiKeys.length === 0 ? (
          <Text size="sm" c="dimmed" ta="center" py="xl">
            暂无 API Key，点击上方按钮生成
          </Text>
        ) : (
          <Stack gap="sm">
            {apiKeys.map((key) => (
              <Card key={key.id} withBorder p="md">
                <Group justify="space-between">
                  <Stack gap="xs" style={{ flex: 1 }}>
                    <Group gap="sm">
                      <Text size="sm" fw={500}>
                        {key.name}
                      </Text>
                      <Badge color={key.enabled ? 'green' : 'gray'} size="sm">
                        {key.enabled ? '启用' : '禁用'}
                      </Badge>
                    </Group>
                    <Group gap="xs">
                      <Code style={{ fontSize: '0.75em' }}>
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
                      创建于: {new Date(key.created_at).toLocaleString()}
                    </Text>
                  </Stack>
                  <Group gap="xs">
                    <ActionIcon
                      variant="light"
                      color={key.enabled ? 'orange' : 'green'}
                      onClick={() => handleToggleKey(key)}
                    >
                      {key.enabled ? <PowerOff size={16} /> : <Power size={16} />}
                    </ActionIcon>
                    <ActionIcon
                      variant="light"
                      color="red"
                      onClick={() => handleDeleteKey(key.id)}
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

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <Globe size={20} color="#7950f2" />
          <Text size="lg" fw={600}>
            API 端点
          </Text>
        </Group>
        <Stack gap="sm">
          <div>
            <Text size="sm" fw={500} mb={4}>
              OpenAI 兼容接口
            </Text>
            <Code block>POST http://127.0.0.1:8080/v1/chat/completions</Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={4}>
              Anthropic 兼容接口
            </Text>
            <Code block>POST http://127.0.0.1:8080/v1/messages</Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={4}>
              模型列表
            </Text>
            <Code block>GET http://127.0.0.1:8080/v1/models</Code>
          </div>
          <div>
            <Text size="sm" fw={500} mb={4}>
              健康检查
            </Text>
            <Code block>GET http://127.0.0.1:8080/health</Code>
          </div>
        </Stack>
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group mb="md">
          <SettingsIcon size={20} color="#868e96" />
          <Text size="lg" fw={600}>
            关于
          </Text>
        </Group>
        <Stack gap="xs">
          <Group>
            <Text size="sm" fw={500}>
              项目名称:
            </Text>
            <Text size="sm">kiro-gateway</Text>
          </Group>
          <Group>
            <Text size="sm" fw={500}>
              版本:
            </Text>
            <Text size="sm">1.0.0</Text>
          </Group>
          <Group>
            <Text size="sm" fw={500}>
              技术栈:
            </Text>
            <Text size="sm">Tauri 2.0 + Rust + React + Mantine</Text>
          </Group>
          <Group>
            <Text size="sm" fw={500}>
              描述:
            </Text>
            <Text size="sm">Kiro API 网关，提供 OpenAI/Anthropic 兼容接口</Text>
          </Group>
        </Stack>
      </Card>
    </Stack>
  )
}
