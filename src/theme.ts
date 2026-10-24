const THEME_KEY = 'outer-loop-theme'

export function initialDark(): boolean {
  const stored = localStorage.getItem(THEME_KEY)
  if (stored === 'dark') return true
  if (stored === 'light') return false
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false
}

export function applyTheme(dark: boolean, persist: boolean) {
  document.documentElement.classList.toggle('dark', dark)
  if (persist) localStorage.setItem(THEME_KEY, dark ? 'dark' : 'light')
}
