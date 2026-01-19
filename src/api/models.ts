// 模型 API
import { fetchWithTimeout } from './utils'

export interface Model {
  id: string
  object: string
  owned_by: string
}

export interface ModelsResponse {
  object: string
  data: Model[]
}

// 模型缓存
let modelsCache: Model[] | null = null
let cacheTimestamp: number = 0
const CACHE_DURATION = 5 * 60 * 1000 // 5 分钟缓存

export const modelsApi = {
  // 获取模型列表
  async list(forceRefresh = false): Promise<Model[]> {
    const now = Date.now()

    // 如果有缓存且未过期，直接返回
    if (!forceRefresh && modelsCache && now - cacheTimestamp < CACHE_DURATION) {
      return modelsCache
    }

    try {
      const res = await fetchWithTimeout('/v1/models')
      const data: ModelsResponse = await res.json()
      modelsCache = data.data
      cacheTimestamp = now

      return data.data
    } catch (error) {
      console.error('获取模型列表失败:', error)
      // 如果有旧缓存，返回旧缓存
      if (modelsCache) {
        return modelsCache
      }
      // 否则返回默认模型列表
      return [
        { id: 'claude-haiku-4.5', object: 'model', owned_by: 'anthropic' },
        { id: 'claude-sonnet-4', object: 'model', owned_by: 'anthropic' },
        { id: 'claude-sonnet-4.5', object: 'model', owned_by: 'anthropic' },
      ]
    }
  },

  // 清除缓存
  clearCache() {
    modelsCache = null
    cacheTimestamp = 0
  },
}
