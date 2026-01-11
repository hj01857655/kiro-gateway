# 部署指南

## 快速开始

### 本地运行

```bash
# 克隆项目
git clone https://github.com/xxx/kirogate.git
cd kirogate

# 安装依赖
npm install

# 配置环境变量
cp .env.example .env
# 编辑 .env 填入配置

# 启动服务
npm start
```

### 配置文件

```bash
# .env
PORT=8080
API_KEY=your-kirogate-api-key

# 账号配置文件路径
ACCOUNTS_FILE=./accounts.json

# 日志级别
LOG_LEVEL=info
```

```json
// accounts.json
{
  "accounts": [
    {
      "id": "account-1",
      "name": "主账号",
      "accessToken": "eyJ...",
      "refreshToken": "eyJ...",
      "enabled": true
    }
  ]
}
```

---

## Docker 部署

### Dockerfile

```dockerfile
FROM node:20-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY . .

EXPOSE 8080

CMD ["node", "src/index.js"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  kirogate:
    build: .
    ports:
      - "8080:8080"
    environment:
      - PORT=8080
      - API_KEY=${API_KEY}
      - LOG_LEVEL=info
    volumes:
      - ./accounts.json:/app/accounts.json:ro
    restart: unless-stopped
```

### 运行

```bash
# 构建并启动
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止
docker-compose down
```

---

## 云服务部署

### Railway

1. Fork 项目到你的 GitHub
2. 在 Railway 创建新项目，选择 GitHub 仓库
3. 添加环境变量：
   - `PORT`: 8080
   - `API_KEY`: 你的 API Key
   - `ACCOUNTS_JSON`: 账号配置 JSON 字符串
4. 部署

### Vercel (Serverless)

```javascript
// api/chat/completions.js
import { handleOpenAI } from '../../src/handlers/openai'

export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' })
  }
  
  return handleOpenAI(req, res)
}

export const config = {
  api: {
    bodyParser: true,
    responseLimit: false
  }
}
```

### 自建服务器

```bash
# 使用 PM2 管理进程
npm install -g pm2

# 启动
pm2 start src/index.js --name kirogate

# 查看状态
pm2 status

# 查看日志
pm2 logs kirogate

# 设置开机自启
pm2 startup
pm2 save
```

---

## Nginx 反向代理

```nginx
server {
    listen 80;
    server_name kirogate.example.com;
    
    # 强制 HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name kirogate.example.com;
    
    ssl_certificate /etc/letsencrypt/live/kirogate.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/kirogate.example.com/privkey.pem;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # SSE 支持
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 86400s;
    }
}
```

---

## 客户端配置

### Claude Code

```bash
# 设置环境变量
export ANTHROPIC_BASE_URL=https://kirogate.example.com
export ANTHROPIC_API_KEY=your-kirogate-api-key

# 或者在 ~/.claude/config.json
{
  "apiBaseUrl": "https://kirogate.example.com",
  "apiKey": "your-kirogate-api-key"
}
```

### Cursor

在 Settings → Models → OpenAI API Key 中配置：

- API Key: `your-kirogate-api-key`
- Base URL: `https://kirogate.example.com/v1`

### Continue (VS Code 插件)

```json
// ~/.continue/config.json
{
  "models": [
    {
      "title": "Kiro Claude",
      "provider": "openai",
      "model": "kiro",
      "apiBase": "https://kirogate.example.com/v1",
      "apiKey": "your-kirogate-api-key"
    }
  ]
}
```

### 通用 OpenAI 客户端

```python
from openai import OpenAI

client = OpenAI(
    api_key="your-kirogate-api-key",
    base_url="https://kirogate.example.com/v1"
)

response = client.chat.completions.create(
    model="kiro",
    messages=[{"role": "user", "content": "Hello"}],
    stream=True
)

for chunk in response:
    print(chunk.choices[0].delta.content, end="")
```

---

## 安全配置

### API Key 认证

```javascript
// 验证 API Key
function validateApiKey(req) {
  const apiKey = req.headers['authorization']?.replace('Bearer ', '') ||
                 req.headers['x-api-key']
  
  if (!apiKey || apiKey !== process.env.API_KEY) {
    throw new Error('Invalid API Key')
  }
}
```

### 速率限制

```javascript
import rateLimit from 'express-rate-limit'

const limiter = rateLimit({
  windowMs: 60 * 1000,  // 1 分钟
  max: 60,              // 最多 60 次请求
  message: { error: 'Too many requests' }
})

app.use('/v1', limiter)
```

### CORS 配置

```javascript
import cors from 'cors'

app.use(cors({
  origin: process.env.CORS_ORIGINS?.split(',') || '*',
  methods: ['GET', 'POST'],
  allowedHeaders: ['Content-Type', 'Authorization', 'x-api-key']
}))
```

---

## 监控和日志

### 健康检查端点

```javascript
app.get('/health', (req, res) => {
  const accounts = accountManager.getAvailableAccounts()
  
  res.json({
    status: accounts.length > 0 ? 'healthy' : 'degraded',
    activeAccounts: accounts.length,
    uptime: process.uptime()
  })
})
```

### 日志格式

```javascript
import winston from 'winston'

const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.json()
  ),
  transports: [
    new winston.transports.Console(),
    new winston.transports.File({ filename: 'logs/error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/combined.log' })
  ]
})
```

### 请求日志

```javascript
app.use((req, res, next) => {
  const start = Date.now()
  
  res.on('finish', () => {
    logger.info({
      method: req.method,
      path: req.path,
      status: res.statusCode,
      duration: Date.now() - start,
      ip: req.ip
    })
  })
  
  next()
})
```

---

## 故障排查

### 常见问题

**1. Token 刷新失败**
- 检查 refreshToken 是否有效
- 确认网络能访问 `prod.us-east-1.auth.desktop.kiro.dev`

**2. 请求超时**
- 检查 Nginx 的 `proxy_read_timeout` 配置
- SSE 流需要较长的超时时间

**3. 响应格式错误**
- 检查客户端期望的格式（OpenAI vs Anthropic）
- 查看日志中的原始响应

**4. 所有账号不可用**
- 检查账号状态
- 手动刷新 Token
- 确认 Kiro 服务是否正常

### 调试模式

```bash
# 启用详细日志
LOG_LEVEL=debug npm start

# 查看请求/响应详情
DEBUG=kirogate:* npm start
```
