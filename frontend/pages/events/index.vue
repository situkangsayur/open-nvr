<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">Detection Events</h1>

    <div class="bg-nvr-card rounded-lg p-4 border border-nvr-border mb-6">
      <div class="flex flex-wrap gap-4 items-end">
        <div>
          <label class="block text-xs text-gray-400 mb-1">Camera</label>
          <select v-model="filter.camera_id" class="bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white text-sm">
            <option value="">All cameras</option>
            <option v-for="cam in cameras" :key="cam.id" :value="cam.id">{{ cam.name }}</option>
          </select>
        </div>
        <div>
          <label class="block text-xs text-gray-400 mb-1">Type</label>
          <select v-model="filter.event_type" class="bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white text-sm">
            <option value="">All types</option>
            <option value="motion">Motion</option>
            <option value="human">Human</option>
            <option value="animal">Animal</option>
            <option value="vehicle">Vehicle</option>
          </select>
        </div>
        <button @click="loadEvents" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded text-sm transition-colors">
          Search
        </button>
      </div>
    </div>

    <div v-if="loading" class="text-gray-400">Loading events...</div>
    <div v-else-if="events.length === 0" class="text-gray-400 text-center py-12">No events found</div>
    <div v-else class="space-y-2">
      <div v-for="event in events" :key="event.id" class="bg-nvr-card rounded-lg p-4 border border-nvr-border flex items-center gap-4">
        <div :class="eventTypeClass(event.event_type)" class="w-10 h-10 rounded-lg flex items-center justify-center text-lg flex-shrink-0">
          {{ eventIcon(event.event_type) }}
        </div>
        <div class="flex-1">
          <div class="flex items-center gap-2">
            <span class="font-medium text-sm capitalize">{{ event.event_type }}</span>
            <span v-if="event.confidence" class="text-xs text-gray-400">{{ (event.confidence * 100).toFixed(0) }}%</span>
          </div>
          <div class="text-xs text-gray-500">
            Camera: {{ getCameraName(event.camera_id) }} &middot; {{ new Date(event.occurred_at).toLocaleString() }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const cameras = ref<any[]>([])
const events = ref<any[]>([])
const loading = ref(false)
const filter = ref({ camera_id: '', event_type: '' })

onMounted(async () => {
  try { cameras.value = await useApi<any[]>('/api/cameras') } catch {}
  await loadEvents()
})

const loadEvents = async () => {
  loading.value = true
  try {
    const params = new URLSearchParams()
    if (filter.value.camera_id) params.set('camera_id', filter.value.camera_id)
    if (filter.value.event_type) params.set('event_type', filter.value.event_type)
    events.value = await useApi<any[]>(`/api/events?${params}`)
  } catch {}
  loading.value = false
}

const getCameraName = (id: string) => cameras.value.find(c => c.id === id)?.name || id.slice(0, 8)

const eventIcon = (type: string) => {
  const icons: Record<string, string> = { motion: '~', human: 'H', animal: 'A', vehicle: 'V', unknown: '?' }
  return icons[type] || '?'
}

const eventTypeClass = (type: string) => {
  const classes: Record<string, string> = {
    motion: 'bg-blue-500/20 text-blue-400',
    human: 'bg-green-500/20 text-green-400',
    animal: 'bg-yellow-500/20 text-yellow-400',
    vehicle: 'bg-purple-500/20 text-purple-400',
  }
  return classes[type] || 'bg-gray-500/20 text-gray-400'
}
</script>
