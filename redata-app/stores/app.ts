import { defineStore } from 'pinia'

export const useAppStore = defineStore('app', {
  state: () => ({
    appName: 'reData',
    version: '0.1.0',
  }),
  
  getters: {
    fullName: (state) => `${state.appName} v${state.version}`,
  },
  
  actions: {
    setAppName(name: string) {
      this.appName = name
    },
  },
})
