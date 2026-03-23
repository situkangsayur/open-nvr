<template>
  <div class="w-full h-full flex flex-col items-center justify-center bg-gray-900 relative text-center p-3">
    <!-- Stream connected -->
    <video ref="videoEl" autoplay muted playsinline class="w-full h-full object-contain absolute inset-0" v-show="streamActive" />

    <!-- Snapshot image (fallback) -->
    <img v-if="snapshotSrc && !streamActive" :src="snapshotSrc" class="w-full h-full object-contain absolute inset-0" />

    <!-- Offline overlay -->
    <div v-if="!streamActive && !snapshotSrc" class="relative z-10 max-w-xs">
      <div class="text-3xl mb-2 opacity-30">&#x1F4F9;</div>
      <div class="text-gray-400 text-sm font-medium mb-1">{{ camera.name }}</div>
      <div class="text-gray-500 text-xs mb-3">
        <span :class="camera.status === 'online' ? 'text-green-400' : 'text-red-400'">{{ camera.status }}</span>
        <span class="mx-1">&middot;</span>
        {{ camera.protocol_type }}
        <span class="mx-1">&middot;</span>
        {{ camera.connection_type }}
      </div>
      <div class="text-gray-600 text-xs mb-3 break-all">{{ camera.stream_url }}</div>

      <div v-if="errorDetail" class="text-yellow-500/80 text-xs mb-3 bg-yellow-500/10 rounded p-2">
        {{ errorDetail }}
      </div>

      <div class="flex gap-2 justify-center flex-wrap">
        <button @click="testConnection" :disabled="testing" class="text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 px-3 py-1.5 rounded transition-colors">
          {{ testing ? 'Testing...' : 'Test Connection' }}
        </button>
        <button @click="trySnapshot" :disabled="snapping" class="text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 px-3 py-1.5 rounded transition-colors">
          {{ snapping ? 'Capturing...' : 'Snapshot' }}
        </button>
        <button @click="retryStream" class="text-xs bg-primary-600 hover:bg-primary-700 text-white px-3 py-1.5 rounded transition-colors">
          Retry Stream
        </button>
      </div>

      <div v-if="testResult" class="mt-2 text-xs p-2 rounded" :class="testResult.ok ? 'bg-green-500/10 text-green-400' : 'bg-red-500/10 text-red-400'">
        {{ testResult.message }}
      </div>
    </div>

    <!-- Live badge -->
    <div v-if="streamActive" class="absolute bottom-2 right-2 z-20">
      <span class="text-xs text-green-400 bg-black/70 px-2 py-0.5 rounded font-medium">LIVE</span>
    </div>

    <!-- Status badge -->
    <div v-if="!streamActive" class="absolute top-2 right-2 z-20">
      <span :class="camera.status === 'online' ? 'bg-green-500' : 'bg-red-500'" class="w-2 h-2 rounded-full inline-block"></span>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ camera: any }>()
const config = useRuntimeConfig()
const videoEl = ref<HTMLVideoElement | null>(null)
const streamActive = ref(false)
const snapshotSrc = ref('')
const errorDetail = ref('')
const testing = ref(false)
const snapping = ref(false)
const testResult = ref<{ ok: boolean; message: string } | null>(null)

let ws: WebSocket | null = null
let retryTimeout: ReturnType<typeof setTimeout> | null = null

onMounted(() => {
  attemptStream()
})

onUnmounted(() => {
  if (ws) { ws.close(); ws = null }
  if (retryTimeout) clearTimeout(retryTimeout)
})

function attemptStream() {
  if (!import.meta.client) return
  errorDetail.value = ''
  testResult.value = null

  const wsUrl = config.public.apiUrl.replace(/^http/, 'ws')
  try {
    ws = new WebSocket(`${wsUrl}/ws/stream/${props.camera.id}`)
    ws.binaryType = 'arraybuffer'

    const connectTimeout = setTimeout(() => {
      if (!streamActive.value && ws) {
        ws.close()
        errorDetail.value = 'Stream timeout — camera may be unreachable from server. Check network connectivity.'
      }
    }, 8000)

    ws.onopen = () => {
      streamActive.value = true
      clearTimeout(connectTimeout)
    }

    ws.onmessage = () => {
      streamActive.value = true
      // MSE would go here for proper video
    }

    ws.onclose = () => {
      streamActive.value = false
      clearTimeout(connectTimeout)
      if (!errorDetail.value) {
        errorDetail.value = 'Stream ended. Camera may be offline or RTSP not enabled.'
      }
    }

    ws.onerror = () => {
      streamActive.value = false
      clearTimeout(connectTimeout)
      errorDetail.value = 'WebSocket connection failed.'
    }
  } catch {
    errorDetail.value = 'Failed to initialize stream connection.'
  }
}

function retryStream() {
  if (ws) { ws.close(); ws = null }
  streamActive.value = false
  snapshotSrc.value = ''
  attemptStream()
}

async function testConnection() {
  testing.value = true
  testResult.value = null
  try {
    const result = await useApi<any>(`/api/cameras/${props.camera.id}/test`, { method: 'POST' })
    if (result.status === 'ok') {
      testResult.value = { ok: true, message: `Connected! Codec: ${result.stream_info?.video_codec || 'unknown'}` }
    } else if (result.status === 'timeout') {
      testResult.value = { ok: false, message: 'Timeout — camera unreachable from server. Check if server and camera are on same network.' }
    } else {
      testResult.value = { ok: false, message: result.error || 'Connection failed' }
    }
  } catch (e: any) {
    testResult.value = { ok: false, message: e?.data?.error?.message || 'Test failed' }
  }
  testing.value = false
}

async function trySnapshot() {
  snapping.value = true
  try {
    const url = `${config.public.apiUrl}/api/cameras/${props.camera.id}/snapshot`
    const resp = await fetch(url)
    if (resp.ok && resp.headers.get('content-type')?.includes('image')) {
      const blob = await resp.blob()
      snapshotSrc.value = URL.createObjectURL(blob)
    } else {
      errorDetail.value = 'Snapshot failed — camera unreachable'
    }
  } catch {
    errorDetail.value = 'Snapshot failed'
  }
  snapping.value = false
}
</script>
