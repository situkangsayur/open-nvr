<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/settings" class="text-gray-400 hover:text-white dark:hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Access Control</h1>
    </div>

    <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border mb-6">
      <h2 class="text-lg font-semibold mb-4">Grant Camera Access</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">User ID</label>
          <input v-model="grantForm.user_id" type="text" placeholder="Keycloak user ID or username" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white text-sm" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">Camera</label>
          <select v-model="grantForm.camera_id" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white text-sm">
            <option value="">Select camera</option>
            <option v-for="cam in cameras" :key="cam.id" :value="cam.id">{{ cam.name }}</option>
          </select>
        </div>
        <div class="flex items-end">
          <button @click="grantAccess" :disabled="!grantForm.user_id || !grantForm.camera_id" class="bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white px-4 py-2 rounded text-sm transition-colors">
            Grant Access
          </button>
        </div>
      </div>
      <div class="flex gap-4 mt-3">
        <label class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
          <input type="checkbox" v-model="grantForm.can_view" class="rounded" /> View
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
          <input type="checkbox" v-model="grantForm.can_ptz" class="rounded" /> PTZ
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
          <input type="checkbox" v-model="grantForm.can_playback" class="rounded" /> Playback
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
          <input type="checkbox" v-model="grantForm.can_export" class="rounded" /> Export
        </label>
      </div>
    </div>

    <!-- Per-camera permissions table -->
    <div v-for="cam in cameras" :key="cam.id" class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border mb-4">
      <div class="flex justify-between items-center mb-3">
        <h3 class="font-semibold">{{ cam.name }}</h3>
        <span class="text-xs text-gray-400">{{ cam.id.slice(0, 8) }}</span>
      </div>
      <div v-if="permissionsByCamera[cam.id]?.length > 0" class="space-y-2">
        <div v-for="perm in permissionsByCamera[cam.id]" :key="perm.id" class="flex items-center justify-between p-2 bg-gray-50 dark:bg-nvr-darker rounded text-sm">
          <div>
            <span class="font-medium">{{ perm.user_id }}</span>
            <div class="flex gap-2 mt-1">
              <span v-if="perm.can_view" class="text-xs bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 px-1.5 py-0.5 rounded">View</span>
              <span v-if="perm.can_ptz" class="text-xs bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 px-1.5 py-0.5 rounded">PTZ</span>
              <span v-if="perm.can_playback" class="text-xs bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-400 px-1.5 py-0.5 rounded">Playback</span>
              <span v-if="perm.can_export" class="text-xs bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400 px-1.5 py-0.5 rounded">Export</span>
            </div>
          </div>
          <button @click="revokeAccess(perm.user_id, cam.id)" class="text-xs text-red-400 hover:underline">Revoke</button>
        </div>
      </div>
      <div v-else class="text-xs text-gray-400">No user-specific permissions (admin/operator have full access by role)</div>
    </div>

    <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4 border border-blue-200 dark:border-blue-800 text-sm text-blue-600 dark:text-blue-400">
      <strong>Roles:</strong>
      <ul class="mt-1 list-disc list-inside text-xs">
        <li><strong>Admin</strong> -- Full access to everything</li>
        <li><strong>Operator</strong> -- View all cameras, PTZ, playback, export. No system settings.</li>
        <li><strong>Viewer</strong> -- Only cameras explicitly granted below. No PTZ/export by default.</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
const { success, error: showError } = useToast()
const cameras = ref<any[]>([])
const permissionsByCamera = ref<Record<string, any[]>>({})
const grantForm = ref({
  user_id: '',
  camera_id: '',
  can_view: true,
  can_ptz: false,
  can_playback: true,
  can_export: false,
})

const loadData = async () => {
  try {
    cameras.value = await useApi<any[]>('/api/cameras')
    for (const cam of cameras.value) {
      try {
        permissionsByCamera.value[cam.id] = await useApi<any[]>(`/api/permissions/cameras/${cam.id}`)
      } catch {
        permissionsByCamera.value[cam.id] = []
      }
    }
  } catch {}
}

const grantAccess = async () => {
  try {
    await useApi('/api/permissions/grant', { method: 'POST', body: grantForm.value })
    success('Access granted')
    grantForm.value.user_id = ''
    grantForm.value.camera_id = ''
    await loadData()
  } catch {
    showError('Failed to grant access')
  }
}

const revokeAccess = async (userId: string, cameraId: string) => {
  if (!confirm(`Revoke access for ${userId}?`)) return
  try {
    await useApi(`/api/permissions/revoke/${userId}/${cameraId}`, { method: 'DELETE' })
    success('Access revoked')
    await loadData()
  } catch {
    showError('Failed to revoke access')
  }
}

onMounted(loadData)
</script>
