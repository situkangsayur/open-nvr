export const useApi = async <T>(path: string, options?: any): Promise<T> => {
  const config = useRuntimeConfig()
  const auth = useAuth()

  const headers: Record<string, string> = {}
  const token = auth.token.value
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const response = await $fetch<T>(`${config.public.apiUrl}${path}`, {
    ...options,
    headers: {
      ...headers,
      ...options?.headers,
    },
  })

  return response
}
