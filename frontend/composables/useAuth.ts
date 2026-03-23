/**
 * Auth composable — single source of truth for authentication state.
 * Supports both direct login (username/password via ROPC) and OIDC redirect.
 * Token is persisted in localStorage and auto-refreshed before expiry.
 */

const authState = reactive({
  user: null as any,
  token: null as string | null,
  refreshToken: null as string | null,
  initialized: false,
})

let refreshTimer: ReturnType<typeof setTimeout> | null = null

export const useAuth = () => {
  const config = useRuntimeConfig()
  const token = computed(() => authState.token)
  const user = computed(() => authState.user)
  const isAuthenticated = computed(() => !!authState.token)

  const init = async () => {
    if (authState.initialized || !import.meta.client) return

    try {
      const savedToken = localStorage.getItem('opennvr-token')
      const savedUser = localStorage.getItem('opennvr-user')
      const savedRefresh = localStorage.getItem('opennvr-refresh-token')

      if (savedToken && savedUser) {
        const payload = JSON.parse(savedUser)
        const exp = payload.exp * 1000

        if (Date.now() < exp) {
          // Token still valid
          authState.token = savedToken
          authState.user = payload
          authState.refreshToken = savedRefresh
          scheduleRefresh()
        } else if (savedRefresh) {
          // Try refresh
          await refreshAccessToken(savedRefresh)
        } else {
          clearStorage()
        }
      }
    } catch (e) {
      console.error('Auth init failed', e)
      clearStorage()
    }
    authState.initialized = true
  }

  /** Direct login with username/password (Keycloak ROPC grant) */
  const loginDirect = async (username: string, password: string): Promise<void> => {
    const tokenUrl = `${config.public.keycloakUrl}/realms/${config.public.keycloakRealm}/protocol/openid-connect/token`

    const body = new URLSearchParams({
      grant_type: 'password',
      client_id: config.public.keycloakClientId,
      username,
      password,
      scope: 'openid profile email',
    })

    const response = await $fetch<any>(tokenUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: body.toString(),
    })

    if (response.access_token) {
      setTokens(response.access_token, response.refresh_token)
    } else {
      throw new Error('No access token in response')
    }
  }

  /** SSO redirect login via Keycloak */
  const loginSSO = () => {
    if (!import.meta.client) return
    const params = new URLSearchParams({
      client_id: config.public.keycloakClientId,
      redirect_uri: `${window.location.origin}/auth/callback`,
      response_type: 'code',
      scope: 'openid profile email',
    })
    window.location.href = `${config.public.keycloakUrl}/realms/${config.public.keycloakRealm}/protocol/openid-connect/auth?${params}`
  }

  /** Handle OIDC callback — exchange code for tokens */
  const handleCallback = async () => {
    if (!import.meta.client) return

    const urlParams = new URLSearchParams(window.location.search)
    const code = urlParams.get('code')
    if (!code) {
      navigateTo('/login')
      return
    }

    try {
      const tokenUrl = `${config.public.keycloakUrl}/realms/${config.public.keycloakRealm}/protocol/openid-connect/token`
      const body = new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: config.public.keycloakClientId,
        code,
        redirect_uri: `${window.location.origin}/auth/callback`,
      })

      const response = await $fetch<any>(tokenUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: body.toString(),
      })

      if (response.access_token) {
        setTokens(response.access_token, response.refresh_token)
        navigateTo('/')
      } else {
        navigateTo('/login')
      }
    } catch (e) {
      console.error('Callback failed', e)
      navigateTo('/login')
    }
  }

  const setTokens = (accessToken: string, refreshToken?: string) => {
    const payload = JSON.parse(atob(accessToken.split('.')[1]))
    authState.token = accessToken
    authState.user = payload
    authState.refreshToken = refreshToken || null

    if (import.meta.client) {
      localStorage.setItem('opennvr-token', accessToken)
      localStorage.setItem('opennvr-user', JSON.stringify(payload))
      if (refreshToken) {
        localStorage.setItem('opennvr-refresh-token', refreshToken)
      }
    }

    scheduleRefresh()
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
        setTokens(response.access_token, response.refresh_token || refreshToken)
        return true
      }
    } catch (e) {
      console.error('Token refresh failed', e)
    }
    clearStorage()
    return false
  }

  const scheduleRefresh = () => {
    if (refreshTimer) clearTimeout(refreshTimer)
    if (!authState.user?.exp || !authState.refreshToken) return

    const expiresIn = (authState.user.exp * 1000) - Date.now() - 30000
    if (expiresIn > 0) {
      refreshTimer = setTimeout(() => {
        if (authState.refreshToken) {
          refreshAccessToken(authState.refreshToken)
        }
      }, expiresIn)
    }
  }

  const logout = () => {
    authState.user = null
    authState.token = null
    authState.refreshToken = null
    if (refreshTimer) clearTimeout(refreshTimer)
    clearStorage()

    if (import.meta.client) {
      // Keycloak logout
      const logoutUrl = `${config.public.keycloakUrl}/realms/${config.public.keycloakRealm}/protocol/openid-connect/logout`
      const params = new URLSearchParams({
        client_id: config.public.keycloakClientId,
        post_logout_redirect_uri: `${window.location.origin}/login`,
      })
      window.location.href = `${logoutUrl}?${params}`
    }
  }

  const clearStorage = () => {
    if (!import.meta.client) return
    localStorage.removeItem('opennvr-token')
    localStorage.removeItem('opennvr-refresh-token')
    localStorage.removeItem('opennvr-user')
  }

  return { token, user, isAuthenticated, init, loginDirect, loginSSO, handleCallback, logout }
}
