<template>
  <div class="w-full h-full bg-black relative overflow-hidden">
    <!-- HLS Video (with audio) -->
    <video
      v-show="hlsReady"
      ref="videoEl"
      autoplay
      playsinline
      class="w-full h-full object-contain"
      @playing="isPlaying = true; loading = false"
      @stalled="onStall"
      @error="onHlsError"
    />

    <!-- Snapshot fallback (while HLS loads or as backup) -->
    <img
      v-if="!hlsReady && currentSnapshot"
      :src="currentSnapshot"
      class="w-full h-full object-contain"
    />

    <!-- Loading -->
    <div v-if="loading && !currentSnapshot" class="absolute inset-0 flex items-center justify-center">
      <div class="text-gray-500 text-sm animate-pulse">{{ camera.name }}...</div>
    </div>

    <!-- Offline -->
    <div v-if="showOffline" class="absolute inset-0 flex flex-col items-center justify-center p-3 text-center">
      <div class="text-gray-500 text-sm mb-1">{{ camera.name }}</div>
      <div class="text-gray-600 text-xs mb-2">{{ errorMsg || 'Offline' }}</div>
      <button @click="startStream" class="text-xs bg-primary-600 hover:bg-primary-700 text-white px-3 py-1 rounded">Retry</button>
    </div>

    <!-- LIVE badge -->
    <div v-if="isPlaying || currentSnapshot" class="absolute bottom-1.5 right-1.5">
      <span class="text-[10px] bg-red-600 text-white px-1.5 py-0.5 rounded font-bold">
        {{ hlsReady ? 'LIVE' : 'SNAP' }}
      </span>
    </div>

    <!-- Audio toggle (HLS only) -->
    <button v-if="hlsReady" @click="toggleMute" class="absolute bottom-1.5 left-1.5 text-white bg-black/50 hover:bg-black/70 rounded px-2 py-1 text-xs transition-colors">
      {{ isMuted ? 'Unmute' : 'Mute' }}
    </button>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ camera: any }>()
const config = useRuntimeConfig()
const { markStarted, isStarted, cacheSnapshot, getCachedSnapshot } = useHlsCache()

const videoEl = ref<HTMLVideoElement | null>(null)
const currentSnapshot = ref('')
const loading = ref(false)
const isPlaying = ref(false)
const hlsReady = ref(false)
const isMuted = ref(true)
const errorMsg = ref('')

let snapTimer: ReturnType<typeof setTimeout> | null = null
let failCount = 0

const showOffline = computed(() => !isPlaying.value && !currentSnapshot.value && !loading.value)

onMounted(() => {
  // Use cached snapshot immediately (no delay on layout change)
  const cached = getCachedSnapshot(props.camera.id)
  if (cached) {
    currentSnapshot.value = cached
    isPlaying.value = true
  }

  if (props.camera.status === 'online') {
    startStream()
  } else {
    errorMsg.value = 'Camera offline'
  }
})

onUnmounted(() => {
  clearTimers()
  // Don't revoke snapshot - keep in cache for layout switches
})

function clearTimers() {
  if (snapTimer) { clearTimeout(snapTimer); snapTimer = null }
}

async function startStream() {
  loading.value = true
  errorMsg.value = ''
  failCount = 0

  // Start snapshot polling immediately (instant feedback)
  grabSnapshot()

  // Start HLS in parallel (takes ~4s to be ready)
  startHls()
}

async function startHls() {
  const apiUrl = config.public.apiUrl
  const camId = props.camera.id

  try {
    // Only request start if not already started globally
    if (!isStarted(camId)) {
      await fetch(`${apiUrl}/api/hls/${camId}/start`, { method: 'POST' })
      markStarted(camId)
    }

    // Poll until playlist is available (max 8 seconds)
    const playlistUrl = `${apiUrl}/api/hls/${camId}/stream.m3u8`
    for (let i = 0; i < 8; i++) {
      await new Promise(r => setTimeout(r, 1000))
      try {
        const resp = await fetch(playlistUrl)
        if (resp.ok && (await resp.text()).includes('#EXTINF')) {
          // Playlist ready with segments
          hlsReady.value = true
          await nextTick()
          if (videoEl.value) {
            videoEl.value.src = playlistUrl
            videoEl.value.muted = isMuted.value
            videoEl.value.play().catch(() => {})
          }
          // Stop snapshot polling once HLS works
          clearTimers()
          return
        }
      } catch {}
    }
  } catch {}

  // HLS didn't start - snapshot polling continues as fallback
}

async function grabSnapshot() {
  try {
    const url = `${config.public.apiUrl}/api/cameras/${props.camera.id}/snapshot?t=${Date.now()}`
    const resp = await fetch(url)
    if (resp.ok && resp.headers.get('content-type')?.includes('image')) {
      const blob = await resp.blob()
      const objUrl = URL.createObjectURL(blob)

      // Revoke old if it's not the global cache
      if (currentSnapshot.value && currentSnapshot.value !== getCachedSnapshot(props.camera.id)) {
        URL.revokeObjectURL(currentSnapshot.value)
      }

      currentSnapshot.value = objUrl
      cacheSnapshot(props.camera.id, objUrl)
      loading.value = false
      isPlaying.value = true
      failCount = 0

      // Continue polling if HLS not ready
      if (!hlsReady.value) {
        snapTimer = setTimeout(grabSnapshot, 1000)
      }
    } else {
      onSnapFail()
    }
  } catch {
    onSnapFail()
  }
}

function onSnapFail() {
  failCount++
  if (failCount > 5) {
    loading.value = false
    if (!currentSnapshot.value) errorMsg.value = 'Camera unreachable'
    snapTimer = setTimeout(grabSnapshot, 10000)
  } else {
    snapTimer = setTimeout(grabSnapshot, 2000)
  }
}

function onStall() {
  // HLS stalled, will auto-recover
}

function onHlsError() {
  hlsReady.value = false
  // Fall back to snapshot polling
  if (!snapTimer) grabSnapshot()
}

function toggleMute() {
  isMuted.value = !isMuted.value
  if (videoEl.value) videoEl.value.muted = isMuted.value
}
</script>
