<template>
  <div class="w-full max-w-md mx-auto text-center">
    <div class="bg-white dark:bg-nvr-card rounded-xl p-8 border border-gray-200 dark:border-nvr-border shadow-xl">
      <div class="mb-6">
        <div class="w-16 h-16 bg-primary-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
          <span class="text-2xl font-bold text-white">NVR</span>
        </div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Open-NVR</h1>
        <p class="text-gray-500 dark:text-gray-400 mt-1">Network Video Recorder</p>
      </div>

      <div v-if="errorMsg" class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-sm text-red-600 dark:text-red-400">
        {{ errorMsg }}
      </div>

      <div v-if="serverMsg" :class="[
          'mb-4 p-3 border rounded-lg text-sm',
          serverOk
            ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800 text-green-700 dark:text-green-400'
            : 'bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800 text-amber-700 dark:text-amber-400',
        ]">
        {{ serverMsg }}
      </div>

      <form @submit.prevent="handleLogin" class="space-y-4 text-left">
        <!-- Server selection: lets the same build reach the NVR at home or remotely -->
        <div class="rounded-lg border border-gray-200 dark:border-nvr-border overflow-hidden">
          <button type="button" @click="showServer = !showServer"
            class="w-full flex items-center justify-between px-4 py-3 bg-gray-50 dark:bg-nvr-darker hover:bg-gray-100 dark:hover:bg-nvr-border transition-colors">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Server</span>
            <span class="flex items-center gap-2 min-w-0">
              <span class="text-xs text-gray-500 dark:text-gray-400 truncate">{{ serverHost || 'not set' }}</span>
              <span class="text-gray-400 text-xs shrink-0">{{ showServer ? '▲' : '▼' }}</span>
            </span>
          </button>

          <div v-show="showServer" class="p-4 space-y-3 border-t border-gray-200 dark:border-nvr-border">
            <div class="flex gap-2">
              <div class="flex-1 min-w-0">
                <label class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">Host / IP</label>
                <input v-model="serverHost" type="text" placeholder="192.168.1.10"
                  class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-primary-500 outline-none" />
              </div>
              <div class="w-24 shrink-0">
                <label class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">API port</label>
                <input v-model="apiPort" type="text" inputmode="numeric" placeholder="8888"
                  class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-primary-500 outline-none" />
              </div>
            </div>

            <div class="flex gap-2 items-end">
              <div class="w-32 shrink-0">
                <label class="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">Keycloak port</label>
                <input v-model="keycloakPort" type="text" inputmode="numeric" placeholder="8080"
                  class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-3 py-2 text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-primary-500 outline-none" />
              </div>
              <label class="flex items-center gap-2 pb-2 cursor-pointer">
                <input v-model="useHttps" type="checkbox"
                  class="rounded border-gray-300 dark:border-nvr-border text-primary-600 focus:ring-primary-500" />
                <span class="text-xs text-gray-600 dark:text-gray-400">HTTPS</span>
              </label>
            </div>

            <div class="flex gap-2">
              <button type="button" @click="handleTest" :disabled="testing"
                class="flex-1 bg-gray-100 dark:bg-nvr-darker hover:bg-gray-200 dark:hover:bg-nvr-border disabled:opacity-50 text-gray-700 dark:text-gray-300 text-sm font-medium py-2 px-3 rounded-lg border border-gray-300 dark:border-nvr-border transition-colors">
                {{ testing ? 'Testing…' : 'Test connection' }}
              </button>
              <button type="button" @click="handleResetServer"
                class="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 px-3 transition-colors">
                Reset
              </button>
            </div>

            <p class="text-xs text-gray-400 leading-relaxed">
              Points this app at a different Open-NVR server — e.g. the LAN address at
              home, or a forwarded address when away. Saved on this device.
            </p>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
          <input v-model="username" type="text" required autofocus placeholder="admin"
            class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-4 py-3 text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-primary-500 focus:border-primary-500 outline-none transition-colors" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Password</label>
          <input v-model="password" type="password" required placeholder="Enter password"
            class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-4 py-3 text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-primary-500 focus:border-primary-500 outline-none transition-colors" />
        </div>
        <button type="submit" :disabled="loading"
          class="w-full bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white font-medium py-3 px-4 rounded-lg transition-colors">
          {{ loading ? 'Signing in...' : 'Sign In' }}
        </button>
      </form>

      <div class="flex items-center my-6">
        <div class="flex-1 border-t border-gray-200 dark:border-nvr-border"></div>
        <span class="px-3 text-xs text-gray-400">OR</span>
        <div class="flex-1 border-t border-gray-200 dark:border-nvr-border"></div>
      </div>

      <button @click="handleSSO"
        class="w-full bg-gray-100 dark:bg-nvr-darker hover:bg-gray-200 dark:hover:bg-nvr-border text-gray-700 dark:text-gray-300 font-medium py-3 px-4 rounded-lg border border-gray-300 dark:border-nvr-border transition-colors">
        Sign in with Keycloak SSO
      </button>

      <p class="mt-6 text-xs text-gray-400">
        Secured by Keycloak OIDC &middot; All access is logged
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'auth' })

