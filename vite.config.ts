import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  clearScreen: false,
  server: {
    host: host || false,
    port: 5173,
    strictPort: true,
    proxy: {
      '/admin': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/v1': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/health': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
    hmr: host
      ? {
          protocol: 'ws',
          host: host,
          port: 5183,
        }
      : undefined,
  },
  // 生产构建优化（防反编译）
  build: {
    // 最小化代码
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,  // 移除 console.log
        drop_debugger: true, // 移除 debugger
        pure_funcs: ['console.log', 'console.info', 'console.debug'], // 移除特定函数
      },
      mangle: {
        toplevel: true, // 混淆顶层作用域
      },
    },
    // 禁用 source map
    sourcemap: false,
    // 代码分割
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          ui: ['@mantine/core', '@mantine/hooks'],
        },
      },
    },
  },
})
