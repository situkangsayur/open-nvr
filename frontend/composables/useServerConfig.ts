/**
 * Server endpoint configuration — lets the user point the app at a different
 * Open-NVR server from the login screen.
 *
 * `NUXT_PUBLIC_API_URL` and friends are baked into the bundle at build time, so
 * they cannot cover the "at home vs. away" case on their own. This composable
 * layers a user-chosen override (persisted in localStorage) on top of those
 * build-time values, which stay as the fallback.
 */

export interface ServerEndpoints {
  apiUrl: string
  keycloakUrl: string
}

const STORAGE_KEY = 'opennvr-server'

/** Module-level so every consumer sees the same endpoints without prop drilling. */
const state = reactive({
  apiUrl: null as string | null,
  keycloakUrl: null as string | null,
  loaded: false,
})

/** Strip a trailing slash so `${base}/api/x` never doubles up. */
const normalise = (url: string): string => url.trim().replace(/\/+$/, '')

/**
 * Build a base URL from loosely typed user input. Accepts bare hosts
 * ("192.168.1.10"), hosts with a scheme, and hosts that already carry a port.
 * An explicit `port` wins over one embedded in `host`.
 */
export const buildBaseUrl = (host: string, port?: string | number): string => {
  let value = host.trim()
  if (!value) return ''

  let scheme = 'http://'
  const schemeMatch = value.match(/^(https?:\/\/)/i)
  if (schemeMatch) {
    scheme = schemeMatch[1].toLowerCase()
    value = value.slice(schemeMatch[1].length)
  }

  // Drop any path the user pasted in, then split off an embedded port.
  value = value.split('/')[0]
  const [hostname, embeddedPort] = value.split(':')
  const finalPort = port !== undefined && `${port}`.trim() !== '' ? `${port}`.trim() : embeddedPort

  return normalise(finalPort ? `${scheme}${hostname}:${finalPort}` : `${scheme}${hostname}`)
}

/** Split a base URL back into parts, for pre-filling the login form. */
export const splitBaseUrl = (url: string): { host: string; port: string; secure: boolean } => {
  try {
    const parsed = new URL(url)
    return {
      host: parsed.hostname,
      port: parsed.port || (parsed.protocol === 'https:' ? '443' : '80'),
      secure: parsed.protocol === 'https:',
    }
  } catch {
    return { host: '', port: '', secure: false }
  }
}

export const useServerConfig = () => {
  const runtime = useRuntimeConfig()

  // Build-time values remain the fallback when the user has not chosen a server.
  const fallback: ServerEndpoints = {
    apiUrl: normalise(runtime.public.apiUrl as string),
    keycloakUrl: normalise(runtime.public.keycloakUrl as string),
  }

  const load = () => {
    if (!import.meta.client || state.loaded) return
    state.loaded = true
    try {
      const raw = localStorage.getItem(STORAGE_KEY)
      if (!raw) return
      const saved = JSON.parse(raw) as Partial<ServerEndpoints>
      if (saved.apiUrl) state.apiUrl = normalise(saved.apiUrl)
      if (saved.keycloakUrl) state.keycloakUrl = normalise(saved.keycloakUrl)
    } catch (e) {
      console.error('Could not read saved server config', e)
      localStorage.removeItem(STORAGE_KEY)
    }
  }

  load()

  const apiUrl = computed(() => state.apiUrl || fallback.apiUrl)
  const keycloakUrl = computed(() => state.keycloakUrl || fallback.keycloakUrl)
  /** True when the user has overridden the build-time endpoints. */
  const isCustom = computed(() => !!state.apiUrl)

  const save = (endpoints: ServerEndpoints) => {
    state.apiUrl = normalise(endpoints.apiUrl)
    state.keycloakUrl = normalise(endpoints.keycloakUrl)
    if (import.meta.client) {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({ apiUrl: state.apiUrl, keycloakUrl: state.keycloakUrl }),
      )
    }
  }

  /** Forget the override and fall back to the build-time endpoints. */
  const reset = () => {
    state.apiUrl = null
    state.keycloakUrl = null
    if (import.meta.client) localStorage.removeItem(STORAGE_KEY)
  }

  /**
   * Probe `GET {base}/health` so a wrong address is reported as a wrong
   * address, instead of surfacing later as a confusing login failure.
   */
  const testConnection = async (
    base?: string,
  ): Promise<{ ok: boolean; message: string; version?: string }> => {
    const target = normalise(base || apiUrl.value)
    if (!target) return { ok: false, message: 'Server address is empty' }

    try {
      const controller = new AbortController()
      const timeout = setTimeout(() => controller.abort(), 8000)
      const response = await fetch(`${target}/health`, { signal: controller.signal })
      clearTimeout(timeout)

      if (!response.ok) {
        return { ok: false, message: `Server answered HTTP ${response.status}` }
      }

      const body = await response.json()
      if (body?.status !== 'ok') {
        return { ok: false, message: `Unexpected health response: ${JSON.stringify(body)}` }
      }
      return { ok: true, message: `Connected to ${body.service} v${body.version}`, version: body.version }
    } catch (e: any) {
      if (e?.name === 'AbortError') {
        return { ok: false, message: 'Timed out — server unreachable from this network' }
      }
      // A cross-origin block and an unreachable host are indistinguishable to
      // fetch, so name both possibilities rather than guessing wrong.
      return {
        ok: false,
        message: 'Cannot reach server. Check the address, or that this origin is in CORS_ALLOWED_ORIGINS.',
      }
    }
  }

  return { apiUrl, keycloakUrl, isCustom, fallback, save, reset, testConnection }
}
