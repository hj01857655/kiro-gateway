---
inclusion: fileMatch
fileMatchPattern: "**/models*.{rs,ts}"
---

# 模型映射规范

## Kiro 支持的模型

modelId 格式：`qdev::模型名`

- `qdev::auto` - 自动选择（默认，1.0x）
- `qdev::claude-haiku-4.5` - 快速（0.4x）
- `qdev::claude-sonnet-4` - 常规（1.3x）
- `qdev::claude-sonnet-4.5` - 最新（1.3x）
- `qdev::claude-opus-4.5` - 最强（2.2x）

## 映射规则

**OpenAI → Kiro**
- `gpt-4` / `gpt-4-turbo` / `gpt-4o` → `claude-sonnet-4.5`
- `gpt-3.5-turbo` → `claude-haiku-4.5`

**Anthropic → Kiro**
- `claude-3-5-sonnet-*` → `claude-sonnet-4.5`
- `claude-3-opus-*` → `claude-opus-4.5`
- `claude-3-haiku-*` → `claude-haiku-4.5`

**模糊匹配**
- 包含 `opus` → `claude-opus-4.5`
- 包含 `haiku` → `claude-haiku-4.5`
- 包含 `sonnet` → `claude-sonnet-4.5`
- 默认 → `auto`

## 超时配置

- Haiku: First Token 30s, Stream 60s
- Sonnet: First Token 60s, Stream 120s
- Opus: First Token 120s, Stream 300s
