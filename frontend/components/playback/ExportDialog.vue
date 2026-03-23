<template>
  <div v-if="show" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border w-full max-w-md">
      <h2 class="text-xl font-bold mb-4">Export Recording</h2>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">Camera</label>
          <select v-model="cameraId" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white">
            <option value="">Select camera</option>
            <option v-for="cam in cameras" :key="cam.id" :value="cam.id">{{ cam.name }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">Start Time</label>
          <input v-model="startTime" type="datetime-local" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">End Time</label>
          <input v-model="endTime" type="datetime-local" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
        </div>
      </div>

      <div v-if="exporting" class="mt-4 text-sm text-gray-500 dark:text-gray-400">
        Preparing export...
      </div>

      <div class="flex justify-end gap-3 mt-6">
        <button @click="$emit('close')" class="px-4 py-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-white transition-colors">Cancel</button>
        <button @click="doExport" :disabled="!cameraId || !startTime || !endTime || exporting" class="bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white px-4 py-2 rounded-lg transition-colors">
          Export
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ show: boolean; cameras: any[] }>()
const emit = defineEmits<{ close: [] }>()
const { success, error: showError } = useToast()

const cameraId = ref('')
const startTime = ref('')
const endTime = ref('')
const exporting = ref(false)

const doExport = async () => {
  if (!cameraId.value || !startTime.value || !endTime.value) return
  exporting.value = true
  try {
    const start = new Date(startTime.value).toISOString()
    const end = new Date(endTime.value).toISOString()
    const response = await useApi<any>(`/api/recordings/export?camera_id=${cameraId.value}&start=${start}&end=${end}`)

    // Download the manifest as JSON
    const blob = new Blob([JSON.stringify(response, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `export_${cameraId.value.slice(0, 8)}_${startTime.value.replace(/[:.]/g, '')}.json`
    a.click()
    URL.revokeObjectURL(url)

    success(`Export ready: ${response.segment_count} segments`)
    emit('close')
  } catch (e) {
    showError('Export failed')
  } finally {
    exporting.value = false
  }
}
</script>
