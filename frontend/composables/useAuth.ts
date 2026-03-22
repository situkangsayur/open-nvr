import { UserManager, WebStorageStateStore } from 'oidc-client-ts'

const authState = reactive({
  user: null as any,
  token: null as string | null,
  initialized: false,
})

let userManager: UserManager | null = null

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
      const mgr = getUserManager()
      const user = await mgr.getUser()
      if (user && !user.expired) {
        authState.user = user.profile
        authState.token = user.access_token
      }
    } catch (e) {
      console.error('Auth init failed', e)
    }
    authState.initialized = true
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
    getUserManager().signoutRedirect()
  }

  return { token, user, isAuthenticated, init, login, handleCallback, logout }
}
