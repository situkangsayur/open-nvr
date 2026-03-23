<template>
  <div>
    <div class="flex justify-between items-center mb-3">
      <h1 class="text-xl font-bold">Live View</h1>
      <div class="flex gap-2 items-center">
        <button @click="showControls = !showControls" class="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-white">
          {{ showControls ? 'Hide' : 'Controls' }}
        </button>
        <div v-if="showControls" class="flex gap-1">
          <button v-for="cols in [1, 2, 3, 4]" :key="cols" @click="gridCols = cols; fullscreenId = null"
            :class="gridCols === cols && !fullscreenId ? 'bg-primary-600 text-white' : 'bg-gray-100 dark:bg-nvr-darker text-gray-700 dark:text-gray-300'"
            class="w-8 h-8 rounded text-xs font-medium border border-gray-200 dark:border-nvr-border transition-colors">
            {{ cols }}
          </button>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="cameras.length === 0" class="text-center py-20">
      <p class="text-gray-400 text-lg mb-4">No cameras configured</p>
      <div class="flex gap-3 justify-center">
        <NuxtLink to="/cameras" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg text-sm">Add Camera</NuxtLink>
        <NuxtLink to="/cameras/discover" class="bg-gray-100 dark:bg-nvr-darker text-gray-700 dark:text-gray-300 px-4 py-2 rounded-lg border border-gray-200 dark:border-nvr-border text-sm">Discover</NuxtLink>
      </div>
    </div>

    <!-- Fullscreen single camera -->
    <div v-else-if="fullscreenCamera" class="relative" @mousemove="showOverlay = true" @mouseleave="hideOverlayTimer">
      <div @dblclick="fullscreenId = null" class="bg-black rounded-lg overflow-hidden cursor-pointer" style="height: calc(100vh - 120px)">
        <CameraPlayerLive :camera="fullscreenCamera" class="w-full h-full" />
      </div>

      <!-- Top bar -->
      <div class="absolute top-0 left-0 right-0 p-3 flex justify-between items-center bg-gradient-to-b from-black/60 to-transparent transition-opacity" :class="showOverlay ? 'opacity-100' : 'opacity-0'">
        <div class="flex items-center gap-2">
          <span :class="fullscreenCamera.status === 'online' ? 'bg-green-500' : 'bg-red-500'" class="w-2.5 h-2.5 rounded-full"></span>
          <span class="text-sm text-white font-medium">{{ fullscreenCamera.name }}</span>
        </div>
        <button @click="fullscreenId = null" class="text-white bg-white/20 hover:bg-white/30 px-3 py-1 rounded text-xs transition-colors">
          Exit
        </button>
      </div>

      <!-- PTZ Controls overlay (bottom right) -->
      <div v-if="fullscreenCamera.ptz_capable && showOverlay" class="absolute bottom-4 right-4 transition-opacity">
        <div class="bg-black/60 backdrop-blur-sm rounded-xl p-3">
          <div class="grid grid-cols-3 gap-1 w-28 mb-2">
            <div></div>
            <button @mousedown="ptz('tilt_up')" @mouseup="ptz('stop')" class="ptz-btn">&#9650;</button>
            <div></div>
            <button @mousedown="ptz('pan_left')" @mouseup="ptz('stop')" class="ptz-btn">&#9664;</button>
            <button @click="ptz('home')" class="ptz-btn text-xs">H</button>
            <button @mousedown="ptz('pan_right')" @mouseup="ptz('stop')" class="ptz-btn">&#9654;</button>
            <div></div>
            <button @mousedown="ptz('tilt_down')" @mouseup="ptz('stop')" class="ptz-btn">&#9660;</button>
            <div></div>
          </div>
          <div class="flex gap-1 justify-center">
            <button @mousedown="ptz('zoom_in')" @mouseup="ptz('stop')" class="ptz-btn text-xs px-2">Z+</button>
            <button @mousedown="ptz('zoom_out')" @mouseup="ptz('stop')" class="ptz-btn text-xs px-2">Z-</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Grid view -->
    <div v-else :style="`display: grid; grid-template-columns: repeat(${gridCols}, 1fr); gap: 0.5rem;`">
      <div v-for="camera in cameras" :key="camera.id"
        class="bg-white dark:bg-nvr-card rounded-lg border border-gray-200 dark:border-nvr-border overflow-hidden cursor-pointer hover:border-primary-500 transition-colors relative"
        :class="selectedId === camera.id ? 'ring-2 ring-primary-500' : ''"
        @click="selectedId = camera.id"
        @dblclick="fullscreenId = camera.id">
        <div class="aspect-video bg-black relative">
          <CameraPlayerLive :camera="camera" />
        </div>

        <!-- PTZ mini controls (bottom right of each camera tile) -->
        <div v-if="selectedId === camera.id && camera.ptz_capable" class="absolute bottom-1 right-1 z-10">
          <div class="bg-black/50 backdrop-blur-sm rounded-lg p-1.5 flex gap-1">
            <button @mousedown="ptzFor(camera.id, 'pan_left')" @mouseup="ptzFor(camera.id, 'stop')" class="ptz-sm">&#9664;</button>
            <button @mousedown="ptzFor(camera.id, 'tilt_up')" @mouseup="ptzFor(camera.id, 'stop')" class="ptz-sm">&#9650;</button>
            <button @mousedown="ptzFor(camera.id, 'tilt_down')" @mouseup="ptzFor(camera.id, 'stop')" class="ptz-sm">&#9660;</button>
            <button @mousedown="ptzFor(camera.id, 'pan_right')" @mouseup="ptzFor(camera.id, 'stop')" class="ptz-sm">&#9654;</button>
            <button @mousedown="ptzFor(camera.id, 'zoom_in')" @mouseup="ptzFor(camera.id, 'stop')" class="ptz-sm text-xs">+</button>
            <button @mousedown="ptzFor(camera.id, 'zoom_out')" @mouseup="ptzFor(camera.id, 'stop')" class="ptz-sm text-xs">-</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const cameras = ref<any[]>([])
