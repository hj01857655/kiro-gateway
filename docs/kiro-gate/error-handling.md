# 错误处理

## 概述

KiroGate 需要将 Kiro API 的错误转换为 OpenAI/Anthropic 格式。

---

## Kiro 错误类型

### 完整错误列表

```javascript
const KIRO_ERRORS = {
  // 认证错误 (401/403)
  'ExpiredTokenException': { status: 401, retry: true },
  'AccessDeniedException': { status: 403, retry: false },
  'UnauthorizedClientException': { status: 401, retry: false },
  
  // 限流错误 (429)
  'ThrottlingException': { status: 429, retry: true },
  'TooManyRequestsException': { status: 429, retry: true },
  'RequestThrottledException': { status: 429, retry: true },
  'RequestLimitExceeded': { status: 429, retry: true },
  'LimitExceededException': { status: 429, retry: true },
  'ProvisionedThroughputExceededException': { status: 429, retry: true },
  'BandwidthLimitExceeded': { status: 429, retry: true },
  'SlowDown': { status: 429, retry: true },
  
  // 配额错误 (402)
  'ServiceQuotaExceededException': { status: 402, retry: false },
  
  // 请求错误 (400)
  'ValidationException': { status: 400, retry: false },
  'InvalidRequestException': { status: 400, retry: false },
  
  // 资源错误 (404/409)
  'ResourceNotFoundException': { status: 404, retry: false },
  'ConflictException': { status: 409, retry: false },
  
  // 服务错误 (500/502/503/504)
  'InternalServerException': { status: 500, retry: true },
  'ServiceUnavailableException': { status: 503, retry: true },
  
  // 超时错误
  'TimeoutError': { status: 504, retry: true },
  'RequestTimeout': { status: 504, retry: true },
  'RequestTimeoutException': { status: 504, retry: true }
}
```

### AccessDeniedException 子类型

AccessDeniedException 有多个 reason，需要特殊处理：

```javascript
// reason 字段值
const ACCESS_DENIED_REASONS = {
  'FEATURE_NOT_SUPPORTED': '功能不支持（账号类型限制）',
  'TEMPORARILY_SUSPENDED': '账号临时封禁'
}
```

### ThrottlingException 子类型

```javascript
// reason 字段值
const THROTTLING_REASONS = {
  'INSUFFICIENT_MODEL_CAPACITY': '模型容量不足',
  'HOURLY_REQUEST_COUNT': '每小时请求数超限',
  'DAILY_REQUEST_COUNT': '每日请求数超限',
  'WEEKLY_REQUEST_COUNT': '每周请求数超限',
  'MONTHLY_REQUEST_COUNT': '每月请求数超限',
  'USAGE_LIMIT_REACHED': '配额上限'
}
```

### ValidationException 子类型

```javascript
// 通过 message 或 reason 判断
const VALIDATION_ERRORS = {
  'Input is too long': '上下文长度超限',
  'INVALID_MODEL_ID': '无效的模型 ID'
}
```

### InternalServerException 子类型

```javascript
// reason 字段值
const INTERNAL_REASONS = {
  'MODEL_TEMPORARILY_UNAVAILABLE': '模型暂时不可用',
  'Encountered unexpectedly high load': '服务高负载'
}
```

---

## 错误转换

### OpenAI 错误格式

```json
{
  "error": {
    "message": "错误描述",
    "type": "invalid_request_error",
    "param": null,
    "code": "invalid_api_key"
  }
}
```

### Anthropic 错误格式

```json
{
  "type": "error",
  "error": {
    "type": "authentication_error",
    "message": "错误描述"
  }
}
```

---

## 错误映射

