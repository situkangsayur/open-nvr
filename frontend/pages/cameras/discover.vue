<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/cameras" class="text-gray-400 hover:text-white transition-colors">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Network Discovery</h1>
    </div>

    <div class="bg-nvr-card rounded-lg p-6 border border-nvr-border mb-6">
      <h2 class="text-lg font-semibold mb-4">Scan Settings</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-1">Subnets (comma separated)</label>
          <input v-model="subnets" type="text" placeholder="192.168.1.0/24, 10.0.0.0/24" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-1">Timeout (seconds)</label>
          <input v-model.number="timeout" type="number" min="5" max="120" class="w-32 bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
        </div>
        <button @click="startScan" :disabled="scanning" class="bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white px-6 py-2 rounded-lg transition-colors">
          {{ scanning ? 'Scanning...' : 'Start Scan' }}
        </button>
      </div>
    </div>

    <div v-if="devices.length > 0" class="bg-nvr-card rounded-lg border border-nvr-border">
      <div class="p-4 border-b border-nvr-border">
        <h2 class="text-lg font-semibold">Discovered Devices ({{ devices.length }})</h2>
      </div>
      <div class="divide-y divide-nvr-border">
        <div v-for="device in devices" :key="device.ip" class="p-4 flex items-center justify-between hover:bg-nvr-darker/50">
          <div>
            <div class="font-medium">{{ device.name || device.ip }}</div>
            <div class="text-sm text-gray-400">
              {{ device.ip }} <span v-if="device.mac" class="ml-2">{{ device.mac }}</span>
              <span v-if="device.brand" class="ml-2 text-primary-400">{{ device.brand }}</span>
            </div>
            <div class="text-xs text-gray-500 mt-1">
              Protocols: {{ device.protocols.join(', ') }}
            </div>
          </div>
          <button @click="addDevice(device)" class="bg-primary-600 hover:bg-primary-700 text-white text-sm px-3 py-1.5 rounded transition-colors">
            Add
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const subnets = ref('192.168.1.0/24')
const timeout = ref(30)
const scanning = ref(false)
const devices = ref<any[]>([])

const startScan = async () => {
  scanning.value = true
  devices.value = []
  try {
    const subnetList = subnets.value.split(',').map(s => s.trim()).filter(Boolean)
    const result = await useApi<any[]>('/api/discovery/scan', {
      method: 'POST',
      body: { subnets: subnetList, timeout_secs: timeout.value },
    })
    devices.value = result
  } catch (e) {
    console.error('Scan failed', e)
  } finally {
    scanning.value = false
  }
}

const addDevice = async (device: any) => {
  try {
    await useApi('/api/cameras', {
      method: 'POST',
      body: {
        name: device.name || `Camera ${device.ip}`,
        protocol_type: device.protocols[0] || 'rtsp',
        stream_url: device.rtsp_url || `rtsp://${device.ip}:554/stream`,
        brand: device.brand,
        onvif_url: device.onvif_url,
      },
    })
    alert('Camera added!')
  } catch (e) {
    console.error('Failed to add device', e)
  }
}
</script>