const { loginDirect, loginSSO } = useAuth()
const server = useServerConfig()

const username = ref('')
const password = ref('')
const loading = ref(false)
const errorMsg = ref('')

const showServer = ref(false)
const serverHost = ref('')
const apiPort = ref('')
const keycloakPort = ref('')
const useHttps = ref(false)
const testing = ref(false)
const serverMsg = ref('')
const serverOk = ref(false)

// Fill the form from whatever the app is currently pointed at. Done on mount
// because the saved value lives in localStorage, which SSR cannot see.
onMounted(() => {
  const api = splitBaseUrl(server.apiUrl.value)
  const kc = splitBaseUrl(server.keycloakUrl.value)
  serverHost.value = api.host
  apiPort.value = api.port
  keycloakPort.value = kc.port
  useHttps.value = api.secure
  // Draw attention to the section when no server has been resolved yet.
  if (!api.host) showServer.value = true
})

/** Current form values as endpoint URLs. */
const endpointsFromForm = () => {
  const scheme = useHttps.value ? 'https://' : 'http://'
  return {
    apiUrl: buildBaseUrl(`${scheme}${serverHost.value}`, apiPort.value),
    keycloakUrl: buildBaseUrl(`${scheme}${serverHost.value}`, keycloakPort.value),
  }
}

const handleTest = async () => {
  testing.value = true
  serverMsg.value = ''
  try {
    const { apiUrl } = endpointsFromForm()
    const result = await server.testConnection(apiUrl)
    serverOk.value = result.ok
    serverMsg.value = result.message
  } finally {
    testing.value = false
  }
}

/** SSO redirects away from the app, so the server choice must be saved first. */
const handleSSO = () => {
  if (serverHost.value.trim()) {
    server.save(endpointsFromForm())
  }
  loginSSO()
}

const handleResetServer = () => {
  server.reset()
  const api = splitBaseUrl(server.fallback.apiUrl)
  const kc = splitBaseUrl(server.fallback.keycloakUrl)
  serverHost.value = api.host
  apiPort.value = api.port
  keycloakPort.value = kc.port
  useHttps.value = api.secure
  serverOk.value = true
  serverMsg.value = `Reset to default: ${server.fallback.apiUrl}`
}

const handleLogin = async () => {
  errorMsg.value = ''
  serverMsg.value = ''
  loading.value = true

  try {
    // Persist the chosen server before authenticating, so the login request
    // itself goes to the server the user just typed in.
    if (serverHost.value.trim()) {
      server.save(endpointsFromForm())
    }
    await loginDirect(username.value, password.value)
    navigateTo('/')
  } catch (e: any) {
    const detail = e?.data?.error_description || e?.data?.error || e?.message || ''
    // No HTTP status at all means the request never reached a server — a wrong
    // address, not a wrong password. Saying so avoids a confusing hunt.
    if (!e?.status && !e?.response?.status && !e?.data) {
      errorMsg.value = `Cannot reach ${server.keycloakUrl.value}. Check the Server settings above.`
      showServer.value = true
    } else if (detail.includes('Invalid user credentials') || detail.includes('invalid_grant')) {
      errorMsg.value = 'Invalid username or password'
    } else if (detail.includes('Account is not fully set up')) {
      errorMsg.value = 'Account requires setup. Use SSO login to complete.'
    } else if (detail.includes('not allowed for direct access')) {
      errorMsg.value = 'Direct login not enabled. Use SSO login.'
    } else {
      errorMsg.value = detail || 'Login failed. Check your credentials.'
    }
  } finally {
    loading.value = false
  }
}
</script>
