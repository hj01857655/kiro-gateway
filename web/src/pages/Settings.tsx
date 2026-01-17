export default function Settings() {
  return (
    <div>
      <h2 className="text-2xl font-bold mb-6">设置</h2>
      <div className="space-y-6">
        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <h3 className="text-lg font-semibold mb-2">API 配置</h3>
          <p className="text-sm text-gray-600 mb-4">
            请在后端配置文件中设置 API Key 和其他参数
          </p>
          <div className="space-y-2 text-sm">
            <p><strong>OpenAI 兼容端点:</strong> <code>/v1/chat/completions</code></p>
            <p><strong>Anthropic 兼容端点:</strong> <code>/v1/messages</code></p>
            <p><strong>模型列表:</strong> <code>/v1/models</code></p>
          </div>
        </div>

        <div className="p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]">
          <h3 className="text-lg font-semibold mb-2">关于</h3>
          <p className="text-sm text-gray-600">
            kiro-gateway - Kiro API 网关服务
          </p>
          <p className="text-sm text-gray-600 mt-2">
            提供 OpenAI/Anthropic 兼容接口，支持多账号管理和自动 Token 刷新
          </p>
        </div>
      </div>
    </div>
  )
}
