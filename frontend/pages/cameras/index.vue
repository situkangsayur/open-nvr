<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">Cameras</h1>
      <div class="flex gap-3">
        <NuxtLink to="/cameras/discover" class="bg-white dark:bg-nvr-card hover:bg-nvr-border text-white px-4 py-2 rounded-lg border border-gray-200 dark:border-nvr-border transition-colors">
          Discover
        </NuxtLink>
        <button @click="showAddModal = true" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg transition-colors">
          Add Camera
        </button>
        <button @click="pingAll" :disabled="pinging" class="bg-gray-100 dark:bg-nvr-darker hover:bg-gray-200 dark:hover:bg-nvr-border text-gray-700 dark:text-gray-300 px-4 py-2 rounded-lg border border-gray-200 dark:border-nvr-border transition-colors text-sm">
          {{ pinging ? 'Checking...' : 'Ping All' }}
        </button>
      </div>
    </div>

    <!-- Quick Add by IP -->
    <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border mb-4">
      <div class="flex gap-3 items-end">
        <div class="flex-1">
          <label class="block text-xs text-gray-500 dark:text-gray-400 mb-1">Quick Add Camera by IP</label>
          <input v-model="quickIp" type="text" placeholder="192.168.1.103" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white text-sm" />
        </div>
        <div>
          <input v-model="quickName" type="text" placeholder="Camera name" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white text-sm" />
        </div>
        <button @click="quickAddByIp" :disabled="!quickIp" class="bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white px-4 py-2 rounded text-sm transition-colors whitespace-nowrap">
          Add
        </button>
      </div>
    </div>

    <!-- Network Warning -->
    <div v-if="cameras.length > 0 && allOffline" class="mb-4 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
      <div class="flex items-start gap-2">
        <span class="text-yellow-500 text-sm mt-0.5">&#9888;</span>
        <div>
          <p class="text-xs font-medium text-yellow-700 dark:text-yellow-400">All cameras are offline</p>
          <p class="text-xs text-yellow-600 dark:text-yellow-500 mt-0.5">
            Common cause: WiFi AP Isolation prevents the server from reaching cameras.
            Try connecting the server via ethernet or disabling AP Isolation in your router settings.
          </p>
        </div>
      </div>
    </div>

    <div v-if="loading" class="text-gray-400">Loading cameras...</div>
    <div v-else-if="cameras.length === 0" class="text-gray-400 text-center py-12">
      No cameras configured. Add one or use discovery.
    </div>
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <CameraCard v-for="camera in cameras" :key="camera.id" :camera="camera" @delete="deleteCamera" @edit="openEdit" />
    </div>

    <!-- Add Camera Modal -->
    <div v-if="showAddModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border w-full max-w-lg">
        <h2 class="text-xl font-bold mb-4">Add Camera</h2>
        <form @submit.prevent="addCamera">
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name</label>
              <input v-model="newCamera.name" type="text" required class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Protocol</label>
              <select v-model="newCamera.protocol_type" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white">
                <option value="rtsp">RTSP</option>
                <option value="onvif">ONVIF</option>
                <option value="mjpeg">MJPEG</option>
                <option value="rtmp">RTMP</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Stream URL</label>
              <input v-model="newCamera.stream_url" type="text" required placeholder="rtsp://..." class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
                <input v-model="newCamera.username" type="text" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Password</label>
                <input v-model="newCamera.password" type="password" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
              </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Brand</label>
                <input v-model="newCamera.brand" type="text" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Connection</label>
                <select v-model="newCamera.connection_type" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white">
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

    <!-- Edit Camera Modal -->
    <CameraEditModal
      :open="!!editingId"
      :camera-id="editingId"
      @close="editingId = null"
      @saved="loadCameras"
    />
  </div>
</template>

<script setup lang="ts">
const cameraStore = useCameraStore()
const { success, error: showError } = useToast()
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

const editingId = ref<string | null>(null)
const openEdit = (id: string) => { editingId.value = id }

const quickIp = ref('')
const quickName = ref('')
const pinging = ref(false)
const cameras = computed(() => cameraStore.cameras)
const loading = computed(() => cameraStore.loading)
const allOffline = computed(() => cameras.value.length > 0 && cameras.value.every((c: any) => c.status !== 'online'))

const loadCameras = () => cameraStore.fetchAll()

const quickAddByIp = async () => {
  if (!quickIp.value) return
  const name = quickName.value || `Camera ${quickIp.value}`
  try {
    await useApi('/api/cameras', {
      method: 'POST',
      body: {
        name,
        protocol_type: 'rtsp',
        stream_url: `rtsp://${quickIp.value}:554/stream1`,
        connection_type: 'ethernet',
      },
    })
    quickIp.value = ''
    quickName.value = ''
    await loadCameras()
    success(`Camera "${name}" added`)
  } catch {
    showError('Failed to add camera')
  }
}

const pingAll = async () => {
  pinging.value = true
  try {
    await useApi('/api/cameras/ping-all', { method: 'POST' })
    await loadCameras()
    success('Camera status updated')
  } catch { }
  pinging.value = false
}

const addCamera = async () => {
  try {
    await cameraStore.create(newCamera.value)
    showAddModal.value = false
    newCamera.value = { name: '', protocol_type: 'rtsp', stream_url: '', username: '', password: '', brand: '', connection_type: 'ethernet' }
    success('Camera added successfully')
  } catch (e) {
    showError('Failed to add camera')
  }
}

const deleteCamera = async (id: string) => {
  if (!confirm('Delete this camera?')) return
  try {
    await cameraStore.remove(id)
    success('Camera deleted')
  } catch (e) {
    showError('Failed to delete camera')
  }
}

// Auto-refresh every 30 seconds
let refreshInterval: ReturnType<typeof setInterval>
onMounted(() => {
  loadCameras()
  refreshInterval = setInterval(loadCameras, 30000)
})
onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval)
})
</script>
