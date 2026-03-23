export default defineNuxtRouteMiddleware(async (to) => {
  // Skip auth for login and callback pages
  if (to.path === '/login' || to.path.startsWith('/auth/')) return

  // Only check auth on client side (localStorage is not available on SSR)
  if (!import.meta.client) return

  const { isAuthenticated, init } = useAuth()

  await init()

  if (!isAuthenticated.value) {
    return navigateTo('/login')
  }
})
