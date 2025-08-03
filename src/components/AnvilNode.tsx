import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect } from 'react'

const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__

interface NodeStatus {
  is_running: boolean
  url?: string
  block_number?: number
  chain_id?: number
  gas_price?: string
}

function AnvilNode() {
  const [port, setPort] = useState(8545)
  const [chainId, setChainId] = useState(31337)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [nodeRunning, setNodeRunning] = useState(false)
  const [nodeUrl, setNodeUrl] = useState('')
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null)

  useEffect(() => {
    let interval: number | null = null

    const checkNodeStatus = async () => {
      if (!isTauri || !nodeRunning || !nodeUrl) return

      try {
        const status = (await invoke('get_node_status', { rpcUrl: nodeUrl })) as NodeStatus
        setNodeStatus(status)
        console.log('Periodic status check:', status)
      } catch (err) {
        console.log('Periodic status check failed:', err)
        setNodeStatus(null)
        setNodeRunning(false)
        setNodeUrl('')
      }
    }

    if (nodeRunning && nodeUrl) {
      checkNodeStatus()
      interval = setInterval(checkNodeStatus, 5000)
    }

    return () => {
      if (interval) {
        clearInterval(interval)
      }
    }
  }, [port, nodeRunning, isTauri])

  const handleStart = async () => {
    setLoading(true)
    setError('')

    if (!isTauri) {
      setError('Anvil node can only be started in the desktop app, not in the web browser.')
      setLoading(false)
      return
    }

    try {
      const result = await invoke('start_anvil_node', { port, chainId })

      setNodeRunning(true)
      setNodeUrl(result as string)

      setTimeout(async () => {
        try {
          const status = (await invoke('get_node_status', { rpcUrl: result })) as NodeStatus
          setNodeStatus(status)
          console.log('Got node status:', status)
        } catch (err) {
          console.error('Failed to get initial status:', err)
          setError(`Failed to get node status: ${err}`)
        }
      }, 3000)
    } catch (err) {
      const errorMsg = (err as Error).toString()
      if (errorMsg.includes('already running')) {
        setError(
          'Anvil node is already running on this port. Use stop first or choose a different port.'
        )
      } else {
        setError(errorMsg)
      }
      console.error('Anvil start error:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleStop = async () => {
    if (!isTauri) {
      setError('Anvil node can only be controlled in the desktop app.')
      return
    }

    try {
      await invoke('stop_anvil_node')
      setNodeRunning(false)
      setNodeUrl('')
      setNodeStatus(null)
    } catch (err) {
      setError((err as Error).toString())
    }
  }

  return (
    <div className="p-6 space-y-6">
      <div className="border border-gray-300 rounded-md p-4">
        <h2 className="text-lg font-bold mb-4">Anvil Local Ethereum Node</h2>

        {error && (
          <div className="flex items-center p-4 bg-red-50 border border-red-200 rounded-md mb-4">
            <svg className="w-5 h-5 text-rose-500 mr-3" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                clipRule="evenodd"
              />
            </svg>
            <span className="text-rose-700">{error}</span>
          </div>
        )}

        <div className="grid grid-cols-2 gap-4 mb-4">
          <div className="flex flex-col space-y-2">
            <label className="text-sm font-medium text-gray-700">Port</label>
            <input
              type="number"
              value={port}
              onChange={(e) => setPort(parseInt(e.target.value))}
              disabled={nodeRunning || loading}
              className="px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
            />
          </div>

          <div className="flex flex-col space-y-2">
            <label className="text-sm font-medium text-gray-700">Chain ID</label>
            <input
              type="number"
              value={chainId}
              onChange={(e) => setChainId(parseInt(e.target.value))}
              disabled={nodeRunning || loading}
              className="px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
            />
          </div>
        </div>

        <div className="flex space-x-2">
          {!nodeRunning ? (
            <button
              onClick={handleStart}
              disabled={loading}
              className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed cursor-pointer flex items-center"
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
              {loading ? 'Starting...' : 'Start Local Node'}
            </button>
          ) : (
            <button
              onClick={handleStop}
              disabled={loading}
              className="px-4 py-2 bg-rose-600 text-white rounded-md hover:bg-rose-700 focus:outline-none focus:ring-2 focus:ring-rose-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed cursor-pointer"
            >
              Stop Local Node
            </button>
          )}
        </div>

        {nodeRunning && nodeStatus && (
          <div className="mt-4 space-y-3">
            <div className="p-3 bg-green-50 border border-green-200 rounded-md">
              <div className="flex items-center mb-2">
                <div className="w-2 h-2 bg-teal-500 rounded-full mr-2"></div>
                <span className="text-teal-700 font-medium">Node running at: {nodeUrl}</span>
              </div>

              <div className="grid grid-cols-3 gap-4 text-sm">
                <div className="bg-white p-2 rounded border">
                  <div className="text-gray-600 text-xs">Block Number</div>
                  <div className="font-mono font-medium">
                    {nodeStatus.block_number !== undefined ? nodeStatus.block_number : 'N/A'}
                  </div>
                </div>

                <div className="bg-white p-2 rounded border">
                  <div className="text-gray-600 text-xs">Chain ID</div>
                  <div className="font-mono font-medium">
                    {nodeStatus.chain_id !== undefined ? nodeStatus.chain_id : 'N/A'}
                  </div>
                </div>

                <div className="bg-white p-2 rounded border">
                  <div className="text-gray-600 text-xs">Gas Price</div>
                  <div className="font-mono font-medium text-xs">
                    {nodeStatus.gas_price || 'N/A'}
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default AnvilNode
