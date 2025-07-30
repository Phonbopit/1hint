import { invoke } from '@tauri-apps/api/core'
import { useState } from 'react'

interface ProxySettingsProps {
  onProxyStart: (url: string) => void
  onProxyStop: () => void
}

function ProxySettings({ onProxyStart, onProxyStop }: ProxySettingsProps) {
  const [apiKey, setApiKey] = useState('')
  const [port, setPort] = useState(8888)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [proxyRunning, setProxyRunning] = useState(false)

  const handleStart = async () => {
    setLoading(true)
    setError('')

    try {
      // Store API key
      await invoke('store_api_key', { key: apiKey })

      // Test API key
      const isValid = await invoke('test_api_key')
      if (!isValid) {
        throw new Error('Invalid API key')
      }

      // Start proxy server
      const proxyUrl = await invoke('start_proxy_server', { port })

      setProxyRunning(true)
      onProxyStart(proxyUrl as string)
    } catch (err) {
      setError((err as Error).toString())
    } finally {
      setLoading(false)
    }
  }

  const handleStop = async () => {
    try {
      await invoke('stop_proxy_server')
      setProxyRunning(false)
      onProxyStop()
    } catch (err) {
      setError((err as Error).toString())
    }
  }

  return (
    <div className="border border-gray-300 rounded-md p-4">
      <div className="flex flex-col space-y-4">
        <h2 className="text-lg font-bold">Proxy Configuration</h2>

        {error && (
          <div className="flex items-center p-4 bg-red-50 border border-red-200 rounded-md">
            <svg className="w-5 h-5 text-red-500 mr-3" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                clipRule="evenodd"
              />
            </svg>
            <span className="text-red-700">{error}</span>
          </div>
        )}

        <div className="flex flex-col space-y-2">
          <label className="text-sm font-medium text-gray-700">1inch API Key</label>
          <input
            type="password"
            placeholder="Enter your 1inch API key"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            disabled={proxyRunning}
            className="px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
          />
        </div>

        <div className="flex flex-col space-y-2">
          <label className="text-sm font-medium text-gray-700">Port</label>
          <input
            type="number"
            value={port}
            onChange={(e) => setPort(parseInt(e.target.value))}
            disabled={proxyRunning}
            className="px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
          />
        </div>

        <div className="flex space-x-2">
          {!proxyRunning ? (
            <button
              onClick={handleStart}
              disabled={!apiKey || loading}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed flex items-center"
            >
              {loading && (
                <svg
                  className="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                  ></circle>
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  ></path>
                </svg>
              )}
              Start Proxy Server
            </button>
          ) : (
            <button
              onClick={handleStop}
              className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2"
            >
              Stop Proxy Server
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

export default ProxySettings
