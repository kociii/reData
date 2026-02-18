// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false }, // 禁用 devtools 避免 fork 问题
  modules: ['@nuxt/ui', '@pinia/nuxt'],
  css: ['~/assets/css/main.css'],
  ssr: false, // Tauri 需要 SPA 模式

  // Nuxt UI 配置 - 使用系统字体
  ui: {
    fonts: false, // 禁用内置字体，使用系统字体
  },

  // 自定义 head 配置
  app: {
    head: {
      link: [
        {
          rel: 'icon',
          type: 'image/x-icon',
          href: '/favicon.ico',
        },
        {
          rel: 'icon',
          type: 'image/png',
          sizes: '32x32',
          href: '/favicon-32x32.png',
        },
        {
          rel: 'icon',
          type: 'image/png',
          sizes: '16x16',
          href: '/favicon-16x16.png',
        },
        {
          rel: 'apple-touch-icon',
          sizes: '180x180',
          href: '/apple-touch-icon.png',
        },
      ],
    },
  },

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
