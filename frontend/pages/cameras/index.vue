<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">Cameras</h1>
      <div class="flex gap-3">
        <NuxtLink to="/cameras/discover" class="bg-nvr-card hover:bg-nvr-border text-white px-4 py-2 rounded-lg border border-nvr-border transition-colors">
          Discover
        </NuxtLink>
        <button @click="showAddModal = true" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg transition-colors">
          Add Camera
        </button>
      </div>
    </div>

    <div v-if="loading" class="text-gray-400">Loading cameras...</div>
    <div v-else-if="cameras.length === 0" class="text-gray-400 text-center py-12">
      No cameras configured. Add one or use discovery.
    </div>
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <CameraCard v-for="camera in cameras" :key="camera.id" :camera="camera" @delete="deleteCamera" />
    </div>

    <!-- Add Camera Modal -->
    <div v-if="showAddModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-nvr-card rounded-lg p-6 border border-nvr-border w-full max-w-lg">
        <h2 class="text-xl font-bold mb-4">Add Camera</h2>
        <form @submit.prevent="addCamera">
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Name</label>
              <input v-model="newCamera.name" type="text" required class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Protocol</label>
              <select v-model="newCamera.protocol_type" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white">
                <option value="rtsp">RTSP</option>
                <option value="onvif">ONVIF</option>
                <option value="mjpeg">MJPEG</option>
                <option value="rtmp">RTMP</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Stream URL</label>
              <input v-model="newCamera.stream_url" type="text" required placeholder="rtsp://..." class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-300 mb-1">Username</label>
                <input v-model="newCamera.username" type="text" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-300 mb-1">Password</label>
                <input v-model="newCamera.password" type="password" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
              </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-300 mb-1">Brand</label>
                <input v-model="newCamera.brand" type="text" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-300 mb-1">Connection</label>
                <select v-model="newCamera.connection_type" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white">
                  <option value="ethernet">Ethernet</option>
                  <option value="wifi">WiFi</option>
                </select>
              </div>
            </div>
          </div>
          <div class="flex justify-end gap-3 mt-6">
            <button type="button" @click="showAddModal = false" class="px-4 py-2 text-gray-400 hover:text-white transition-colors">Cancel</button>
            <button type="submit" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg transition-colors">Add</button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const cameras = ref<any[]>([])
const loading = ref(true)
const showAddModal = ref(false)
const newCamera = ref({
  name: '',
  protocol_type: 'rtsp',
  stream_url: '',
  username: '',
  password: '',
  brand: '',
  connection_type: 'ethernet',
})

const loadCameras = async () => {
  loading.value = true
  try {
    cameras.value = await useApi<any[]>('/api/cameras')
  } catch (e) {
    console.error('Failed to load cameras', e)
  } finally {
    loading.value = false
  }
}

const addCamera = async () => {
  try {
    await useApi('/api/cameras', { method: 'POST', body: newCamera.value })
    showAddModal.value = false
    newCamera.value = { name: '', protocol_type: 'rtsp', stream_url: '', username: '', password: '', brand: '', connection_type: 'ethernet' }
    await loadCameras()
  } catch (e) {
    console.error('Failed to add camera', e)
  }
}

const deleteCamera = async (id: string) => {
  if (!confirm('Delete this camera?')) return
  try {
    await useApi(`/api/cameras/${id}`, { method: 'DELETE' })
    await loadCameras()
  } catch (e) {
    console.error('Failed to delete camera', e)
  }
}

onMounted(loadCameras)
</script>
