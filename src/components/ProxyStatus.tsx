import clsx from 'clsx'

interface ProxyStatusProps {
  running: boolean
  url: string
}

function ProxyStatus({ running, url }: ProxyStatusProps) {
  return (
    <div className="border border-gray-300 dark:border-gray-600 rounded-md p-4 bg-white dark:bg-gray-800">
      <div className="flex flex-col space-y-4">
        <h2 className="text-lg font-bold text-gray-900 dark:text-gray-100">Proxy Status</h2>

        <div className="flex items-center space-x-3">
          <div className={clsx('w-3 h-3 rounded-full', running ? 'bg-green-500' : 'bg-red-500')} />
          <span
            className={clsx(
              'font-medium',
              running ? 'text-green-700 dark:text-green-400' : 'text-red-700 dark:text-red-400'
            )}
          >
            {running ? 'Running' : 'Stopped'}
          </span>
        </div>

        {running && url && (
          <div className="flex flex-col space-y-2">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Proxy URL:</span>
            <code className="px-3 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded text-sm font-mono text-gray-900 dark:text-gray-100">
              {url}
            </code>
          </div>
        )}
      </div>
    </div>
  )
}

export default ProxyStatus
