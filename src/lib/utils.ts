import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import type { Account } from '@/types'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * 生成账号唯一标识（用于去重）
 * 只使用 email + provider 作为唯一标识
 * @param account 账号对象
 * @returns 唯一标识字符串，如果无法生成则返回 null
 */
export function getAccountKey(account: Account): string | null {
  // 支持两种格式：
  // 1. account.email (Kiro Account Manager 格式)
  // 2. account.quota?.userInfo?.email (kiro-gateway 格式)
  const email = account.email || account.quota?.userInfo?.email
  
  if (email && account.provider) {
    return `${email}-${account.provider}`
  }
  
  // 没有 email 或 provider，返回 null 表示不参与去重
  return null
}
