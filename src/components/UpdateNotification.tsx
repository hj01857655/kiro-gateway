import { useEffect } from 'react'
import { Alert, Button, Group, Stack, Text, Loader } from '@mantine/core'
import { AlertCircle, Download } from 'lucide-react'
import { notifications } from '@mantine/notifications'
import { useAutoUpdate } from '../hooks/useAutoUpdate'

export function UpdateNotification() {
  const { updateInfo, isChecking, error, installUpdate } = useAutoUpdate()

  useEffect(() => {
    if (error) {
      notifications.show({
        title: '更新检查失败',
        message: error,
        color: 'red',
        autoClose: 5000,
      })
    }
  }, [error])

  if (!updateInfo?.available) {
    return null
  }

  return (
    <Alert
      icon={<AlertCircle size={16} />}
      title="发现新版本"
      color="blue"
      style={{
        position: 'fixed',
        bottom: 24,
        right: 24,
        maxWidth: 400,
        zIndex: 1000,
        boxShadow: '0 8px 24px rgba(0, 0, 0, 0.15)',
      }}
    >
      <Stack gap="sm">
        <div>
          <Text size="sm">
            当前版本: <strong>{updateInfo.currentVersion}</strong>
          </Text>
          <Text size="sm">
            新版本: <strong>{updateInfo.newVersion}</strong>
          </Text>
        </div>

        {updateInfo.body && (
          <Text size="xs" c="dimmed" style={{ maxHeight: 100, overflow: 'auto' }}>
            {updateInfo.body}
          </Text>
        )}

        <Group justify="flex-end" gap="xs">
          <Button
            variant="default"
            size="xs"
            onClick={() => {
              // 关闭通知
              const alerts = document.querySelectorAll('[role="alert"]')
              alerts.forEach((alert) => {
                const parent = alert.parentElement
                if (parent) parent.style.display = 'none'
              })
            }}
          >
            稍后
          </Button>
          <Button
            size="xs"
            leftSection={isChecking ? <Loader size={14} /> : <Download size={14} />}
            onClick={installUpdate}
            disabled={isChecking}
          >
            {isChecking ? '安装中...' : '立即更新'}
          </Button>
        </Group>
      </Stack>
    </Alert>
  )
}
