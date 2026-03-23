<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/cameras" class="text-gray-400 hover:text-white dark:hover:text-white transition-colors">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Network Discovery</h1>
    </div>

    <!-- Scan Settings -->
    <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border mb-6">
      <h2 class="text-lg font-semibold mb-4">Scan Settings</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">Subnets (comma separated)</label>
          <input v-model="subnets" type="text" placeholder="192.168.1.0/24, 10.0.0.0/24" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
        </div>
        <div class="flex gap-4 items-end">
          <div>
            <label class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">Timeout (seconds)</label>
            <input v-model.number="timeout" type="number" min="5" max="120" class="w-32 bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
          </div>
          <button @click="startScan" :disabled="scanning" class="bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white px-6 py-2 rounded-lg transition-colors">
            {{ scanning ? 'Scanning...' : 'Scan Network' }}
          </button>
          <button @click="pingAll" :disabled="pinging" class="bg-white dark:bg-nvr-card dark:bg-gray-100 dark:bg-nvr-darker hover:bg-nvr-border border border-gray-200 dark:border-nvr-border text-gray-700 dark:text-white px-4 py-2 rounded-lg transition-colors">
            {{ pinging ? 'Checking...' : 'Ping All Saved Cameras' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Quick RTSP Test -->
    <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border mb-6">
      <h2 class="text-lg font-semibold mb-4">Quick RTSP Test</h2>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">Test an RTSP URL to check if a camera is accessible</p>
      <div class="flex gap-3">
        <input v-model="testUrl" type="text" placeholder="rtsp://admin:password@192.168.1.9:554/stream1" class="flex-1 bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white text-sm" />
        <button @click="testRtsp" :disabled="testing" class="bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm transition-colors">
          {{ testing ? 'Testing...' : 'Test' }}
        </button>
      </div>
      <div v-if="testResult" class="mt-3 p-3 rounded text-sm" :class="testResult.status === 'ok' ? 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400' : 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400'">
        <strong>{{ testResult.status === 'ok' ? 'Connection OK!' : 'Connection Failed' }}</strong>
        <span v-if="testResult.error" class="ml-2">{{ testResult.error }}</span>
        <div v-if="testResult.stream_info" class="mt-1 text-xs">Codec: {{ testResult.stream_info.video_codec }}</div>
      </div>
    </div>

    <!-- Discovered Devices -->
    <div v-if="devices.length > 0" class="bg-white dark:bg-nvr-card rounded-lg border border-gray-200 dark:border-nvr-border mb-6">
      <div class="p-4 border-b border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold">Discovered Devices ({{ devices.length }})</h2>
      </div>
      <div class="divide-y divide-gray-200 dark:divide-nvr-border">
        <div v-for="(device, idx) in devices" :key="device.ip" class="p-4">
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1">
              <div class="flex items-center gap-2">
                <span class="font-medium">{{ device.ip }}</span>
                <!-- Device type badge -->
                <span :class="deviceTypeClass(device)" class="text-[10px] px-1.5 py-0.5 rounded font-medium">
                  {{ deviceType(device) }}
                </span>
                <!-- Already added badge -->
                <span v-if="isAlreadyAdded(device)" class="text-[10px] bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 px-1.5 py-0.5 rounded">
                  Added
                </span>
              </div>
              <div class="text-sm text-gray-500 dark:text-gray-400">
                <span v-if="device.brand" class="text-primary-500 dark:text-primary-400 mr-2">{{ device.brand }}</span>
                <span v-if="device.mac">MAC: {{ device.mac }}</span>
                <span v-if="device.protocols.length" class="ml-2">{{ device.protocols.join(', ') }}</span>
              </div>
              <div v-if="device.rtsp_url" class="text-xs text-gray-400 dark:text-gray-500 mt-1">RTSP: {{ device.rtsp_url }}</div>
            </div>

            <!-- Quick add or customize -->
            <div class="flex items-center gap-2 flex-shrink-0">
              <div v-if="!device.showEdit && !isAlreadyAdded(device)">
                <button @click="quickAdd(device)" class="bg-primary-600 hover:bg-primary-700 text-white text-sm px-3 py-1.5 rounded transition-colors">
                  Quick Add
                </button>
                <button @click="device.showEdit = true" class="text-sm text-primary-400 hover:underline ml-2">
                  Customize
                </button>
              </div>
              <span v-else-if="isAlreadyAdded(device)" class="text-xs text-gray-400">Already in system</span>
            </div>
          </div>

          <!-- Inline edit form -->
          <div v-if="device.showEdit" class="mt-3 p-3 bg-gray-50 dark:bg-nvr-darker rounded-lg">
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="text-xs text-gray-500 dark:text-gray-400">Name</label>
                <input v-model="device.editName" type="text" class="w-full bg-white dark:bg-nvr-card border border-gray-200 dark:border-nvr-border rounded px-2 py-1.5 text-sm text-gray-900 dark:text-white" />
              </div>
              <div>
                <label class="text-xs text-gray-500 dark:text-gray-400">Brand</label>
                <input v-model="device.editBrand" type="text" class="w-full bg-white dark:bg-nvr-card border border-gray-200 dark:border-nvr-border rounded px-2 py-1.5 text-sm text-gray-900 dark:text-white" />
              </div>
              <div>
                <label class="text-xs text-gray-500 dark:text-gray-400">Stream URL</label>
                <input v-model="device.editUrl" type="text" class="w-full bg-white dark:bg-nvr-card border border-gray-200 dark:border-nvr-border rounded px-2 py-1.5 text-sm text-gray-900 dark:text-white" />
              </div>
              <div>
                <label class="text-xs text-gray-500 dark:text-gray-400">Protocol</label>
                <select v-model="device.editProtocol" class="w-full bg-white dark:bg-nvr-card border border-gray-200 dark:border-nvr-border rounded px-2 py-1.5 text-sm text-gray-900 dark:text-white">
                  <option value="rtsp">RTSP</option>
                  <option value="onvif">ONVIF</option>
                  <option value="mjpeg">MJPEG</option>
                  <option value="rtmp">RTMP</option>
                </select>
              </div>
              <div>
                <label class="text-xs text-gray-500 dark:text-gray-400">Username</label>
                <input v-model="device.editUsername" type="text" class="w-full bg-white dark:bg-nvr-card border border-gray-200 dark:border-nvr-border rounded px-2 py-1.5 text-sm text-gray-900 dark:text-white" />
              </div>
              <div>
                <label class="text-xs text-gray-500 dark:text-gray-400">Password</label>
                <input v-model="device.editPassword" type="password" class="w-full bg-white dark:bg-nvr-card border border-gray-200 dark:border-nvr-border rounded px-2 py-1.5 text-sm text-gray-900 dark:text-white" />
              </div>
            </div>
            <div class="flex gap-2 mt-3">
              <button @click="saveCustom(device)" class="bg-primary-600 hover:bg-primary-700 text-white text-sm px-4 py-1.5 rounded transition-colors">Save Camera</button>
              <button @click="device.showEdit = false" class="text-sm text-gray-400 hover:text-gray-600 dark:hover:text-white">Cancel</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Saved Cameras Liveness -->
    <div v-if="pingResults.length > 0" class="bg-white dark:bg-nvr-card rounded-lg border border-gray-200 dark:border-nvr-border">
      <div class="p-4 border-b border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold">Camera Liveness Check</h2>
      </div>
      <div class="divide-y divide-gray-200 dark:divide-nvr-border">
        <div v-for="result in pingResults" :key="result.camera_id" class="p-4 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span :class="result.alive ? 'bg-green-500' : 'bg-red-500'" class="w-3 h-3 rounded-full"></span>
            <span class="font-medium">{{ result.name }}</span>
          </div>
          <div class="flex items-center gap-3">
            <span :class="result.alive ? 'text-green-400' : 'text-red-400'" class="text-sm">
              {{ result.alive ? 'Online' : 'Offline' }}
            </span>
            <button @click="pingSingle(result.camera_id)" class="text-xs text-primary-400 hover:underline">Rescan</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const { success, error: showError } = useToast()
const store = useDiscoveryStore()
const timeout = ref(30)
const pinging = ref(false)
const testUrl = ref('')
const testing = ref(false)
const testResult = ref<any>(null)
const existingCameras = ref<any[]>([])

// Load existing cameras to mark duplicates
onMounted(async () => {
  try {
    existingCameras.value = await useApi<any[]>('/api/cameras')
  } catch {}
})

// Check if device IP is already added as a camera
const isAlreadyAdded = (device: any) => {
  const ip = String(device.ip)
  return existingCameras.value.some(c => c.stream_url?.includes(ip))
}

// Detect device type from protocols/ports/brand
const deviceType = (device: any) => {
  const p = device.protocols || []
  const brand = (device.brand || '').toLowerCase()
  if (p.includes('rtsp') || brand.includes('v360') || brand.includes('hikvision') || brand.includes('dahua') || brand.includes('reolink')) return 'Camera'
  if (p.includes('onvif')) return 'Camera'
  if (brand.includes('tp-link') || brand.includes('router')) return 'Router'
  if (p.length === 0 && !device.brand) return 'Device'
  if (p.includes('http') && !p.includes('rtsp')) return 'Web Device'
  return 'Device'
}

const deviceTypeClass = (device: any) => {
  const type = deviceType(device)
  switch (type) {
    case 'Camera': return 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400'
    case 'Router': return 'bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-400'
    case 'Web Device': return 'bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400'
    default: return 'bg-gray-100 dark:bg-gray-700/30 text-gray-600 dark:text-gray-400'
  }
}

// Use store state (persists across navigation)
const subnets = computed({
  get: () => store.subnets || '192.168.1.0/24',
  set: (v: string) => store.setSubnets(v),
})
const scanning = computed(() => store.scanning)
const devices = computed(() => store.devices)
const pingResults = computed(() => store.pingResults)

const testRtsp = async () => {
  if (!testUrl.value) return
  testing.value = true
  testResult.value = null
  try {
    testResult.value = await useApi<any>('/api/cameras/test-url', {
      method: 'POST',
      body: { protocol_type: 'rtsp', stream_url: testUrl.value },
    })
  } catch (e: any) {
    testResult.value = { status: 'error', error: e?.data?.error?.message || 'Connection failed' }
  }
  testing.value = false
}

onMounted(async () => {
  // Auto-detect subnets if not already set
  if (!store.subnets) {
    try {
      const detected = await useApi<string[]>('/api/discovery/subnets')
      if (detected.length > 0) {
        store.setSubnets(detected.join(', '))
      }
    } catch {}
  }
})

const startScan = async () => {
  store.setScanning(true)
  try {
    const subnetList = subnets.value.split(',').map((s: string) => s.trim()).filter(Boolean)
    const result = await useApi<any[]>('/api/discovery/scan', {
      method: 'POST',
      body: { subnets: subnetList, timeout_secs: timeout.value },
    })
    store.setDevices(result)
    success(`Found ${result.length} devices`)
  } catch (e) {
    showError('Scan failed')
  } finally {
    store.setScanning(false)
  }
}

const quickAdd = async (device: any) => {
  try {
    await useApi('/api/cameras', {
      method: 'POST',
      body: {
        name: device.editName || `Camera ${device.ip}`,
        protocol_type: device.editProtocol || device.protocols[0] || 'rtsp',
        stream_url: device.editUrl || device.rtsp_url || `rtsp://${device.ip}:554/stream1`,
        brand: device.brand,
        onvif_url: device.onvif_url,
      },
    })
    success(`Camera "${device.editName}" added!`)
    store.removeDevice(device.ip)
  } catch (e) {
    showError('Failed to add camera')
  }
}

const saveCustom = async (device: any) => {
  try {
    await useApi('/api/cameras', {
      method: 'POST',
      body: {
        name: device.editName,
        protocol_type: device.editProtocol,
        stream_url: device.editUrl,
        brand: device.editBrand,
        username: device.editUsername || undefined,
        password: device.editPassword || undefined,
        onvif_url: device.onvif_url,
      },
    })
    success(`Camera "${device.editName}" saved!`)
    device.showEdit = false
    store.removeDevice(device.ip)
  } catch (e) {
    showError('Failed to save camera')
  }
}

const pingAll = async () => {
  pinging.value = true
  try {
    const results = await useApi<any[]>('/api/cameras/ping-all', { method: 'POST' })
    store.setPingResults(results)
    const online = results.filter((r: any) => r.alive).length
    success(`${online}/${results.length} cameras online`)
  } catch (e) {
    showError('Ping failed')
  } finally {
    pinging.value = false
  }
}

const pingSingle = async (cameraId: string) => {
  try {
    const result = await useApi<any>(`/api/cameras/${cameraId}/ping`, { method: 'POST' })
    const results = [...store.pingResults]
    const idx = results.findIndex((r: any) => r.camera_id === cameraId)
    if (idx >= 0) results[idx] = result
    store.setPingResults(results)
    if (result.alive) {
      success(`${result.name} is online`)
    } else {
      showError(`${result.name} is offline`)
    }
  } catch {}
}
</script>
