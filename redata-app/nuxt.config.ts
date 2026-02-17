// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false }, // 禁用 devtools 避免 fork 问题
  modules: ['@nuxt/ui', '@pinia/nuxt'],
  ssr: false, // Tauri 需要 SPA 模式

  // Nuxt UI 配置 - 使用国内字体镜像
  ui: {
    fonts: {
      fallbacks: false,
      families: [
        { name: 'Inter', weights: ['400', '500', '600', '700'] },
      ],
    },
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
