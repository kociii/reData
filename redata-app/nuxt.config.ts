// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: true },
  modules: ['@nuxt/ui', '@pinia/nuxt'],
  ssr: false, // Tauri 需要 SPA 模式
  vite: {
    // Tauri 需要清除 host 和 strictPort
    clearScreen: false,
    server: {
      strictPort: true,
    },
    envPrefix: ['VITE_', 'TAURI_'],
  },
})
