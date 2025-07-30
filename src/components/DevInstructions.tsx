import { useState } from 'react'
import { proxyUsageExample } from '../examples/proxy-usage'

interface DevInstructionsProps {
  proxyUrl: string
}

function DevInstructions({ proxyUrl }: DevInstructionsProps) {
  const [copied, setCopied] = useState(false)
  const exampleCode = proxyUsageExample.replace('{PROXY_URL}', proxyUrl)

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(exampleCode)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (error) {
      console.error('Failed to copy:', error)
    }
  }

  return (
    <div className="border border-gray-300 rounded-md p-4 bg-gray-50">
      <div className="flex flex-col space-y-4">
        <h2 className="text-lg font-bold text-gray-700">Developer Instructions</h2>

        <p className="text-gray-700">
          Your proxy server is running at:{' '}
          <code className="px-2 py-1 bg-gray-200 rounded text-sm font-mono">{proxyUrl}</code>
        </p>

        <p className="text-gray-700">
          Use this base URL in your frontend application instead of the 1inch API directly:
        </p>

        <div className="bg-black text-green-400 p-4 rounded-md font-mono text-sm overflow-x-auto">
          <pre>{exampleCode}</pre>
        </div>

        <button
          onClick={copyToClipboard}
          className={`self-start px-3 py-1 text-sm rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 transition-colors duration-200 cursor-pointer ${
            copied
              ? 'bg-green-600 text-white hover:bg-green-700 focus:ring-green-500'
              : 'bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500'
          }`}
        >
          {copied ? '✓ Copied!' : 'Copy Code Example'}
        </button>
      </div>
    </div>
  )
}

export default DevInstructions
