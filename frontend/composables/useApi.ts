export const useApi = async <T>(path: string, options?: any): Promise<T> => {
  const { apiUrl } = useServerConfig()
  const { token } = useAuth()

  const headers: Record<string, string> = {}
  if (token.value) {
    headers['Authorization'] = `Bearer ${token.value}`
  }

  try {
    const response = await $fetch<T>(`${apiUrl.value}${path}`, {
      ...options,
      headers: {
        ...headers,
        ...options?.headers,
      },
    })
    return response
  } catch (e: any) {
    // If 401, redirect to login
    if (e?.status === 401 || e?.response?.status === 401) {
      if (import.meta.client) {
        localStorage.removeItem('opennvr-token')
        localStorage.removeItem('opennvr-user')
        localStorage.removeItem('opennvr-refresh-token')
        navigateTo('/login')
      }
    }
    throw e
  }
}
