# kiro-gateway 项目文档

kiro-gateway 是一个将 Kiro API 转换为 OpenAI/Anthropic 兼容接口的网关服务。

## 📚 文档目录

- [快速开始](./getting-started.md) - 安装、配置、运行指南
- [账号管理](./account-management.md) - Token 刷新、多账号配置
- [API 使用](./api-usage.md) - 接口调用示例、参数说明
- [模型映射](./model-mapping.md) - 支持的模型列表、映射规则
- [错误处理](./error-handling.md) - 常见错误、重试策略
- [部署指南](./deployment.md) - 生产环境部署建议

## 🎯 项目特点

- ✅ OpenAI 兼容接口 - 支持 `/v1/chat/completions`
- ✅ Anthropic 兼容接口 - 支持 `/v1/messages`
- ✅ 流式响应 - SSE 实时输出
- ✅ 多账号轮询 - 自动负载均衡
- ✅ Token 自动刷新 - 无需手动维护
- ✅ 错误重试 - 指数退避策略

## 🔗 相关链接

- [Kiro API 参考](../kiro-gate/) - Kiro 官方 API 文档整理
- [GitHub 仓库](https://github.com/your-username/kiro-gate)
- [问题反馈](https://github.com/your-username/kiro-gate/issues)
