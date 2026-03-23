<template>
  <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border">
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm font-semibold text-gray-400">Layout</h3>
      <button @click="showSave = !showSave" class="text-xs text-primary-400 hover:underline">
        {{ showSave ? 'Cancel' : 'Save Layout' }}
      </button>
    </div>

    <!-- Preset layouts -->
    <div class="flex gap-2 mb-3">
      <button v-for="preset in presets" :key="preset.cols"
        @click="$emit('change', preset.cols)"
        :class="currentCols === preset.cols ? 'bg-primary-600 border-primary-500' : 'bg-gray-100 dark:bg-nvr-darker border-gray-200 dark:border-nvr-border'"
        class="flex-1 py-2 rounded text-xs border transition-colors text-center">
        {{ preset.label }}
      </button>
    </div>

    <!-- Saved layouts -->
    <div v-if="layouts.length > 0" class="space-y-1 mb-3">
      <div v-for="layout in layouts" :key="layout.id"
        @click="$emit('load', layout)"
        class="flex justify-between items-center p-2 rounded cursor-pointer hover:bg-gray-100 dark:bg-nvr-darker text-xs">
        <span>{{ layout.name }}</span>
        <button @click.stop="deleteLayout(layout.id)" class="text-red-400 hover:underline">x</button>
      </div>
    </div>

    <!-- Save form -->
    <div v-if="showSave" class="space-y-2">
      <input v-model="saveName" type="text" placeholder="Layout name" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-1.5 text-white text-xs" />
      <button @click="saveLayout" :disabled="!saveName.trim()" class="w-full bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white py-1.5 rounded text-xs transition-colors">
        Save
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ currentCols: number }>()
const emit = defineEmits<{ change: [cols: number]; load: [layout: any] }>()

const presets = [
  { cols: 1, label: '1x1' },
  { cols: 2, label: '2x2' },
  { cols: 3, label: '3x3' },
  { cols: 4, label: '4x4' },
]

const layouts = ref<any[]>([])
const showSave = ref(false)
const saveName = ref('')

const loadLayouts = async () => {
  try {
    layouts.value = await useApi<any[]>('/api/layouts')
  } catch {}
}

const saveLayout = async () => {
  if (!saveName.value.trim()) return
  try {
    await useApi('/api/layouts', {
      method: 'POST',
      body: {
        name: saveName.value,
        layout_type: 'grid',
        camera_positions: [],
        is_default: false,
      },
    })
    saveName.value = ''
    showSave.value = false
    await loadLayouts()
  } catch {}
}

const deleteLayout = async (id: string) => {
  try {
    await useApi(`/api/layouts/${id}`, { method: 'DELETE' })
    await loadLayouts()
  } catch {}
}

onMounted(loadLayouts)
</script>
