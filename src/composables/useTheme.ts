import { ref, onMounted } from 'vue'

export type Theme = 'dark' | 'light'

const currentTheme = ref<Theme>('dark')

export function useTheme() {
  const setTheme = (theme: Theme) => {
    currentTheme.value = theme
    document.documentElement.className = theme
    localStorage.setItem('theme', theme)
  }

  const toggleTheme = () => {
    const newTheme = currentTheme.value === 'dark' ? 'light' : 'dark'
    setTheme(newTheme)
  }

  const initTheme = () => {
    // Check localStorage first, then default to dark
    const savedTheme = localStorage.getItem('theme') as Theme
    const theme = savedTheme || 'dark'
    setTheme(theme)
  }

  onMounted(() => {
    initTheme()
  })

  return {
    currentTheme,
    setTheme,
    toggleTheme,
    initTheme
  }
}
