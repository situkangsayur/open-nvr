<template>
  <div class="w-full h-full bg-black relative overflow-hidden">
    <!-- Live snapshot feed -->
    <img
      v-if="currentSrc"
      :src="currentSrc"
      class="w-full h-full object-contain"
      @load="onImageLoad"
      @error="onImageError"
    />

    <!-- Loading -->
    <div v-if="!currentSrc && loading" class="absolute inset-0 flex items-center justify-center">
      <div class="text-gray-500 text-sm animate-pulse">{{ camera.name }}...</div>
    </div>

    <!-- Offline -->
    <div v-if="!currentSrc && !loading" class="absolute inset-0 flex flex-col items-center justify-center p-3 text-center">
      <div class="text-gray-500 text-sm mb-1">{{ camera.name }}</div>
      <div class="text-gray-600 text-xs mb-2">{{ errorMsg || 'Offline' }}</div>
      <button @click="start" class="text-xs bg-primary-600 hover:bg-primary-700 text-white px-3 py-1 rounded">Retry</button>
    </div>

    <!-- LIVE badge -->
    <div v-if="currentSrc && isLive" class="absolute bottom-1.5 right-1.5">
      <span class="text-[10px] bg-red-600 text-white px-1.5 py-0.5 rounded font-bold">LIVE</span>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ camera: any }>()
const config = useRuntimeConfig()

const currentSrc = ref('')
const nextSrc = ref('')
const loading = ref(false)
const isLive = ref(false)
const errorMsg = ref('')

let pollTimer: ReturnType<typeof setTimeout> | null = null
let failCount = 0

onMounted(() => {
  if (props.camera.status === 'online') start()
  else errorMsg.value = 'Camera offline'
})

onUnmounted(() => stop())

watch(() => props.camera.status, (s) => {
  if (s === 'online' && !pollTimer) start()
  else if (s !== 'online') { stop(); errorMsg.value = 'Camera offline' }
})

function start() {
  stop()
  loading.value = true
  errorMsg.value = ''
  failCount = 0
  grabNext()
}

function stop() {
  if (pollTimer) { clearTimeout(pollTimer); pollTimer = null }
  isLive.value = false
}

async function grabNext() {
  try {
    const url = `${config.public.apiUrl}/api/cameras/${props.camera.id}/snapshot?t=${Date.now()}`
    const resp = await fetch(url)
    if (resp.ok && resp.headers.get('content-type')?.includes('image')) {
      const blob = await resp.blob()
      const objUrl = URL.createObjectURL(blob)

      // Swap: revoke old, show new
      const old = currentSrc.value
      currentSrc.value = objUrl
      if (old) URL.revokeObjectURL(old)

      loading.value = false
      isLive.value = true
      failCount = 0

      // Schedule next grab immediately (pipeline: grab while displaying)
      pollTimer = setTimeout(grabNext, 500)
    } else {
      fail('No image')
    }
  } catch {
    fail('Fetch failed')
  }
}

function fail(msg: string) {
  failCount++
  loading.value = false
  if (failCount > 5) {
    isLive.value = false
    errorMsg.value = msg
    // Slow retry
    pollTimer = setTimeout(grabNext, 10000)
  } else {
    pollTimer = setTimeout(grabNext, 2000)
  }
}

function onImageLoad() {
  // Image displayed successfully
}

function onImageError() {
  // Will be retried on next poll
}
</script>