```javascript
function convertKiroError(kiroError, format = 'openai') {
  const errorType = kiroError.name || kiroError.__type || 'UnknownError'
  const message = kiroError.message || '未知错误'
  const reason = kiroError.reason
  
  // 特殊处理 AccessDeniedException
  if (errorType === 'AccessDeniedException') {
    if (reason === 'TEMPORARILY_SUSPENDED') {
      return formatError('账号已被临时封禁', 'permission_error', 'account_suspended', format)
    }
    if (reason === 'FEATURE_NOT_SUPPORTED') {
      return formatError('账号类型不支持此功能', 'permission_error', 'feature_not_supported', format)
    }
  }
  
  // 特殊处理 ValidationException
  if (errorType === 'ValidationException') {
    if (message.includes('Input is too long')) {
      return formatError('上下文长度超出限制', 'invalid_request_error', 'context_length_exceeded', format)
    }
    if (reason === 'INVALID_MODEL_ID') {
      return formatError('无效的模型 ID', 'invalid_request_error', 'invalid_model', format)
    }
  }
  
  // 特殊处理 ThrottlingException
  if (errorType === 'ThrottlingException') {
    if (reason === 'INSUFFICIENT_MODEL_CAPACITY') {
      return formatError('模型容量不足，请稍后重试', 'rate_limit_error', 'model_overloaded', format)
    }
    if (['HOURLY_REQUEST_COUNT', 'DAILY_REQUEST_COUNT', 'WEEKLY_REQUEST_COUNT', 
         'MONTHLY_REQUEST_COUNT', 'USAGE_LIMIT_REACHED'].includes(reason)) {
      return formatError(`配额已用尽: ${reason}`, 'rate_limit_error', 'quota_exceeded', format)
    }
  }
  
  // 特殊处理 ServiceQuotaExceededException
  if (errorType === 'ServiceQuotaExceededException') {
    if (reason === 'OVERAGE_REQUEST_LIMIT_EXCEEDED') {
      return formatError('超额请求限制已达上限', 'rate_limit_error', 'overage_limit', format)
    }
  }
  
  // 特殊处理 InternalServerException
  if (errorType === 'InternalServerException') {
    if (reason === 'MODEL_TEMPORARILY_UNAVAILABLE' || message.includes('high load')) {
      return formatError('模型暂时不可用，请稍后重试', 'server_error', 'model_unavailable', format)
    }
  }
  
  // 通用映射
  const errorMapping = {
    'ExpiredTokenException': {
      openai: { type: 'invalid_request_error', code: 'invalid_api_key' },
      anthropic: { type: 'authentication_error' }
    },
    'AccessDeniedException': {
      openai: { type: 'invalid_request_error', code: 'access_denied' },
      anthropic: { type: 'permission_error' }
    },
    'ThrottlingException': {
      openai: { type: 'rate_limit_error', code: 'rate_limit_exceeded' },
      anthropic: { type: 'rate_limit_error' }
    },
    'ServiceQuotaExceededException': {
      openai: { type: 'rate_limit_error', code: 'quota_exceeded' },
      anthropic: { type: 'rate_limit_error' }
    },
    'ValidationException': {
      openai: { type: 'invalid_request_error', code: 'invalid_request' },
      anthropic: { type: 'invalid_request_error' }
    },
    'ResourceNotFoundException': {
      openai: { type: 'invalid_request_error', code: 'not_found' },
      anthropic: { type: 'not_found_error' }
    },
    'ConflictException': {
      openai: { type: 'invalid_request_error', code: 'conflict' },
      anthropic: { type: 'invalid_request_error' }
    },
    'InternalServerException': {
      openai: { type: 'server_error', code: 'internal_error' },
      anthropic: { type: 'api_error' }
    },
    'ServiceUnavailableException': {
      openai: { type: 'server_error', code: 'service_unavailable' },
      anthropic: { type: 'overloaded_error' }
    },
    'TimeoutError': {
      openai: { type: 'server_error', code: 'timeout' },
      anthropic: { type: 'api_error' }
    }
  }
  
  const mapping = errorMapping[errorType] || {
    openai: { type: 'server_error', code: 'unknown_error' },
    anthropic: { type: 'api_error' }
  }
  
  return formatError(message, mapping[format].type, mapping[format].code || mapping[format].type, format)
}

function formatError(message, type, code, format) {
  if (format === 'openai') {
    return {
      error: {
        message,
        type,
        param: null,
        code
      }
    }
  }
  
  if (format === 'anthropic') {
    return {
      type: 'error',
      error: {
        type,
        message
      }
    }
  }
}
```

---

## HTTP 状态码映射

```javascript
function getHttpStatus(kiroError) {
  const errorType = kiroError.name || kiroError.__type
  
  const statusMapping = {
    'ExpiredTokenException': 401,
    'AccessDeniedException': 403,
    'UnauthorizedClientException': 401,
    'ThrottlingException': 429,
    'TooManyRequestsException': 429,
    'ServiceQuotaExceededException': 429,
    'ValidationException': 400,
    'InvalidRequestException': 400,
    'InternalServerException': 500,
    'ServiceUnavailableException': 503
  }
  
  return statusMapping[errorType] || 500
}
```

---

## 重试策略

### 可重试错误判断

