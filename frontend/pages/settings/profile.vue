<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/settings" class="text-gray-400 hover:text-gray-700 dark:hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Profile & Password</h1>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- User Info -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">User Information</h2>
        <div class="space-y-3 text-sm">
          <div class="flex justify-between">
            <span class="text-gray-500 dark:text-gray-400">Username</span>
            <span class="font-medium">{{ userInfo.username || '-' }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-500 dark:text-gray-400">Email</span>
            <span class="font-medium">{{ userInfo.email || '-' }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-500 dark:text-gray-400">Name</span>
            <span class="font-medium">{{ userInfo.name || '-' }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-500 dark:text-gray-400">Roles</span>
            <div class="flex gap-1">
              <span v-for="role in userInfo.roles" :key="role" class="text-xs bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-400 px-2 py-0.5 rounded">
                {{ role }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Change Password -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Change Password</h2>

        <div v-if="successMsg" class="mb-4 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg text-sm text-green-600 dark:text-green-400">
          {{ successMsg }}
        </div>
        <div v-if="errorMsg" class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-sm text-red-600 dark:text-red-400">
          {{ errorMsg }}
        </div>

        <form @submit.prevent="changePassword" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Current Password</label>
            <input
              v-model="form.currentPassword"
              type="password"
              required
              class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-3 py-2.5 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 outline-none"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">New Password</label>
            <input
              v-model="form.newPassword"
              type="password"
              required
              minlength="8"
              class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-3 py-2.5 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 outline-none"
            />
            <p class="text-xs text-gray-400 mt-1">Minimum 8 characters</p>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Confirm New Password</label>
            <input
              v-model="form.confirmPassword"
              type="password"
              required
              class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-300 dark:border-nvr-border rounded-lg px-3 py-2.5 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 outline-none"
            />
          </div>
          <button
            type="submit"
            :disabled="loading"
            class="w-full bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white font-medium py-2.5 rounded-lg transition-colors"
          >
            {{ loading ? 'Updating...' : 'Update Password' }}
          </button>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const { token, user } = useAuth()

const form = ref({
  currentPassword: '',
  newPassword: '',
  confirmPassword: '',
})
const loading = ref(false)
const errorMsg = ref('')
const successMsg = ref('')

const userInfo = computed(() => {
  const u = user.value || {}
  return {
    username: u.preferred_username || u.sub || '',
    email: u.email || '',
    name: u.name || `${u.given_name || ''} ${u.family_name || ''}`.trim() || '',
    roles: u.realm_access?.roles?.filter((r: string) => !r.startsWith('default-')) || [],
  }
})

const changePassword = async () => {
  errorMsg.value = ''
  successMsg.value = ''

  if (form.value.newPassword !== form.value.confirmPassword) {
    errorMsg.value = 'New passwords do not match'
    return
  }
  if (form.value.newPassword.length < 8) {
    errorMsg.value = 'Password must be at least 8 characters'
    return
  }
  if (form.value.newPassword === form.value.currentPassword) {
    errorMsg.value = 'New password must be different'
    return
  }

  loading.value = true
  try {
    await useApi('/api/user/change-password', {
      method: 'POST',
      body: {
        current_password: form.value.currentPassword,
        new_password: form.value.newPassword,
      },
    })
    successMsg.value = 'Password changed successfully!'
    form.value = { currentPassword: '', newPassword: '', confirmPassword: '' }
  } catch (e: any) {
    const msg = e?.data?.error?.message || e?.message || ''
    if (msg.includes('incorrect') || msg.includes('Authentication')) {
      errorMsg.value = 'Current password is incorrect'
    } else {
      errorMsg.value = msg || 'Failed to change password'
    }
  } finally {
    loading.value = false
  }
}
</script>
