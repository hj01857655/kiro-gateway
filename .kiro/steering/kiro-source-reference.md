---
inclusion: always
---

# Kiro 源码参考规范

## 重要提醒

**每次发现新的 Kiro 源码信息时，必须同时更新以下内容：**

1. **文档**: `docs/kiro-gate/kiro-api.md` - 添加源码路径和关键函数
2. **规则**: `.kiro/steering/kiro-api.md` - 更新 API 规范
3. **README**: `docs/kiro-gate/README.md` - 更新参考资料部分

## Kiro IDE 源码位置

### 本地安装路径

**Windows**:
```
C:\Users\{用户名}\AppData\Local\Programs\Kiro\resources\app\extensions\
```

**关键文件**:
- `kiro.kiro-agent\dist\extension.js` - 主扩展代码（已压缩）
- 其他扩展目录中也可能包含相关代码

### 关键函数和位置

**已知函数**:
- `convertToGenerateAssistantMessages` (extension.js 约 702988 行) - 转换消息格式
- `GenerateAssistantResponseCommand` - API 调用命令

### GitHub 参考

**AWS Toolkit for VS Code**: [aws/aws-toolkit-vscode](https://github.com/aws/aws-toolkit-vscode)
- 路径: `packages/core/src/codewhispererChat/clients/chat/v0/chat.ts`
- 包含类似的 API 调用逻辑

## 更新流程

当发现新的源码信息时：

1. **立即记录** - 在本文件中添加新发现
2. **更新文档** - 同步到 `docs/kiro-gate/kiro-api.md`
3. **更新规则** - 同步到 `.kiro/steering/kiro-api.md`
4. **验证代码** - 根据源码验证当前实现是否正确

## 已验证的 API 行为

### 响应格式（已确认）

Kiro API 返回**扁平 JSON 对象**，不是嵌套结构：

```json
// ✅ 实际格式
{"content": "文本内容"}
{"content": "代码", "language": "python"}
{"toolUseId": "xxx", "name": "tool", "input": {...}}
{"unit": "credit", "usage": 0.003}
{"contextUsagePercentage": 0.022}

// ❌ 错误理解（嵌套格式）
{"assistantResponseEvent": {"content": "..."}}
{"codeEvent": {"content": "...", "language": "..."}}
```

### 请求格式（已确认）

```json
{
  "conversationState": {
    "conversationId": "uuid",
    "chatTriggerType": "MANUAL",
    "currentMessage": {
      "userInputMessage": {
        "content": "字符串",  // 不是数组
        "modelId": "auto",    // 不需要 qdev:: 前缀
        "origin": "AI_EDITOR"
      }
    },
    "history": []
  }
}
```

## 待验证项

- [ ] 工具调用的完整格式
- [ ] 图片上传的格式
- [ ] System prompt 的处理方式
- [ ] History 消息的格式
- [ ] 错误响应的完整结构

## 模型可用性测试（2026-01-15）

测试结果（可能受 Anthropic 服务状态影响）：

**可用模型**：
- ✅ `auto` - 自动选择
- ✅ `claude-sonnet-4` - Sonnet 4
- ✅ `claude-sonnet-4.5` - Sonnet 4.5

**不可用模型**（可能是临时故障）：
- ❌ `claude-haiku-4.5` - 返回空错误
- ❌ `claude-opus-4.5` - INVALID_MODEL_ID

**注意**：
- 2026-01-14 Anthropic 状态页显示 Opus 4.5 和 Sonnet 4.5 出现过错误率升高
- 模型可用性可能取决于账号类型（免费/付费）或服务状态
- 建议定期重新测试模型可用性
