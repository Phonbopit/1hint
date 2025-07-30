import { useState } from 'react'
import DevInstructions from './components/DevInstructions'
import ProxySettings from './components/ProxySettings'
import ProxyStatus from './components/ProxyStatus'
import Sidebar from './components/Sidebar'

import './App.css'

function App() {
  const [proxyRunning, setProxyRunning] = useState(false)
  const [proxyUrl, setProxyUrl] = useState('')
  const [currentPage, setCurrentPage] = useState('proxy')

  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-800">
      <Sidebar currentPage={currentPage} onPageChange={setCurrentPage} />

      <div className="flex-1 overflow-auto bg-gray-50 dark:bg-gray-800">
        <div className="p-6">
          <div className="flex flex-col space-y-6">
            <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              1hint - 1inch API Development Proxy
            </h1>

            <ProxySettings
              onProxyStart={(url) => {
                setProxyUrl(url)
                setProxyRunning(true)
              }}
              onProxyStop={() => {
                setProxyRunning(false)
                setProxyUrl('')
              }}
            />

            <ProxyStatus running={proxyRunning} url={proxyUrl} />

            {proxyRunning && <DevInstructions proxyUrl={proxyUrl} />}
          </div>
        </div>
      </div>
    </div>
  )
}

export default App
