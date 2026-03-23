<template>
  <div class="w-full h-full bg-black relative overflow-hidden">
    <!-- Live snapshot feed (polling mode) -->
    <img
      v-if="snapshotSrc"
      :src="snapshotSrc"
      class="w-full h-full object-contain"
      @error="onImageError"
    />

    <!-- Loading / connecting state -->
    <div v-if="!snapshotSrc && loading" class="absolute inset-0 flex items-center justify-center">
      <div class="text-gray-500 text-sm">Connecting to {{ camera.name }}...</div>
    </div>

    <!-- Offline / error state -->
    <div v-if="!snapshotSrc && !loading" class="absolute inset-0 flex flex-col items-center justify-center p-4 text-center">
      <div class="text-gray-600 text-sm mb-2">{{ camera.name }}</div>
      <div class="text-gray-500 text-xs mb-1">{{ camera.status }} &middot; {{ camera.protocol_type }}</div>
      <div class="text-gray-600 text-xs mb-3 break-all max-w-xs">{{ camera.stream_url }}</div>
      <div v-if="errorMsg" class="text-yellow-500/80 text-xs mb-3">{{ errorMsg }}</div>
      <button @click="startPolling" class="text-xs bg-primary-600 hover:bg-primary-700 text-white px-3 py-1.5 rounded transition-colors">
        Retry
      </button>
    </div>

    <!-- Live badge -->
    <div v-if="snapshotSrc" class="absolute bottom-2 right-2">
      <span class="text-xs bg-black/70 px-2 py-0.5 rounded font-medium" :class="isLive ? 'text-green-400' : 'text-yellow-400'">
        {{ isLive ? 'LIVE' : 'PAUSED' }}
      </span>
    </div>

    <!-- Camera name overlay -->
    <div v-if="snapshotSrc" class="absolute top-2 left-2">
      <span class="text-xs text-white bg-black/60 px-2 py-0.5 rounded">{{ camera.name }}</span>
    </div>

    <!-- FPS indicator -->
    <div v-if="snapshotSrc" class="absolute top-2 right-2">
      <span class="text-xs text-gray-400 bg-black/60 px-1.5 py-0.5 rounded">{{ fps }}fps</span>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ camera: any }>()
const config = useRuntimeConfig()

const snapshotSrc = ref('')
const loading = ref(false)
const errorMsg = ref('')
const isLive = ref(false)
const fps = ref(0)

let pollInterval: ReturnType<typeof setInterval> | null = null
let frameCount = 0
let fpsInterval: ReturnType<typeof setInterval> | null = null
let lastFrameTime = 0

onMounted(() => {
  if (props.camera.status === 'online') {
    startPolling()
  } else {
    errorMsg.value = 'Camera offline'
  }
})

onUnmounted(() => {
  stopPolling()
})

// Watch for camera status changes
watch(() => props.camera.status, (newStatus) => {
  if (newStatus === 'online' && !pollInterval) {
    startPolling()
  } else if (newStatus !== 'online') {
    stopPolling()
    errorMsg.value = 'Camera offline'
  }
})

function startPolling() {
  stopPolling()
  loading.value = true
  errorMsg.value = ''
  frameCount = 0

  // Grab first frame
  grabFrame()

  // Poll every 1 second for "live" feed
  pollInterval = setInterval(grabFrame, 1000)

  // Calculate FPS every 3 seconds
  fpsInterval = setInterval(() => {
    fps.value = Math.round(frameCount / 3)
    frameCount = 0
  }, 3000)
}

function stopPolling() {
  if (pollInterval) { clearInterval(pollInterval); pollInterval = null }
  if (fpsInterval) { clearInterval(fpsInterval); fpsInterval = null }
  isLive.value = false
}

async function grabFrame() {
  try {
    const timestamp = Date.now()
    const url = `${config.public.apiUrl}/api/cameras/${props.camera.id}/snapshot?t=${timestamp}`

    const response = await fetch(url)
    if (response.ok && response.headers.get('content-type')?.includes('image')) {
      const blob = await response.blob()

      // Revoke old URL to prevent memory leak
      if (snapshotSrc.value) {
        URL.revokeObjectURL(snapshotSrc.value)
      }

      snapshotSrc.value = URL.createObjectURL(blob)
      loading.value = false
      isLive.value = true
      errorMsg.value = ''
      frameCount++
      lastFrameTime = timestamp
    } else {
      handleError('Camera not responding')
    }
  } catch (e) {
    handleError('Connection failed')
  }
}

function handleError(msg: string) {
  loading.value = false
  if (Date.now() - lastFrameTime > 10000) {
    // No frame for 10 seconds
    isLive.value = false
    errorMsg.value = msg
  }
}

function onImageError() {
  // Image failed to load, will retry on next poll
}
</script>
