import { createContext, useContext } from 'react'

export type Theme = 'light' | 'dark'

export const STORAGE_KEY = 'raas-theme'

export interface ThemeContextValue {
  theme: Theme
  setTheme: (t: Theme) => void
  toggle: () => void
}

export const ThemeContext = createContext<ThemeContextValue | null>(null)

/** The OS preference; dark is the primary theme when it can't be read. */
export function systemTheme(): Theme {
  if (
    typeof window !== 'undefined' &&
    window.matchMedia('(prefers-color-scheme: dark)').matches
  ) {
    return 'dark'
  }
  return 'light'
}

/** The user's explicit choice, or null when they have not chosen. */
export function readStoredTheme(): Theme | null {
  if (typeof window === 'undefined') return null
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored === 'light' || stored === 'dark') return stored
  } catch {
    /* storage unavailable — fall through to system preference */
  }
  return null
}

export function initialTheme(): Theme {
  return readStoredTheme() ?? systemTheme()
}

/** Every token is scoped to this attribute, so setting it retheme the app. */
export function applyDocumentTheme(theme: Theme) {
  document.documentElement.setAttribute('data-theme', theme)
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within a ThemeProvider')
  return ctx
}
