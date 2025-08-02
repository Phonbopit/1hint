import { useState } from 'react'
import { Routes, Route } from 'react-router'
import DevInstructions from './components/DevInstructions'
import ProxySettings from './components/ProxySettings'
import ProxyStatus from './components/ProxyStatus'
import AnvilNode from './components/AnvilNode'
import Sidebar from './components/Sidebar'

import './App.css'

// Proxy page component
function ProxyPage() {
  const [proxyRunning, setProxyRunning] = useState(false)
  const [proxyUrl, setProxyUrl] = useState('')

  return (
    <div className="p-6">
      <div className="flex flex-col space-y-6">
        <h1 className="text-2xl font-bold text-gray-900">1hint - 1inch API Development Proxy</h1>

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
  )
}

// Anvil page component
function AnvilPage() {
  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">Local Ethereum Node (Anvil)</h1>
      <AnvilNode />
    </div>
  )
}

// Coming soon page component
function ComingSoonPage({ title }: { title: string }) {
  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-gray-900">{title}</h1>
      <p className="text-gray-600 mt-4">This page is coming soon...</p>
    </div>
  )
}

function App() {
  return (
    <div className="flex h-screen bg-gray-50">
      <Sidebar />

      <div className="flex-1 overflow-auto bg-gray-50">
        <Routes>
          <Route path="/" element={<ProxyPage />} />
          <Route path="/anvil" element={<AnvilPage />} />
          <Route path="/history" element={<ComingSoonPage title="Request History" />} />
        </Routes>
      </div>
    </div>
  )
}

export default App
