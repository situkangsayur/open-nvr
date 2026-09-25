<template>
  <div v-if="camera">
    <div class="flex flex-wrap items-center gap-4 mb-6">
      <NuxtLink to="/cameras" class="text-gray-400 hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">{{ camera.name }}</h1>
      <span :class="statusClass" class="w-3 h-3 rounded-full"></span>
      <button @click="showEdit = true" class="ml-auto bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg transition-colors text-sm whitespace-nowrap">
        Edit Camera
      </button>
    </div>

    <div :class="camera.ptz_capable ? 'grid grid-cols-1 lg:grid-cols-3 gap-6' : 'grid grid-cols-1 lg:grid-cols-2 gap-6'">
      <!-- Camera Info -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Camera Details</h2>
        <div class="space-y-3 text-sm">
          <div class="flex justify-between"><span class="text-gray-400">Protocol</span><span>{{ camera.protocol_type }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">Stream URL</span><span class="truncate ml-4">{{ camera.stream_url }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">Brand</span><span>{{ camera.brand || '-' }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">Model</span><span>{{ camera.model || '-' }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">ONVIF URL</span><span class="truncate ml-4">{{ camera.onvif_url || '-' }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">Camera login</span><span>{{ camera.has_credentials ? 'Stored' : 'None' }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">Connection</span><span>{{ camera.connection_type }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">Recording</span><span>{{ camera.recording_mode }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">PTZ</span><span>{{ camera.ptz_capable ? 'Yes' : 'No' }}</span></div>
          <div class="flex justify-between"><span class="text-gray-400">Audio</span><span>{{ camera.audio_capable ? 'Yes' : 'No' }}</span></div>
        </div>
      </div>

      <!-- Detection Zones -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Detection Zones</h2>
        <CameraDetectionZoneEditor
          :camera-id="camera.id"
          :existing-zones="zones"
          @saved="loadZones"
        />
        <div v-if="zones.length > 0" class="mt-4 space-y-2">
          <div v-for="zone in zones" :key="zone.id" class="flex justify-between items-center p-2 bg-gray-100 dark:bg-nvr-darker rounded text-sm">
            <span>{{ zone.name }}</span>
            <button @click="deleteZone(zone.id)" class="text-red-400 hover:underline text-xs">Delete</button>
          </div>
        </div>
      </div>

      <!-- PTZ Controls (if capable) -->
      <div v-if="camera.ptz_capable">
        <CameraPTZControls :camera-id="camera.id" />
      </div>
    </div>

    <!-- Edit Camera Modal -->
    <CameraEditModal
      :open="showEdit"
      :camera-id="String(route.params.id)"
      @close="showEdit = false"
      @saved="onSaved"
    />
  </div>
</template>

<script setup lang="ts">
const route = useRoute()
const camera = ref<any>(null)
const zones = ref<any[]>([])
const showEdit = ref(false)

const onSaved = (updated: any) => {
  camera.value = updated
}

const statusClass = computed(() => ({
  'bg-green-500': camera.value?.status === 'online',
  'bg-red-500': camera.value?.status === 'offline',
  'bg-yellow-500': camera.value?.status === 'connecting',
}))

const loadZones = async () => {
  try {
    zones.value = await useApi<any[]>(`/api/cameras/${route.params.id}/zones`)
  } catch {}
}

const deleteZone = async (zoneId: string) => {
  if (!confirm('Delete this zone?')) return
  try {
    await useApi(`/api/cameras/${route.params.id}/zones/${zoneId}`, { method: 'DELETE' })
    await loadZones()
  } catch {}
}

onMounted(async () => {
  try {
    camera.value = await useApi<any>(`/api/cameras/${route.params.id}`)
  } catch {}
  await loadZones()
})
</script>
