export default function Chat() {
  return (
    <div>
      <h2 className="text-2xl font-bold mb-6">聊天测试</h2>
      <div className="text-center py-12 text-gray-500">
        <p>聊天测试功能开发中...</p>
        <p className="mt-2 text-sm">请使用 API 端点测试：</p>
        <code className="block mt-4 p-4 bg-[hsl(var(--muted))] rounded-md text-left">
          POST /v1/chat/completions<br />
          POST /v1/messages
        </code>
      </div>
    </div>
  )
}
