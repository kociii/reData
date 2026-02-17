// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false }, // 禁用 devtools 避免 fork 问题
  modules: ['@nuxt/ui', '@pinia/nuxt'],
  ssr: false, // Tauri 需要 SPA 模式
  nitro: {
    // 禁用实验性功能避免 Node.js 23 的 fork 问题
    experimental: {
      tasks: false,
    },
  },
  vite: {
    // Tauri 需要清除 host 和 strictPort
    clearScreen: false,
    server: {
      strictPort: true,
    },
    envPrefix: ['VITE_', 'TAURI_'],
  },
})
