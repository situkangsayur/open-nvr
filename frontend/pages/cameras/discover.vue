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

    <!-- Discovered Devices -->
    <div v-if="devices.length > 0" class="bg-white dark:bg-nvr-card rounded-lg border border-gray-200 dark:border-nvr-border mb-6">
      <div class="p-4 border-b border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold">Discovered Devices ({{ devices.length }})</h2>
      </div>
      <div class="divide-y divide-gray-200 dark:divide-nvr-border">
        <div v-for="(device, idx) in devices" :key="device.ip" class="p-4">
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1">
              <div class="font-medium">{{ device.ip }}</div>
              <div class="text-sm text-gray-500 dark:text-gray-400">
                <span v-if="device.brand" class="text-primary-500 dark:text-primary-400 mr-2">{{ device.brand }}</span>
                <span v-if="device.mac">MAC: {{ device.mac }}</span>
                <span class="ml-2">Protocols: {{ device.protocols.join(', ') }}</span>
              </div>
              <div v-if="device.rtsp_url" class="text-xs text-gray-400 dark:text-gray-500 mt-1">RTSP: {{ device.rtsp_url }}</div>
            </div>

            <!-- Quick add or customize -->
            <div class="flex items-center gap-2 flex-shrink-0">
              <div v-if="!device.showEdit">
                <button @click="quickAdd(device)" class="bg-primary-600 hover:bg-primary-700 text-white text-sm px-3 py-1.5 rounded transition-colors">
                  Quick Add
                </button>
                <button @click="device.showEdit = true" class="text-sm text-primary-400 hover:underline ml-2">
                  Customize
                </button>
              </div>
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
const subnets = ref('192.168.1.0/24')
const timeout = ref(30)
const scanning = ref(false)
const pinging = ref(false)
const devices = ref<any[]>([])
const pingResults = ref<any[]>([])

const startScan = async () => {
  scanning.value = true
  devices.value = []
  try {
    const subnetList = subnets.value.split(',').map(s => s.trim()).filter(Boolean)
    const result = await useApi<any[]>('/api/discovery/scan', {
      method: 'POST',
      body: { subnets: subnetList, timeout_secs: timeout.value },
    })
    // Enrich each device with edit fields
    devices.value = result.map(d => ({
      ...d,
      showEdit: false,
      editName: d.name || `Camera ${d.ip}`,
      editBrand: d.brand || '',
      editUrl: d.rtsp_url || `rtsp://${d.ip}:554/stream1`,
      editProtocol: d.protocols[0] || 'rtsp',
      editUsername: '',
      editPassword: '',
    }))
    success(`Found ${result.length} devices`)
  } catch (e) {
    showError('Scan failed')
  } finally {
    scanning.value = false
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
    // Remove from discovered list
    devices.value = devices.value.filter(d => d.ip !== device.ip)
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
    devices.value = devices.value.filter(d => d.ip !== device.ip)
  } catch (e) {
    showError('Failed to save camera')
  }
}

const pingAll = async () => {
  pinging.value = true
  try {
    pingResults.value = await useApi<any[]>('/api/cameras/ping-all', { method: 'POST' })
    const online = pingResults.value.filter(r => r.alive).length
    success(`${online}/${pingResults.value.length} cameras online`)
  } catch (e) {
    showError('Ping failed')
  } finally {
    pinging.value = false
  }
}

const pingSingle = async (cameraId: string) => {
  try {
    const result = await useApi<any>(`/api/cameras/${cameraId}/ping`, { method: 'POST' })
    const idx = pingResults.value.findIndex(r => r.camera_id === cameraId)
    if (idx >= 0) {
      pingResults.value[idx] = result
    }
    if (result.alive) {
      success(`${result.name} is online`)
    } else {
      showError(`${result.name} is offline`)
    }
  } catch {}
}
</script>
