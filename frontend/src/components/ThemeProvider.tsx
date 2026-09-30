import { useEffect, useRef, useState, type ReactNode } from 'react'
import {
  STORAGE_KEY,
  ThemeContext,
  applyDocumentTheme,
  initialTheme,
  readStoredTheme,
  type Theme,
} from '../theme'

export default function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(initialTheme)
  // Until the user chooses explicitly, follow the OS preference.
  const explicit = useRef<boolean>(
    typeof window !== 'undefined' && readStoredTheme() !== null,
  )

  useEffect(() => {
    applyDocumentTheme(theme)
  }, [theme])

  useEffect(() => {
    const query = window.matchMedia('(prefers-color-scheme: dark)')
    const onChange = () => {
      if (!explicit.current) setThemeState(query.matches ? 'dark' : 'light')
    }
    query.addEventListener('change', onChange)
    return () => query.removeEventListener('change', onChange)
  }, [])

  const setTheme = (next: Theme) => {
    explicit.current = true
    try {
      localStorage.setItem(STORAGE_KEY, next)
    } catch {
      /* non-fatal */
    }
    setThemeState(next)
  }

  const toggle = () => setTheme(theme === 'dark' ? 'light' : 'dark')

  return (
    <ThemeContext.Provider value={{ theme, setTheme, toggle }}>
      {children}
    </ThemeContext.Provider>
  )
}
