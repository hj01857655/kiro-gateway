import { create } from 'zustand'
import { persist } from 'zustand/middleware'

type ColorScheme = 'light' | 'dark'

interface ThemeStore {
  colorScheme: ColorScheme
  toggleColorScheme: () => void
  setColorScheme: (scheme: ColorScheme) => void
}

export const useThemeStore = create<ThemeStore>()(
  persist(
    (set) => ({
      colorScheme: 'light',
      toggleColorScheme: () =>
        set((state) => ({ colorScheme: state.colorScheme === 'light' ? 'dark' : 'light' })),
      setColorScheme: (scheme) => set({ colorScheme: scheme }),
    }),
    { name: 'theme-storage' }
  )
)
