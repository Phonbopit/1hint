import ThemeToggle from './ThemeToggle'

interface SidebarProps {
  currentPage: string
  onPageChange: (page: string) => void
}

// TODO: will implement React Router, and need to finalize features
function Sidebar({ currentPage, onPageChange }: SidebarProps) {
  const pages = [
    { id: 'proxy', label: 'CORS Proxy', icon: '🔗', url: '/proxy' },
    { id: 'tester', label: 'API Tester', icon: '🧪', url: '/tester' },
    { id: 'history', label: 'Request History', icon: '📊', url: '/history' },
    { id: 'analytics', label: 'Analytics', icon: '📈', url: '/analytics' },
    { id: 'docs', label: 'Documentation', icon: '📚', url: '/docs' },
    { id: 'settings', label: 'Settings', icon: '⚙️', url: '/settings' }
  ]

  return (
    <div className="h-full bg-gray-50 border-r border-gray-200 w-64 flex flex-col dark:bg-gray-800 dark:border-gray-700">
      <div className="flex-1 p-6">
        <div className="mb-8">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">1hint</h1>
          <p className="text-sm text-gray-600 mt-1 dark:text-gray-400">1inch Dev Tools</p>
        </div>

        <nav className="space-y-1">
          {pages.map((page) => (
            <a
              key={page.id}
              href={page.url}
              onClick={(e) => {
                e.preventDefault()
                onPageChange(page.id)
              }}
              className={`
                w-full flex items-center gap-3 px-3 py-3 rounded-lg text-left transition-all duration-200 no-underline
                ${
                  currentPage === page.id
                    ? 'bg-blue-600 text-white shadow-sm'
                    : 'text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-gray-100'
                }
              `}
            >
              <span className="text-lg">{page.icon}</span>
              <span className="font-medium">{page.label}</span>
            </a>
          ))}
        </nav>
      </div>

      <div className="p-6 border-t border-gray-200 dark:border-gray-700">
        <ThemeToggle className="w-full" />
      </div>
    </div>
  )
}

export default Sidebar
