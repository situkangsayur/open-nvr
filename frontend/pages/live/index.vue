<template>
  <div>
    <div class="flex justify-between items-center mb-4">
      <h1 class="text-xl font-bold">Live View</h1>
      <div class="flex gap-2 items-center">
        <button @click="showControls = !showControls" class="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-white">
          {{ showControls ? 'Hide Controls' : 'Show Controls' }}
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

    <div v-if="cameras.length === 0" class="text-center py-20">
      <p class="text-gray-400 text-lg mb-4">No cameras configured</p>
      <div class="flex gap-3 justify-center">
        <NuxtLink to="/cameras" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg text-sm">Add Camera</NuxtLink>
        <NuxtLink to="/cameras/discover" class="bg-gray-100 dark:bg-nvr-darker hover:bg-gray-200 dark:hover:bg-nvr-border text-gray-700 dark:text-gray-300 px-4 py-2 rounded-lg border border-gray-200 dark:border-nvr-border text-sm">Discover</NuxtLink>
      </div>
    </div>

    <!-- Fullscreen single camera -->
    <div v-else-if="fullscreenCamera" class="relative">
      <div @dblclick="fullscreenId = null" class="bg-black rounded-lg overflow-hidden cursor-pointer" style="height: calc(100vh - 140px)">
        <CameraPlayerLive :camera="fullscreenCamera" class="w-full h-full" />
      </div>
      <div class="absolute top-3 left-3 flex items-center gap-2">
        <span :class="fullscreenCamera.status === 'online' ? 'bg-green-500' : 'bg-red-500'" class="w-2.5 h-2.5 rounded-full"></span>
        <span class="text-sm text-white bg-black/60 px-2 py-1 rounded">{{ fullscreenCamera.name }}</span>
      </div>
      <button @click="fullscreenId = null" class="absolute top-3 right-3 text-white bg-black/60 px-2 py-1 rounded text-xs hover:bg-black/80">Exit Fullscreen</button>
    </div>

    <!-- Grid view -->
    <div v-else :style="`display: grid; grid-template-columns: repeat(${gridCols}, 1fr); gap: 0.5rem;`">
      <div v-for="camera in cameras" :key="camera.id"
        @dblclick="fullscreenId = camera.id"
        class="bg-white dark:bg-nvr-card rounded-lg border border-gray-200 dark:border-nvr-border overflow-hidden cursor-pointer hover:border-primary-500 transition-colors">
        <div class="aspect-video bg-black relative">
          <CameraPlayerLive :camera="camera" />
          <div class="absolute top-2 left-2 flex items-center gap-2">
            <span :class="camera.status === 'online' ? 'bg-green-500' : 'bg-red-500'" class="w-2 h-2 rounded-full"></span>
            <span class="text-xs text-white bg-black/60 px-2 py-0.5 rounded">{{ camera.name }}</span>
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
const showControls = ref(true)

const fullscreenCamera = computed(() =>
  fullscreenId.value ? cameras.value.find(c => c.id === fullscreenId.value) : null
)

onMounted(async () => {
  try {
    cameras.value = await useApi<any[]>('/api/cameras')
  } catch {}
})
</script>
