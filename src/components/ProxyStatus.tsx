import clsx from 'clsx'

interface ProxyStatusProps {
  running: boolean
  url: string
}

function ProxyStatus({ running, url }: ProxyStatusProps) {
  return (
    <div className="border border-gray-300 rounded-md p-4 bg-white">
      <div className="flex flex-col space-y-4">
        <h2 className="text-lg font-bold text-gray-900">Proxy Status</h2>

        <div className="flex items-center space-x-3">
          <div className={clsx('w-3 h-3 rounded-full', running ? 'bg-teal-500' : 'bg-rose-500')} />
          <span className={clsx('font-medium', running ? 'text-teal-700' : 'text-rose-700')}>
            {running ? 'Running' : 'Stopped'}
          </span>
        </div>

        {running && url && (
          <div className="flex flex-col space-y-2">
            <span className="text-sm font-medium text-gray-700">Proxy URL:</span>
            <code className="px-3 py-2 bg-gray-100 border border-gray-300 rounded text-sm font-mono text-gray-900">
              {url}
            </code>
          </div>
        )}
      </div>
    </div>
  )
}

export default ProxyStatus
