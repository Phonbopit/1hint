import clsx from 'clsx'
import { useEffect, useState } from 'react'

interface ThemeToggleProps {
  className?: string
}

function ThemeToggle({ className }: ThemeToggleProps) {
  const [isDark, setIsDark] = useState(() => {
    if (typeof window !== 'undefined') {
      return (
        localStorage.getItem('theme') === 'dark' ||
        (!localStorage.getItem('theme') &&
          window.matchMedia('(prefers-color-scheme: dark)').matches)
      )
    }
    return false
  })

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark')
      localStorage.setItem('theme', 'dark')
    } else {
      document.documentElement.classList.remove('dark')
      localStorage.setItem('theme', 'light')
    }
  }, [isDark])

  const toggleTheme = () => {
    setIsDark(!isDark)
  }

  return (
    <button
      onClick={toggleTheme}
      className={clsx(
        'flex items-center gap-2 px-3 py-2 rounded-lg transition-all duration-200 cursor-pointer',
        'text-gray-600 hover:bg-gray-100 hover:text-gray-900',
        'dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-gray-100',
        className
      )}
      title={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
    >
      <span className="text-lg">{isDark ? '☀️' : '🌙'}</span>
      <span className="text-sm font-medium">{isDark ? 'Light' : 'Dark'}</span>
    </button>
  )
}

export default ThemeToggle
