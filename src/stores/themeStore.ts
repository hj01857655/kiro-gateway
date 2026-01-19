import { create } from 'zustand'
import { persist } from 'zustand/middleware'

type ColorScheme = 'light' | 'dark'

interface ThemeStore {
  colorScheme: ColorScheme
  toggleColorScheme: () => void
  setColorScheme: (scheme: ColorScheme) => void
}

const getSystemTheme = (): ColorScheme => {
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  return 'light'
}

export const useThemeStore = create<ThemeStore>()(
  persist(
    (set) => ({
      colorScheme: getSystemTheme(),
      toggleColorScheme: () =>
        set((state) => ({ colorScheme: state.colorScheme === 'light' ? 'dark' : 'light' })),
      setColorScheme: (scheme) => set({ colorScheme: scheme }),
    }),
    { name: 'theme-storage' }
  )
)
