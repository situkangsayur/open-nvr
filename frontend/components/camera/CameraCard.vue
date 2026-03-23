<template>
  <div class="bg-white dark:bg-nvr-card rounded-lg border border-gray-200 dark:border-nvr-border overflow-hidden">
    <div class="aspect-video bg-black flex items-center justify-center">
      <span class="text-gray-600 text-sm">Preview</span>
    </div>
    <div class="p-3">
      <div class="flex items-center justify-between mb-1">
        <h3 class="font-medium text-sm truncate">{{ camera.name }}</h3>
        <span :class="statusClass" class="w-2 h-2 rounded-full flex-shrink-0 ml-2"></span>
      </div>
      <div class="text-xs text-gray-500 dark:text-gray-400 space-y-0.5">
        <div>{{ camera.protocol_type }} &middot; {{ camera.connection_type }}</div>
        <div v-if="camera.brand">{{ camera.brand }} {{ camera.model || '' }}</div>
      </div>
      <div class="flex justify-end gap-2 mt-3">
        <NuxtLink :to="`/cameras/${camera.id}`" class="text-xs text-primary-400 hover:underline">Edit</NuxtLink>
        <button @click="testCamera" class="text-xs text-green-400 hover:underline">Test</button>
        <button @click="toggleRecording" class="text-xs text-yellow-400 hover:underline">
          {{ camera.status === 'online' ? 'Stop' : 'Start' }}
        </button>
        <button @click="$emit('delete', camera.id)" class="text-xs text-red-400 hover:underline">Delete</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ camera: any }>()
defineEmits<{ delete: [id: string] }>()
const statusClass = computed(() => ({
  'bg-green-500': props.camera.status === 'online',
  'bg-red-500': props.camera.status === 'offline',
  'bg-yellow-500': props.camera.status === 'connecting',
  'bg-orange-500': props.camera.status === 'error',
}))

const testCamera = async () => {
  try {
    const result = await useApi<any>(`/api/cameras/${props.camera.id}/test`, { method: 'POST' })
    alert(result.status === 'ok' ? 'Connection OK!' : `Error: ${result.error}`)
  } catch (e) {
    alert('Test failed')
  }
}

const toggleRecording = async () => {
  const action = props.camera.status === 'online' ? 'stop' : 'start'
  try {
    await useApi(`/api/cameras/${props.camera.id}/${action}`, { method: 'POST' })
  } catch {}
}
</script>
