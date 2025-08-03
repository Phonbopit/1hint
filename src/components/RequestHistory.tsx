import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect } from 'react'

const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__

interface ApiRequest {
  id: string
  timestamp: string
  method: string
  url: string
  status: number | null
  duration_ms: number | null
  request_body: string | null
  response_body: string | null
  error: string | null
}

function RequestHistory() {
  const [requests, setRequests] = useState<ApiRequest[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [selectedRequest, setSelectedRequest] = useState<ApiRequest | null>(null)

  const fetchRequests = async () => {
    if (!isTauri) {
      setError('Request history can only be viewed in the desktop app.')
      setLoading(false)
      return
    }

    try {
      const history = await invoke('get_request_history')
      setRequests(history as ApiRequest[])
      setError('')
    } catch (err) {
      setError((err as Error).toString())
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchRequests()
  }, [])

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString()
  }

  const getStatusColor = (status: number | null) => {
    if (!status) return 'text-gray-500'
    if (status >= 200 && status < 300) return 'text-green-600'
    if (status >= 400 && status < 500) return 'text-yellow-600'
    if (status >= 500) return 'text-red-600'
    return 'text-gray-600'
  }

  const truncateUrl = (url: string) => {
    if (url.length > 60) {
      return url.substring(0, 60) + '...'
    }
    return url
  }

  if (loading) {
    return (
      <div className="p-6">
        <div className="flex items-center justify-center py-12">
          <svg className="animate-spin h-8 w-8 text-indigo-600" fill="none" viewBox="0 0 24 24">
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
          <span className="ml-2 text-gray-600">Loading request history...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="p-6">
      <div className="flex flex-col space-y-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900">Request History</h1>
          <button
            onClick={fetchRequests}
            className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 cursor-pointer"
          >
            Refresh
          </button>
        </div>

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

        {requests.length === 0 && !error && (
          <div className="text-center py-12">
            <div className="text-gray-500 text-lg">No requests logged yet</div>
            <div className="text-gray-400 text-sm mt-2">
              Start the proxy server and make some API calls to see them here
            </div>
          </div>
        )}

        {requests.length > 0 && (
          <div className="border border-gray-300 rounded-md overflow-hidden">
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Timestamp
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Method
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      URL
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Status
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Duration
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {requests.map((request) => (
                    <tr key={request.id} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {formatTimestamp(request.timestamp)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span
                          className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                            request.method === 'GET'
                              ? 'bg-blue-100 text-blue-800'
                              : 'bg-green-100 text-green-800'
                          }`}
                        >
                          {request.method}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900 max-w-xs">
                        <div title={request.url}>{truncateUrl(request.url)}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm">
                        <span className={getStatusColor(request.status)}>
                          {request.status || 'Error'}
                        </span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {request.duration_ms ? `${request.duration_ms}ms` : '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm">
                        <button
                          onClick={() => setSelectedRequest(request)}
                          className="text-indigo-600 hover:text-indigo-900 font-medium"
                        >
                          View Details
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {selectedRequest && (
          <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
            <div className="relative top-20 mx-auto p-5 border w-11/12 max-w-4xl shadow-lg rounded-md bg-white">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-lg font-bold text-gray-900">Request Details</h3>
                <button
                  onClick={() => setSelectedRequest(null)}
                  className="text-gray-400 hover:text-gray-600"
                >
                  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              </div>

              <div className="space-y-4">
                <div>
                  <h4 className="font-semibold text-gray-700">Request Info</h4>
                  <div className="mt-2 bg-gray-50 p-3 rounded">
                    <p>
                      <strong>ID:</strong> {selectedRequest.id}
                    </p>
                    <p>
                      <strong>Method:</strong> {selectedRequest.method}
                    </p>
                    <p>
                      <strong>URL:</strong> {selectedRequest.url}
                    </p>
                    <p>
                      <strong>Timestamp:</strong> {formatTimestamp(selectedRequest.timestamp)}
                    </p>
                    <p>
                      <strong>Status:</strong> {selectedRequest.status || 'Error'}
                    </p>
                    <p>
                      <strong>Duration:</strong>{' '}
                      {selectedRequest.duration_ms ? `${selectedRequest.duration_ms}ms` : 'N/A'}
                    </p>
                  </div>
                </div>

                {selectedRequest.request_body && (
                  <div>
                    <h4 className="font-semibold text-gray-700">Request Body</h4>
                    <pre className="mt-2 bg-gray-50 p-3 rounded text-sm overflow-x-auto">
                      {JSON.stringify(JSON.parse(selectedRequest.request_body), null, 2)}
                    </pre>
                  </div>
                )}

                {selectedRequest.response_body && (
                  <div>
                    <h4 className="font-semibold text-gray-700">Response Body</h4>
                    <pre className="mt-2 bg-gray-50 p-3 rounded text-sm overflow-x-auto max-h-96">
                      {JSON.stringify(JSON.parse(selectedRequest.response_body), null, 2)}
                    </pre>
                  </div>
                )}

                {selectedRequest.error && (
                  <div>
                    <h4 className="font-semibold text-gray-700">Error</h4>
                    <div className="mt-2 bg-red-50 border border-red-200 p-3 rounded text-red-700">
                      {selectedRequest.error}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default RequestHistory
