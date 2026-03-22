<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/settings" class="text-gray-400 hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Storage Calculator</h1>
    </div>
    <div class="bg-nvr-card rounded-lg p-6 border border-nvr-border max-w-lg">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-1">Number of Cameras</label>
          <input v-model.number="form.cameras" type="number" min="1" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-1">Bitrate (Mbps)</label>
          <input v-model.number="form.bitrate_mbps" type="number" min="0.5" step="0.5" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-1">Retention (days)</label>
          <input v-model.number="form.retention_days" type="number" min="1" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-1">Motion Ratio (0.0 = motion-only, 1.0 = continuous)</label>
          <input v-model.number="form.motion_ratio" type="number" min="0" max="1" step="0.1" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
        </div>
        <button @click="calculate" class="bg-primary-600 hover:bg-primary-700 text-white px-6 py-2 rounded-lg transition-colors">
          Calculate
        </button>
      </div>
      <div v-if="result" class="mt-6 p-4 bg-nvr-darker rounded-lg">
        <div class="text-2xl font-bold text-primary-400">{{ result.formatted }}</div>
        <div class="text-sm text-gray-400 mt-1">
          {{ result.cameras }} cameras &times; {{ result.bitrate_mbps }} Mbps &times; {{ result.retention_days }} days
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const form = ref({ cameras: 4, bitrate_mbps: 4, retention_days: 30, motion_ratio: 1.0 })
const result = ref<any>(null)

const calculate = async () => {
  try {
    result.value = await useApi('/api/settings/storage/estimate', { method: 'POST', body: form.value })
  } catch (e) {
    console.error('Calculation failed', e)
  }
}
</script>