```javascript
// 限流错误码列表
const THROTTLING_ERROR_CODES = [
  'BandwidthLimitExceeded',
  'EC2ThrottledException',
  'LimitExceededException',
  'PriorRequestNotComplete',
  'ProvisionedThroughputExceededException',
  'RequestLimitExceeded',
  'RequestThrottled',
  'RequestThrottledException',
  'SlowDown',
  'ThrottledException',
  'Throttling',
  'ThrottlingException',
  'TooManyRequestsException',
  'TransactionInProgressException'
]

// 临时错误码列表
const TRANSIENT_ERROR_CODES = [
  'TimeoutError',
  'RequestTimeout',
  'RequestTimeoutException'
]

// 可重试的 HTTP 状态码
const TRANSIENT_STATUS_CODES = [500, 502, 503, 504]

function isThrottlingError(error) {
  return error.$metadata?.httpStatusCode === 429
    || THROTTLING_ERROR_CODES.includes(error.name)
    || error.$retryable?.throttling === true
}

function isTransientError(error) {
  return TRANSIENT_ERROR_CODES.includes(error.name)
    || TRANSIENT_STATUS_CODES.includes(error.$metadata?.httpStatusCode || 0)
}

function isRetryableError(error) {
  // Token 过期可重试（刷新后）
  if (error.name === 'ExpiredTokenException') return true
  // 限流错误可重试
  if (isThrottlingError(error)) return true
  // 临时错误可重试
  if (isTransientError(error)) return true
  // 配额用尽不重试
  if (error.name === 'ServiceQuotaExceededException') return false
  // 其他情况不重试
  return false
}
```

### 重试处理器

```javascript
class RetryHandler {
  constructor(options = {}) {
    this.maxRetries = options.maxRetries || 3
    this.baseDelay = options.baseDelay || 100           // AWS SDK 默认 100ms
    this.throttlingDelay = options.throttlingDelay || 500  // 限流错误 500ms
    this.maxDelay = options.maxDelay || 20000           // 最大 20 秒
  }
  
  // 计算延迟时间（AWS SDK 标准算法：指数退避 + 随机抖动）
  getDelay(attempt, error) {
    const errorType = error.name || error.__type
    
    // 限流错误使用更长的基础延迟
    const base = isThrottlingError(error) ? this.throttlingDelay : this.baseDelay
    
    // AWS SDK 算法: Math.random() * 2^attempts * delayBase
    const delay = Math.random() * Math.pow(2, attempt) * base
    
    return Math.min(Math.floor(delay), this.maxDelay)
  }
  
  // 执行带重试的请求
  async execute(fn, onRetry) {
    let lastError
    
    for (let attempt = 0; attempt <= this.maxRetries; attempt++) {
      try {
        return await fn()
      } catch (error) {
        lastError = error
        
        if (!this.shouldRetry(error, attempt)) {
          throw error
        }
        
        const delay = this.getDelay(attempt, error)
        
        if (onRetry) {
          onRetry({ attempt, error, delay })
        }
        
        await this.sleep(delay)
        
        // Token 过期需要刷新
        if (error.name === 'ExpiredTokenException') {
          // 触发 Token 刷新
          await this.refreshToken?.()
        }
      }
    }
    
    throw lastError
  }
  
  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms))
  }
}
```

---

## 流式错误处理

### First Token Timeout

Kiro 源码中有 `timeToFirstToken` 统计，说明首 Token 时间是重要指标。如果长时间没收到首个 Token，应该超时重试。

```javascript
class StreamHandler {
  constructor(options = {}) {
    this.firstTokenTimeout = options.firstTokenTimeout || 60000  // 默认 60 秒
    this.streamReadTimeout = options.streamReadTimeout || 120000 // 默认 120 秒
  }
  
  async* streamWithTimeout(response, format) {
    const reader = response.body.getReader()
    let firstTokenReceived = false
    let lastDataTime = Date.now()
    
    // First Token 超时检测
    const firstTokenTimer = setTimeout(() => {
      if (!firstTokenReceived) {
        reader.cancel('First token timeout')
      }
    }, this.firstTokenTimeout)
    
    try {
      while (true) {
        // 流读取超时检测
        const timeoutPromise = new Promise((_, reject) => {
          setTimeout(() => reject(new Error('Stream read timeout')), 
            this.streamReadTimeout - (Date.now() - lastDataTime))
        })
        
        const { done, value } = await Promise.race([
          reader.read(),
          timeoutPromise
        ])
        
        if (done) break
        
        // 收到首个数据
        if (!firstTokenReceived) {
          firstTokenReceived = true
          clearTimeout(firstTokenTimer)
        }
        
        lastDataTime = Date.now()
        
        // 解析并转换事件...
        for (const event of parseChunk(value)) {
          yield convertEvent(event, format)
        }
      }
    } catch (error) {
      if (error.message.includes('timeout')) {
        throw { name: 'TimeoutError', message: error.message }
      }
      throw error
    } finally {
      clearTimeout(firstTokenTimer)
    }
  }
}
```

