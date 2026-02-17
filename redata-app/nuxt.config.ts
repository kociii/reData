// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false }, // 禁用 devtools 避免 fork 问题
  modules: ['@nuxt/ui', '@pinia/nuxt'],
  css: ['~/assets/css/main.css'],
  ssr: false, // Tauri 需要 SPA 模式

  // Nuxt UI 配置 - 禁用自动字体加载（使用国内镜像）
  ui: {
    fonts: false, // 完全禁用内置字体功能
  },

  // 自定义 head 配置 - 使用国内 Google Fonts 镜像
  app: {
    head: {
      link: [
        {
          rel: 'preconnect',
          href: 'https://www.googlefonts.cn',
        },
        {
          rel: 'preconnect',
          href: 'https://www.googlefonts.cn',
          crossorigin: 'anonymous',
        },
        {
          rel: 'stylesheet',
          href: 'https://www.googlefonts.cn/css2?family=Inter:wght@400;500;600;700&display=swap',
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
