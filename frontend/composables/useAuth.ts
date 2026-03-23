import { UserManager, WebStorageStateStore } from 'oidc-client-ts'

const authState = reactive({
  user: null as any,
  token: null as string | null,
  refreshToken: null as string | null,
  initialized: false,
})

let userManager: UserManager | null = null
let refreshTimer: ReturnType<typeof setTimeout> | null = null

export const useAuth = () => {
  const config = useRuntimeConfig()
  const token = computed(() => authState.token)
  const user = computed(() => authState.user)
  const isAuthenticated = computed(() => !!authState.token)

  const getUserManager = () => {
    if (!userManager && import.meta.client) {
      userManager = new UserManager({
        authority: `${config.public.keycloakUrl}/realms/${config.public.keycloakRealm}`,
        client_id: config.public.keycloakClientId,
        redirect_uri: `${window.location.origin}/auth/callback`,
        post_logout_redirect_uri: window.location.origin,
        response_type: 'code',
        scope: 'openid profile email',
        automaticSilentRenew: true,
        userStore: new WebStorageStateStore({ store: window.localStorage }),
      })
    }
    return userManager!
  }

  const init = async () => {
    if (authState.initialized || !import.meta.client) return

    try {
      // First try: check localStorage for direct login token
      const savedToken = localStorage.getItem('opennvr-token')
      const savedUser = localStorage.getItem('opennvr-user')
      const savedRefresh = localStorage.getItem('opennvr-refresh-token')

      if (savedToken && savedUser) {
        // Check if token is expired
        try {
          const payload = JSON.parse(savedUser)
          const exp = payload.exp * 1000
          if (Date.now() < exp) {
            authState.token = savedToken
            authState.user = payload
            authState.refreshToken = savedRefresh
            scheduleRefresh()
            authState.initialized = true
            return
          } else if (savedRefresh) {
            // Try to refresh
            const refreshed = await refreshAccessToken(savedRefresh)
            if (refreshed) {
              authState.initialized = true
              return
            }
          }
        } catch {}
        // Token expired and refresh failed, clear
        clearStorage()
      }

      // Second try: check OIDC session
      const mgr = getUserManager()
      const oidcUser = await mgr.getUser()
      if (oidcUser && !oidcUser.expired) {
        authState.user = oidcUser.profile
        authState.token = oidcUser.access_token
      }
    } catch (e) {
      console.error('Auth init failed', e)
    }
    authState.initialized = true
  }

  const refreshAccessToken = async (refreshToken: string): Promise<boolean> => {
    try {
      const tokenUrl = `${config.public.keycloakUrl}/realms/${config.public.keycloakRealm}/protocol/openid-connect/token`
      const body = new URLSearchParams({
        grant_type: 'refresh_token',
        client_id: config.public.keycloakClientId,
        refresh_token: refreshToken,
      })

      const response = await $fetch<any>(tokenUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: body.toString(),
      })

      if (response.access_token) {
        const payload = JSON.parse(atob(response.access_token.split('.')[1]))
        authState.token = response.access_token
        authState.user = payload
        authState.refreshToken = response.refresh_token || refreshToken

        localStorage.setItem('opennvr-token', response.access_token)
        localStorage.setItem('opennvr-refresh-token', authState.refreshToken || '')
        localStorage.setItem('opennvr-user', JSON.stringify(payload))

        scheduleRefresh()
        return true
      }
    } catch (e) {
      console.error('Token refresh failed', e)
    }
    return false
  }

  const scheduleRefresh = () => {
    if (refreshTimer) clearTimeout(refreshTimer)
    if (!authState.user?.exp) return

    // Refresh 30 seconds before expiry
    const expiresIn = (authState.user.exp * 1000) - Date.now() - 30000
    if (expiresIn > 0 && authState.refreshToken) {
      refreshTimer = setTimeout(() => {
        refreshAccessToken(authState.refreshToken!)
      }, expiresIn)
    }
  }

  const login = () => {
    if (!import.meta.client) return
    getUserManager().signinRedirect()
  }

  const handleCallback = async () => {
    if (!import.meta.client) return
    try {
      const user = await getUserManager().signinRedirectCallback()
      authState.user = user.profile
      authState.token = user.access_token
      navigateTo('/')
    } catch (e) {
      console.error('Callback failed', e)
      navigateTo('/login')
    }
  }

  const logout = () => {
    if (!import.meta.client) return
    authState.user = null
    authState.token = null
    authState.refreshToken = null
    if (refreshTimer) clearTimeout(refreshTimer)
    clearStorage()

    // Try OIDC logout, fallback to just navigating
    try {
      getUserManager().signoutRedirect()
    } catch {
      navigateTo('/login')
    }
  }

  const clearStorage = () => {
    if (!import.meta.client) return
    localStorage.removeItem('opennvr-token')
    localStorage.removeItem('opennvr-refresh-token')
    localStorage.removeItem('opennvr-user')
  }

  return { token, user, isAuthenticated, init, login, handleCallback, logout }
}
