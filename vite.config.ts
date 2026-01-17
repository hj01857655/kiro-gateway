import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:8080',
      '/v1': 'http://127.0.0.1:8080',
      '/admin': 'http://127.0.0.1:8080',
    },
    watch: {
      ignored: ['**/target/**', '**/src-tauri/target/**', '**/node_modules/**']
    },
    fs: {
      strict: false,
    }
  },
  optimizeDeps: {
    exclude: ['target', 'src-tauri'],
    entries: ['src/**/*.{ts,tsx}'],
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // React 核心
          'react-vendor': ['react', 'react-dom', 'react/jsx-runtime'],
          // Mantine UI 组件库
          'mantine-core': ['@mantine/core', '@mantine/hooks'],
          'mantine-charts': ['@mantine/charts'],
          'mantine-form': ['@mantine/form'],
          // TanStack Query
          'tanstack-query': ['@tanstack/react-query'],
          // 图表库
          'recharts': ['recharts'],
          // 工具库
          'utils': ['date-fns', 'zustand'],
        },
      },
    },
    chunkSizeWarningLimit: 600, // 提高警告阈值到 600KB
  },
})
