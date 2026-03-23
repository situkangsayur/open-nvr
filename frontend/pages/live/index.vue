<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">Live View</h1>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-5 gap-4">
      <!-- Sidebar controls -->
      <div class="space-y-4">
        <LayoutGridLayoutSelector :current-cols="gridCols" @change="gridCols = $event" />

        <!-- Camera list for selection -->
        <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border">
          <h3 class="text-sm font-semibold text-gray-400 mb-3">Cameras</h3>
          <div class="space-y-1 max-h-64 overflow-y-auto">
            <div v-for="cam in cameras" :key="cam.id"
              @click="selectedCamera = cam"
              :class="selectedCamera?.id === cam.id ? 'bg-primary-600/20 border-primary-500' : 'border-transparent'"
              class="flex items-center gap-2 p-2 rounded cursor-pointer hover:bg-gray-100 dark:bg-nvr-darker text-sm border transition-colors">
              <span :class="cam.status === 'online' ? 'bg-green-500' : 'bg-red-500'" class="w-2 h-2 rounded-full flex-shrink-0"></span>
              <span class="truncate">{{ cam.name }}</span>
            </div>
          </div>
        </div>

        <!-- PTZ for selected camera -->
        <CameraPTZControls v-if="selectedCamera?.ptz_capable" :camera-id="selectedCamera.id" />
      </div>

      <!-- Video Grid -->
      <div class="lg:col-span-4">
        <div v-if="cameras.length === 0" class="text-gray-400 text-center py-12">
          No cameras configured. <NuxtLink to="/cameras" class="text-primary-400 hover:underline">Add cameras</NuxtLink> first.
        </div>
        <div v-else :class="`grid gap-2`" :style="`grid-template-columns: repeat(${gridCols}, 1fr)`">
          <div v-for="camera in cameras" :key="camera.id"
            @click="selectedCamera = camera"
            :class="selectedCamera?.id === camera.id ? 'ring-2 ring-primary-500' : ''"
            class="bg-white dark:bg-nvr-card rounded-lg border border-gray-200 dark:border-nvr-border overflow-hidden cursor-pointer">
            <div class="aspect-video bg-black flex items-center justify-center relative">
              <CameraPlayer :camera-id="camera.id" />
              <div class="absolute top-2 left-2 flex items-center gap-2">
                <span :class="camera.status === 'online' ? 'bg-green-500' : 'bg-red-500'" class="w-2 h-2 rounded-full"></span>
                <span class="text-xs text-white bg-black/60 px-2 py-0.5 rounded">{{ camera.name }}</span>
              </div>
              <div v-if="camera.ptz_capable" class="absolute top-2 right-2">
                <span class="text-xs text-primary-400 bg-black/60 px-1.5 py-0.5 rounded">PTZ</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const cameras = ref<any[]>([])
const gridCols = ref(2)
const selectedCamera = ref<any>(null)

onMounted(async () => {
  try {
    cameras.value = await useApi<any[]>('/api/cameras')
    if (cameras.value.length > 0) {
      selectedCamera.value = cameras.value[0]
    }
  } catch (e) {
    console.error('Failed to load cameras', e)
  }
})
</script>
