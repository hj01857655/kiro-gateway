# 模型映射

## 概述

Claude Code 请求的模型名称需要映射到 Kiro 支持的模型。

---

## Kiro 支持的模型

Kiro 后端实际使用的是 AWS Bedrock 上的 Claude 模型：

```
claude-sonnet-4-5-20250929      # Claude 4.5 Sonnet (最新)
claude-sonnet-4-20250514        # Claude 4 Sonnet
claude-opus-4-5-20251101        # Claude 4.5 Opus (顶级)
claude-haiku-4-5                # Claude 4.5 Haiku (快速)
claude-3-7-sonnet-latest        # Claude 3.7 Sonnet (Kiro 源码确认)
```

---

## 模型映射表

```javascript
const MODEL_MAPPING = {
  // Claude Code / Anthropic SDK 常用名称
  'claude-3-5-sonnet-20241022': 'claude-sonnet-4-5-20250929',
  'claude-3-5-sonnet': 'claude-sonnet-4-5-20250929',
  'claude-3-opus': 'claude-opus-4-5-20251101',
  'claude-3-haiku': 'claude-haiku-4-5',
  
  // 简写
  'sonnet': 'claude-sonnet-4-5-20250929',
  'opus': 'claude-opus-4-5-20251101',
  'haiku': 'claude-haiku-4-5',
  
  // OpenAI 格式（有些客户端用这个）
  'gpt-4': 'claude-sonnet-4-5-20250929',
  'gpt-4-turbo': 'claude-sonnet-4-5-20250929',
  'gpt-3.5-turbo': 'claude-haiku-4-5',
  
  // 直接使用 Kiro 模型名
  'claude-sonnet-4-5': 'claude-sonnet-4-5-20250929',
  'claude-sonnet-4': 'claude-sonnet-4-20250514',
  'claude-opus-4-5': 'claude-opus-4-5-20251101',
  
  // 默认
  'default': 'claude-sonnet-4-5-20250929'
}
```

---

## 映射函数

```javascript
function mapModel(requestedModel) {
  if (!requestedModel) {
    return MODEL_MAPPING['default']
  }
  
  const normalized = requestedModel.toLowerCase()
  
  // 精确匹配
  if (MODEL_MAPPING[normalized]) {
    return MODEL_MAPPING[normalized]
  }
  
  // 模糊匹配
  if (normalized.includes('opus')) {
    return MODEL_MAPPING['opus']
  }
  if (normalized.includes('haiku')) {
    return MODEL_MAPPING['haiku']
  }
  if (normalized.includes('sonnet')) {
    return MODEL_MAPPING['sonnet']
  }
  
  // 默认
  return MODEL_MAPPING['default']
}
```

---

## /v1/models 端点

提供模型列表给客户端：

```javascript
// GET /v1/models
const modelsResponse = {
  object: 'list',
  data: [
    {
      id: 'claude-sonnet-4-5',
      object: 'model',
      created: 1704067200,
      owned_by: 'kiro'
    },
    {
      id: 'claude-opus-4-5',
      object: 'model',
      created: 1704067200,
      owned_by: 'kiro'
    },
    {
      id: 'claude-haiku-4-5',
      object: 'model',
      created: 1704067200,
      owned_by: 'kiro'
    },
    {
      id: 'claude-sonnet-4',
      object: 'model',
      created: 1704067200,
      owned_by: 'kiro'
    }
  ]
}
```

---

## 模型特性

```
模型                    速度      质量      适用场景
────────────────────────────────────────────────────
claude-opus-4-5        慢        最高      复杂推理、长文档
claude-sonnet-4-5      中        高        日常编码、对话
claude-sonnet-4        中        中高      一般任务
claude-haiku-4-5       快        中        快速响应、简单任务
```

---

## 超时配置建议

不同模型需要不同的超时配置：

```javascript
const TIMEOUT_CONFIG = {
  'claude-opus-4-5': {
    firstTokenTimeout: 120,    // 首个 token 超时 120s
    streamReadTimeout: 300,    // 流读取超时 300s
    nonStreamTimeout: 600      // 非流式超时 600s
  },
  'claude-sonnet-4-5': {
    firstTokenTimeout: 60,
    streamReadTimeout: 120,
    nonStreamTimeout: 300
  },
  'claude-haiku-4-5': {
    firstTokenTimeout: 30,
    streamReadTimeout: 60,
    nonStreamTimeout: 120
  },
  'default': {
    firstTokenTimeout: 60,
    streamReadTimeout: 120,
    nonStreamTimeout: 300
  }
}

function getTimeoutConfig(model) {
  return TIMEOUT_CONFIG[model] || TIMEOUT_CONFIG['default']
}
```
