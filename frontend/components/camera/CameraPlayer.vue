<template>
  <div class="w-full h-full flex items-center justify-center bg-black">
    <video ref="videoEl" autoplay muted playsinline class="w-full h-full object-contain"></video>
    <div v-if="!connected" class="absolute inset-0 flex items-center justify-center">
      <span class="text-gray-500 text-sm">{{ connecting ? 'Connecting...' : 'No stream' }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ cameraId: string }>()
const videoEl = ref<HTMLVideoElement | null>(null)
const connected = ref(false)
const connecting = ref(false)
const config = useRuntimeConfig()

onMounted(() => {
  connectStream()
})

onUnmounted(() => {
  // Cleanup WebSocket connection if active
})

const connectStream = () => {
  connecting.value = true
  const wsUrl = config.public.apiUrl.replace('http', 'ws')
  // WebSocket connection for fMP4 streaming will be implemented
  // when the backend RTSP pipeline is ready
  connecting.value = false
}
</script>
