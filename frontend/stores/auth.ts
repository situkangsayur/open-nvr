import { defineStore } from 'pinia'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    token: null as string | null,
    user: null as any,
    initialized: false,
  }),

  getters: {
    isAuthenticated: (state) => !!state.token,
    isAdmin: (state) => state.user?.realm_access?.roles?.includes('admin') ?? false,
    userName: (state) => state.user?.preferred_username ?? state.user?.email ?? 'Unknown',
  },

  actions: {
    setAuth(token: string, user: any) {
      this.token = token
      this.user = user
    },

    clear() {
      this.token = null
      this.user = null
    },
  },
})
