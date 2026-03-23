<template>
  <div class="text-center">
    <div class="bg-white dark:bg-nvr-card rounded-xl p-8 border border-gray-200 dark:border-nvr-border max-w-md mx-auto shadow-xl">
      <!-- Logo -->
      <div class="mb-6">
        <div class="w-16 h-16 bg-primary-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
          <span class="text-2xl font-bold text-white">NVR</span>
        </div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Open-NVR</h1>
        <p class="text-gray-500 dark:text-gray-400 mt-1">Network Video Recorder</p>
      </div>

      <!-- Error message -->
      <div v-if="errorMsg" class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-sm text-red-600 dark:text-red-400">
        {{ errorMsg }}
      </div>

      <!-- Login Form -->
      <form @submit.prevent="handleLogin" class="space-y-4 text-left">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
          <input
            v-model="username"
            type="text"
            required
            autofocus
            placeholder="admin"
            class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-4 py-3 text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-primary-500 focus:border-primary-500 outline-none transition-colors"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Password</label>
          <input
            v-model="password"
            type="password"
            required
            placeholder="Enter password"
            class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-4 py-3 text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-primary-500 focus:border-primary-500 outline-none transition-colors"
          />
        </div>

        <button
          type="submit"
          :disabled="loading"
          class="w-full bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white font-medium py-3 px-4 rounded-lg transition-colors"
        >
          {{ loading ? 'Signing in...' : 'Sign In' }}
        </button>
      </form>

      <!-- Divider -->
      <div class="flex items-center my-6">
        <div class="flex-1 border-t border-gray-200 dark:border-nvr-border"></div>
        <span class="px-3 text-xs text-gray-400">OR</span>
        <div class="flex-1 border-t border-gray-200 dark:border-nvr-border"></div>
      </div>

      <!-- SSO Button -->
      <button
        @click="loginSSO"
        class="w-full bg-gray-100 dark:bg-nvr-darker hover:bg-gray-200 dark:hover:bg-nvr-border text-gray-700 dark:text-gray-300 font-medium py-3 px-4 rounded-lg border border-gray-300 dark:border-nvr-border transition-colors"
      >
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

const config = useRuntimeConfig()
const { login: loginSSO } = useAuth()

const username = ref('')
const password = ref('')
const loading = ref(false)
const errorMsg = ref('')

const handleLogin = async () => {
  errorMsg.value = ''
  loading.value = true

  try {
    // Use Keycloak Resource Owner Password Credentials grant
    const tokenUrl = `${config.public.keycloakUrl}/realms/${config.public.keycloakRealm}/protocol/openid-connect/token`

    const body = new URLSearchParams({
      grant_type: 'password',
      client_id: config.public.keycloakClientId,
      username: username.value,
      password: password.value,
      scope: 'openid profile email',
    })

    const response = await $fetch<any>(tokenUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: body.toString(),
    })

    if (response.access_token) {
      // Decode user info from token
      const payload = JSON.parse(atob(response.access_token.split('.')[1]))

      // Store in auth state
      const auth = useAuth()
      const authStore = useAuthStore()
      authStore.setAuth(response.access_token, payload)

      // Also store in localStorage for persistence
      if (import.meta.client) {
        localStorage.setItem('opennvr-token', response.access_token)
        localStorage.setItem('opennvr-refresh-token', response.refresh_token || '')
        localStorage.setItem('opennvr-user', JSON.stringify(payload))
      }

      navigateTo('/')
    }
  } catch (e: any) {
    const detail = e?.data?.error_description || e?.data?.error || ''
    if (detail.includes('Invalid user credentials') || detail.includes('invalid_grant')) {
      errorMsg.value = 'Invalid username or password'
    } else if (detail.includes('Account is not fully set up')) {
      errorMsg.value = 'Account requires setup. Use SSO login to complete.'
    } else {
      errorMsg.value = detail || 'Login failed. Check your credentials.'
    }
  } finally {
    loading.value = false
  }
}
</script>
