# kiro-gateway Web 管理界面

基于 React + TypeScript + Vite + TailwindCSS 的 Web 管理界面。

## 功能

- **账号管理** - 查看、添加、删除、刷新 Kiro 账号
- **统计监控** - 实时查看请求统计、延迟分布、模型使用情况
- **日志查看** - 查看系统日志，支持过滤和搜索
- **主题切换** - 支持深色/浅色模式

## 开发

```bash
# 安装依赖
npm install

# 启动开发服务器（默认 http://localhost:5173）
npm run dev

# 构建生产版本
npm run build

# 预览生产构建
npm run preview
```

## 配置

开发环境下，Vite 会自动代理 API 请求到后端（`http://127.0.0.1:8080`）。

如需修改后端地址，编辑 `vite.config.ts`：

```typescript
export default defineConfig({
  server: {
    proxy: {
      '/api': 'http://your-backend:8080',
      '/v1': 'http://your-backend:8080',
      '/admin': 'http://your-backend:8080',
    }
  }
})
```

## 部署

### 静态部署

构建后将 `dist/` 目录部署到任意静态服务器（Nginx、Vercel、Netlify 等）。

**Nginx 配置示例**：

```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /path/to/dist;
    index index.html;

    # SPA 路由支持
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API 代理到后端
    location /api {
        proxy_pass http://127.0.0.1:8080;
    }
    location /v1 {
        proxy_pass http://127.0.0.1:8080;
    }
    location /admin {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

### Vercel 部署

1. 将项目推送到 GitHub
2. 在 Vercel 导入项目
3. 设置构建配置：
   - Build Command: `cd web && npm run build`
   - Output Directory: `web/dist`
4. 添加环境变量（如需要）
5. 部署

## 技术栈

- **React 19** - UI 框架
- **TypeScript** - 类型安全
- **Vite 6** - 构建工具
- **TailwindCSS 4** - 样式框架
- **React Router 7** - 路由管理

## 项目结构

```
web/
├── src/
│   ├── api/              # API 调用
│   │   └── accounts.ts   # 账号相关 API
│   ├── pages/            # 页面组件
│   │   ├── Accounts.tsx  # 账号管理
│   │   ├── Metrics.tsx   # 统计监控
│   │   ├── Logs.tsx      # 日志查看
│   │   ├── Chat.tsx      # 聊天测试
│   │   └── Settings.tsx  # 设置
│   ├── App.tsx           # 主应用
│   ├── main.tsx          # 入口文件
│   └── index.css         # 全局样式
├── public/               # 静态资源
├── dist/                 # 构建输出
├── index.html            # HTML 模板
├── package.json          # 依赖配置
├── tsconfig.json         # TypeScript 配置
└── vite.config.ts        # Vite 配置
```

## API 端点

前端调用的后端 API：

- `GET /admin/accounts` - 获取账号列表
- `POST /admin/accounts` - 添加账号
- `DELETE /admin/accounts/:id` - 删除账号
- `POST /admin/accounts/:id/refresh` - 刷新账号
- `POST /admin/accounts/:id/enable` - 启用账号
- `POST /admin/accounts/:id/disable` - 禁用账号
- `POST /admin/accounts/import-kiro` - 从 Kiro IDE 导入
- `GET /admin/metrics` - 获取统计数据
- `GET /admin/logs` - 获取日志
- `POST /admin/logs/clear` - 清空日志

## 许可证

MIT
