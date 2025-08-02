/// <reference types="vite/client" />

declare global {
  interface Window {
    __TAURI__?: any
  }
}

declare namespace NodeJS {
  interface Timeout {}
}
