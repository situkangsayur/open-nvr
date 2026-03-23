import { defineStore } from 'pinia'

export const useThemeStore = defineStore('theme', {
  state: () => ({
    mode: 'dark' as 'dark' | 'light',
  }),

  actions: {
    toggle() {
      this.mode = this.mode === 'dark' ? 'light' : 'dark'
      this.apply()
      if (import.meta.client) {
        localStorage.setItem('opennvr-theme', this.mode)
      }
    },

    init() {
      if (import.meta.client) {
        const saved = localStorage.getItem('opennvr-theme')
        if (saved === 'light' || saved === 'dark') {
          this.mode = saved
        }
        this.apply()
      }
    },

    apply() {
      if (import.meta.client) {
        if (this.mode === 'dark') {
          document.documentElement.classList.add('dark')
        } else {
          document.documentElement.classList.remove('dark')
        }
      }
    },
  },
})
