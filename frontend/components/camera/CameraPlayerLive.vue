<template>
  <div class="w-full h-full flex items-center justify-center bg-black relative">
    <!-- Stream attempt via WebSocket -->
    <video ref="videoEl" autoplay muted playsinline class="w-full h-full object-contain" v-show="connected" />

    <!-- No stream fallback -->
    <div v-if="!connected" class="absolute inset-0 flex flex-col items-center justify-center text-center p-4">
      <div class="text-gray-500 text-sm mb-2">{{ statusMessage }}</div>
      <div class="text-gray-600 text-xs">{{ camera.stream_url }}</div>
      <div class="mt-3 flex gap-2">
        <button @click="trySnapshot" class="text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 px-3 py-1 rounded transition-colors">
          Snapshot
        </button>
        <button @click="retryConnect" class="text-xs bg-primary-600 hover:bg-primary-700 text-white px-3 py-1 rounded transition-colors">
          Retry
        </button>
      </div>
      <!-- Show snapshot if available -->
      <img v-if="snapshotUrl" :src="snapshotUrl" class="mt-3 max-w-full max-h-32 rounded" />
    </div>

    <!-- Live badge -->
    <div v-if="connected" class="absolute bottom-2 right-2">
      <span class="text-xs text-green-400 bg-black/60 px-2 py-0.5 rounded">LIVE</span>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ camera: any }>()
const videoEl = ref<HTMLVideoElement | null>(null)
const connected = ref(false)
const statusMessage = ref('No stream available')
const snapshotUrl = ref('')
const config = useRuntimeConfig()

let ws: WebSocket | null = null

onMounted(() => {
  connectStream()
})

onUnmounted(() => {
  cleanup()
})

function cleanup() {
  if (ws) { ws.close(); ws = null }
  connected.value = false
}

function connectStream() {
  if (!import.meta.client) return

  statusMessage.value = 'Connecting...'
  const wsUrl = config.public.apiUrl.replace(/^http/, 'ws')

  try {
    ws = new WebSocket(`${wsUrl}/ws/stream/${props.camera.id}`)
    ws.binaryType = 'arraybuffer'

    ws.onopen = () => {
      connected.value = true
      statusMessage.value = ''
    }

    ws.onclose = () => {
      connected.value = false
      if (props.camera.status === 'online') {
        statusMessage.value = 'Stream disconnected. Retrying...'
        setTimeout(connectStream, 5000)
      } else {
        statusMessage.value = 'Camera offline — enable RTSP in camera settings'
      }
    }

    ws.onerror = () => {
      connected.value = false
      statusMessage.value = 'Camera offline — no RTSP stream detected'
    }

    // Timeout: if no data in 5 seconds, consider it failed
    setTimeout(() => {
      if (!connected.value) {
        statusMessage.value = 'No stream — camera may need RTSP enabled'
        if (ws) { ws.close(); ws = null }
      }
    }, 5000)
  } catch {
    statusMessage.value = 'Failed to connect'
  }
}

function retryConnect() {
  cleanup()
  connectStream()
}

async function trySnapshot() {
  try {
    const response = await fetch(`${config.public.apiUrl}/api/cameras/${props.camera.id}/snapshot`)
    if (response.ok && response.headers.get('content-type')?.includes('image')) {
      const blob = await response.blob()
      snapshotUrl.value = URL.createObjectURL(blob)
    } else {
      statusMessage.value = 'Snapshot failed — camera not reachable'
    }
  } catch {
    statusMessage.value = 'Snapshot failed'
  }
}
</script>
