import { Link, useLocation } from 'react-router'

function Sidebar() {
  const location = useLocation()
  const pages = [
    { id: 'proxy', label: 'CORS Proxy', icon: '🔗', url: '/' },
    { id: 'anvil', label: 'Local Node (Anvil)', icon: '⛏️', url: '/anvil' },
    { id: 'history', label: 'Request History', icon: '📊', url: '/history' }
  ]

  return (
    <div className="h-full bg-gray-50 border-r border-gray-200 w-64 flex flex-col">
      <div className="flex-1 p-6">
        <div className="mb-8">
          <h1 className="text-2xl font-bold text-gray-900">1hint</h1>
          <p className="text-sm text-gray-600 mt-1">1inch Dev Tools</p>
        </div>

        <nav className="space-y-1">
          {pages.map((page) => (
            <Link
              key={page.id}
              to={page.url}
              className={`
                w-full flex items-center gap-3 px-3 py-3 rounded-lg text-left transition-all duration-200 no-underline
                ${
                  location.pathname === page.url
                    ? 'bg-indigo-600 text-white shadow-sm'
                    : 'text-gray-700 hover:bg-gray-100 hover:text-gray-900'
                }
              `}
            >
              <span className="text-lg">{page.icon}</span>
              <span className="font-medium">{page.label}</span>
            </Link>
          ))}
        </nav>
      </div>
    </div>
  )
}

export default Sidebar