### First Token Timeout + 自动重试

```javascript
async function callWithFirstTokenRetry(request, account, options = {}) {
  const maxRetries = options.maxRetries || 2
  const firstTokenTimeout = options.firstTokenTimeout || 60000
  
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch(KIRO_API, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${account.accessToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(request),
        signal: AbortSignal.timeout(firstTokenTimeout)  // 整体超时
      })
      
      if (!response.ok) {
        throw await response.json()
      }
      
      // 检测首 Token
      const reader = response.body.getReader()
      const firstChunk = await Promise.race([
        reader.read(),
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('First token timeout')), firstTokenTimeout)
        )
      ])
      
      if (firstChunk.done) {
        throw new Error('Empty response')
      }
      
      // 返回带首 chunk 的流
      return createStreamWithFirstChunk(reader, firstChunk.value)
      
    } catch (error) {
      const isTimeout = error.message?.includes('timeout') || error.name === 'TimeoutError'
      
      if (isTimeout && attempt < maxRetries) {
        console.log(`First token timeout, retry ${attempt + 1}/${maxRetries}`)
        continue
      }
      
      throw error
    }
  }
}

// 创建带首 chunk 的流
function createStreamWithFirstChunk(reader, firstChunk) {
  let firstChunkSent = false
  
  return new ReadableStream({
    async pull(controller) {
      if (!firstChunkSent) {
        firstChunkSent = true
        controller.enqueue(firstChunk)
        return
      }
      
      const { done, value } = await reader.read()
      if (done) {
        controller.close()
      } else {
        controller.enqueue(value)
      }
    }
  })
}
```

### 按模型配置超时

不同模型响应速度不同，应该配置不同的超时时间：

```javascript
const MODEL_TIMEOUT = {
  'claude-opus-4-5': { firstToken: 120000, stream: 300000 },   // Opus 慢，给 2 分钟
  'claude-sonnet-4-5': { firstToken: 60000, stream: 120000 },  // Sonnet 正常
  'claude-haiku-4-5': { firstToken: 30000, stream: 60000 },    // Haiku 快
  'default': { firstToken: 60000, stream: 120000 }
}

function getTimeoutForModel(model) {
  return MODEL_TIMEOUT[model] || MODEL_TIMEOUT['default']
}
```

---

### 流式错误事件处理

流式响应中的错误需要特殊处理：

```javascript
async function* streamWithErrorHandling(response, format) {
  try {
    for await (const event of parseSSEStream(response)) {
      // 检查错误事件
      if (event.invalidStateEvent) {
        const error = convertKiroError(event.invalidStateEvent, format)
        yield formatSSE(error, format)
        return
      }
      
      // 正常事件
      const converted = convertEvent(event, format)
      if (converted) {
        yield formatSSE(converted, format)
      }
    }
    
    // 正常结束
    yield formatEndEvent(format)
    
  } catch (error) {
    // 网络错误或解析错误
    const converted = convertKiroError(error, format)
    yield formatSSE(converted, format)
  }
}

function formatSSE(data, format) {
  if (format === 'anthropic' && data.event) {
    return `event: ${data.event}\ndata: ${JSON.stringify(data.data)}\n\n`
  }
  return `data: ${JSON.stringify(data)}\n\n`
}

function formatEndEvent(format) {
  if (format === 'openai') {
    return 'data: [DONE]\n\n'
  }
  return 'event: message_stop\ndata: {"type":"message_stop"}\n\n'
}
```

---

## 常见错误场景

### 1. Token 过期

```
Kiro: ExpiredTokenException
→ 刷新 Token
→ 重试请求
→ 如果刷新失败: 返回 401
```

### 2. 限流

```
Kiro: ThrottlingException
→ 等待 2^n 秒
→ 重试请求
→ 最多重试 3 次
→ 超过重试次数: 返回 429
```

### 3. 配额用尽

```
Kiro: ServiceQuotaExceededException
→ 不重试
→ 返回 429 + 提示信息
```

### 4. 请求无效

```
Kiro: ValidationException
→ 不重试
→ 返回 400 + 错误详情
```
