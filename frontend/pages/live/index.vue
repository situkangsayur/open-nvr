<template>
  <div @keydown="onKeydown" tabindex="0" class="outline-none">
    <div class="flex justify-between items-center mb-3">
      <h1 class="text-xl font-bold">Live View</h1>
      <div class="flex gap-2 items-center">
        <!-- Layout always visible -->
        <div class="flex gap-1">
          <button v-for="cols in [1, 2, 3, 4]" :key="cols" @click="gridCols = cols; fullscreenId = null"
            :class="gridCols === cols && !fullscreenId ? 'bg-primary-600 text-white' : 'bg-gray-100 dark:bg-nvr-darker text-gray-700 dark:text-gray-300'"
            class="w-8 h-8 rounded text-xs font-medium border border-gray-200 dark:border-nvr-border transition-colors">
            {{ cols }}
          </button>
        </div>
      </div>
    </div>

    <!-- Empty -->
    <div v-if="cameras.length === 0" class="text-center py-20">
      <p class="text-gray-400 text-lg mb-4">No cameras configured</p>
      <NuxtLink to="/cameras" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg text-sm">Add Camera</NuxtLink>
    </div>

    <!-- Fullscreen -->
    <div v-else-if="fullscreenCamera" class="relative" @mousemove="showOverlay = true" @mouseleave="hideOverlayDelayed">
      <div @dblclick="fullscreenId = null" class="bg-black rounded-lg overflow-hidden cursor-pointer" style="height: calc(100vh - 120px)">
        <CameraPlayerLive :camera="fullscreenCamera" />
      </div>
      <!-- Top bar -->
      <div class="absolute top-0 left-0 right-0 p-3 flex justify-between bg-gradient-to-b from-black/60 to-transparent transition-opacity" :class="showOverlay ? 'opacity-100' : 'opacity-0'">
        <div class="flex items-center gap-2">
          <span :class="fullscreenCamera.status === 'online' ? 'bg-green-500' : 'bg-red-500'" class="w-2.5 h-2.5 rounded-full"></span>
          <span class="text-sm text-white font-medium">{{ fullscreenCamera.name }}</span>
        </div>
        <button @click="fullscreenId = null" class="text-white bg-white/20 hover:bg-white/30 px-4 py-2 rounded-lg text-sm transition-colors">
          Exit Fullscreen
        </button>
      </div>
      <!-- PTZ -->
      <div v-if="showOverlay" class="absolute bottom-4 right-4">
        <PtzPanel :camera-id="fullscreenCamera.id" size="large" :camera-name="fullscreenCamera.name" />
      </div>
    </div>

    <!-- Grid -->
    <div v-else class="relative">
      <div :style="`display: grid; grid-template-columns: repeat(${gridCols}, 1fr); gap: 0.5rem;`">
        <div v-for="camera in cameras" :key="camera.id"
          class="rounded-lg border-2 overflow-hidden cursor-pointer transition-colors"
          :class="selectedId === camera.id ? 'border-primary-500' : 'border-gray-200 dark:border-nvr-border hover:border-primary-300'"
          @click="selectedId = camera.id"
          @dblclick="fullscreenId = camera.id">
          <div class="aspect-video bg-black">
            <CameraPlayerLive :camera="camera" />
          </div>
        </div>
      </div>

      <!-- Shared PTZ (bottom right, for selected camera) -->
      <div v-if="selectedCamera" class="fixed bottom-4 right-4 z-30">
        <PtzPanel :camera-id="selectedCamera.id" size="medium" :camera-name="selectedCamera.name" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const cameras = ref<any[]>([])
const gridCols = ref(2)
const fullscreenId = ref<string | null>(null)
const selectedId = ref<string | null>(null)
const showOverlay = ref(true)
let overlayTimer: ReturnType<typeof setTimeout> | null = null

const fullscreenCamera = computed(() =>
  fullscreenId.value ? cameras.value.find(c => c.id === fullscreenId.value) : null
)
const selectedCamera = computed(() =>
  selectedId.value ? cameras.value.find(c => c.id === selectedId.value) : null
)

const hideOverlayDelayed = () => {
  if (overlayTimer) clearTimeout(overlayTimer)
  overlayTimer = setTimeout(() => { showOverlay.value = false }, 3000)
}

const onKeydown = async (e: KeyboardEvent) => {
  const camId = fullscreenId.value || selectedId.value
  if (!camId) return
  const map: Record<string, string> = {
    ArrowUp: 'tilt_up', ArrowDown: 'tilt_down',
    ArrowLeft: 'pan_left', ArrowRight: 'pan_right',
    '+': 'zoom_in', '=': 'zoom_in', '-': 'zoom_out',
  }
  const action = map[e.key]
  if (action) {
    e.preventDefault()
    try {
      await useApi(`/api/cameras/${camId}/ptz`, { method: 'POST', body: { action, speed: 0.5 } })
      setTimeout(async () => { try { await useApi(`/api/cameras/${camId}/ptz`, { method: 'POST', body: { action: 'stop', speed: 0 } }) } catch {} }, 300)
    } catch {}
  }
}

onMounted(async () => {
  try {
    cameras.value = await useApi<any[]>('/api/cameras')
    if (cameras.value.length > 0) selectedId.value = cameras.value[0].id
  } catch {}
})
</script>
