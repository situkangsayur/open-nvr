<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">Live View</h1>
      <div class="flex gap-2">
        <button v-for="cols in [1, 2, 3, 4]" :key="cols" @click="gridCols = cols"
          :class="gridCols === cols ? 'bg-primary-600' : 'bg-nvr-card hover:bg-nvr-border'"
          class="w-8 h-8 rounded text-sm font-medium border border-nvr-border transition-colors">
          {{ cols }}
        </button>
      </div>
    </div>

    <div v-if="cameras.length === 0" class="text-gray-400 text-center py-12">
      No cameras configured. <NuxtLink to="/cameras" class="text-primary-400 hover:underline">Add cameras</NuxtLink> first.
    </div>
    <div v-else :class="`grid gap-2 grid-cols-${gridCols}`">
      <div v-for="camera in cameras" :key="camera.id" class="bg-nvr-card rounded-lg border border-nvr-border overflow-hidden">
        <div class="aspect-video bg-black flex items-center justify-center relative">
          <CameraPlayer :camera-id="camera.id" />
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

onMounted(async () => {
  try {
    cameras.value = await useApi<any[]>('/api/cameras')
  } catch (e) {
    console.error('Failed to load cameras', e)
  }
})
</script>