const gridCols = ref(2)
const fullscreenId = ref<string | null>(null)
const selectedId = ref<string | null>(null)
const showControls = ref(true)
const showOverlay = ref(true)
let overlayTimer: ReturnType<typeof setTimeout> | null = null

const fullscreenCamera = computed(() =>
  fullscreenId.value ? cameras.value.find(c => c.id === fullscreenId.value) : null
)

const hideOverlayTimer = () => {
  if (overlayTimer) clearTimeout(overlayTimer)
  overlayTimer = setTimeout(() => { showOverlay.value = false }, 3000)
}

const ptz = async (action: string) => {
  if (!fullscreenId.value) return
  try {
    await useApi(`/api/cameras/${fullscreenId.value}/ptz`, {
      method: 'POST',
      body: { action, speed: 0.5 },
    })
  } catch {}
}

const ptzFor = async (cameraId: string, action: string) => {
  try {
    await useApi(`/api/cameras/${cameraId}/ptz`, {
      method: 'POST',
      body: { action, speed: 0.5 },
    })
  } catch {}
}

onMounted(async () => {
  try {
    cameras.value = await useApi<any[]>('/api/cameras')
    if (cameras.value.length > 0) {
      selectedId.value = cameras.value[0].id
    }
  } catch {}
})
</script>

<style scoped>
.ptz-btn {
  @apply bg-white/20 hover:bg-white/40 active:bg-primary-600 text-white rounded p-2 text-center transition-colors select-none text-sm;
}
.ptz-sm {
  @apply bg-white/20 hover:bg-white/40 active:bg-primary-600 text-white rounded w-6 h-6 flex items-center justify-center text-xs transition-colors select-none;
}
</style>
