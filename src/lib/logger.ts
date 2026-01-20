// 前端统一日志系统

type LogLevel = 'debug' | 'info' | 'warn' | 'error'

interface LogConfig {
  enabled: boolean
  level: LogLevel
  prefix: string
}

const config: LogConfig = {
  enabled: import.meta.env.DEV, // 生产环境默认关闭
  level: 'info',
  prefix: '[Kiro Gateway]',
}

const levels: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
}

function shouldLog(level: LogLevel): boolean {
  return config.enabled && levels[level] >= levels[config.level]
}

export const logger = {
  debug: (...args: unknown[]) => {
    if (shouldLog('debug')) {
      console.debug(config.prefix, ...args)
    }
  },

  info: (...args: unknown[]) => {
    if (shouldLog('info')) {
      console.info(config.prefix, ...args)
    }
  },

  warn: (...args: unknown[]) => {
    if (shouldLog('warn')) {
      console.warn(config.prefix, ...args)
    }
  },

  error: (...args: unknown[]) => {
    if (shouldLog('error')) {
      console.error(config.prefix, ...args)
    }
  },

  // 配置方法
  setLevel: (level: LogLevel) => {
    config.level = level
  },

  setEnabled: (enabled: boolean) => {
    config.enabled = enabled
  },

  setPrefix: (prefix: string) => {
    config.prefix = prefix
  },
}
