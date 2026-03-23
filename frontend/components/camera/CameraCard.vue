<template>
  <div class="bg-white dark:bg-nvr-card rounded-lg border border-gray-200 dark:border-nvr-border overflow-hidden">
    <div class="aspect-video bg-black flex items-center justify-center relative">
      <span class="text-gray-600 text-sm">Preview</span>
      <!-- Status badge on preview -->
      <div class="absolute top-2 right-2">
        <span :class="statusBadgeClass" class="text-xs px-2 py-0.5 rounded font-medium">
          {{ camera.status }}
        </span>
      </div>
    </div>
    <div class="p-3">
      <div class="flex items-center justify-between mb-1">
        <h3 class="font-medium text-sm truncate">{{ camera.name }}</h3>
        <span :class="statusClass" class="w-2.5 h-2.5 rounded-full flex-shrink-0 ml-2 ring-2 ring-white dark:ring-nvr-card"></span>
      </div>
      <div class="text-xs text-gray-500 dark:text-gray-400 space-y-0.5">
        <div>{{ camera.protocol_type }} &middot; {{ camera.connection_type }}</div>
        <div v-if="camera.brand">{{ camera.brand }} {{ camera.model || '' }}</div>
        <div class="text-gray-600 dark:text-gray-500 break-all">{{ extractHost(camera.stream_url) }}</div>
      </div>
      <!-- Connection test result -->
      <div v-if="testStatus" class="mt-2 text-xs p-1.5 rounded" :class="testStatus.ok ? 'bg-green-500/10 text-green-400' : 'bg-red-500/10 text-red-400'">
        {{ testStatus.message }}
      </div>
      <div class="flex justify-end gap-2 mt-3">
        <NuxtLink :to="`/cameras/${camera.id}`" class="text-xs text-primary-400 hover:underline">Edit</NuxtLink>
        <button @click="testCamera" :disabled="testingCamera" class="text-xs text-green-400 hover:underline">
          {{ testingCamera ? 'Testing...' : 'Test' }}
        </button>
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

const testingCamera = ref(false)
const testStatus = ref<{ ok: boolean; message: string } | null>(null)

const statusClass = computed(() => ({
  'bg-green-500': props.camera.status === 'online',
  'bg-red-500': props.camera.status === 'offline',
  'bg-yellow-500': props.camera.status === 'connecting',
  'bg-orange-500': props.camera.status === 'error',
}))

const statusBadgeClass = computed(() => {
  const base = {
    online: 'bg-green-600/80 text-green-100',
    offline: 'bg-red-600/80 text-red-100',
    connecting: 'bg-yellow-600/80 text-yellow-100',
    error: 'bg-orange-600/80 text-orange-100',
  }
  return base[props.camera.status as keyof typeof base] || 'bg-gray-600/80 text-gray-100'
})

function extractHost(url: string) {
  if (!url) return ''
  try {
    const match = url.match(/:\/\/([^/:]+)/)
    return match ? match[1] : url
  } catch {
    return url
  }
}

const testCamera = async () => {
  testingCamera.value = true
  testStatus.value = null
  try {
    const result = await useApi<any>(`/api/cameras/${props.camera.id}/test`, { method: 'POST' })
    if (result.status === 'ok') {
      testStatus.value = { ok: true, message: `Reachable - ${result.stream_info?.video_codec || 'connected'}` }
    } else if (result.status === 'timeout') {
      testStatus.value = { ok: false, message: 'Unreachable - check network/AP isolation' }
    } else {
      testStatus.value = { ok: false, message: result.error || 'Connection failed' }
    }
  } catch (e: any) {
    testStatus.value = { ok: false, message: 'Test failed - server error' }
  }
  testingCamera.value = false
}

const toggleRecording = async () => {
  const action = props.camera.status === 'online' ? 'stop' : 'start'
  try {
    await useApi(`/api/cameras/${props.camera.id}/${action}`, { method: 'POST' })
  } catch {}
}
</script>
