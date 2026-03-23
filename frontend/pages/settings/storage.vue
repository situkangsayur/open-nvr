<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/settings" class="text-gray-400 hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Storage & Retention</h1>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Storage Calculator -->
      <div class="bg-nvr-card rounded-lg p-6 border border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Storage Calculator</h2>
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
            <label class="block text-sm font-medium text-gray-300 mb-1">Motion Ratio (0.0-1.0)</label>
            <input v-model.number="form.motion_ratio" type="number" min="0" max="1" step="0.1" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white" />
          </div>
          <button @click="calculate" class="bg-primary-600 hover:bg-primary-700 text-white px-6 py-2 rounded-lg transition-colors">Calculate</button>
        </div>
        <div v-if="result" class="mt-6 p-4 bg-nvr-darker rounded-lg">
          <div class="text-2xl font-bold text-primary-400">{{ result.formatted }}</div>
          <div class="text-sm text-gray-400 mt-1">
            {{ result.cameras }} cameras x {{ result.bitrate_mbps }} Mbps x {{ result.retention_days }} days
          </div>
        </div>
      </div>

      <!-- Retention Policies -->
      <div class="bg-nvr-card rounded-lg p-6 border border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Retention Policies</h2>
        <div v-if="policies.length === 0" class="text-gray-400 text-sm mb-4">No retention policies configured.</div>
        <div v-else class="space-y-2 mb-4">
          <div v-for="policy in policies" :key="policy.id" class="flex justify-between items-center p-3 bg-nvr-darker rounded">
            <div>
              <div class="text-sm font-medium">{{ policy.name }}</div>
              <div class="text-xs text-gray-400">{{ policy.retention_days }} days &middot; {{ policy.camera_id ? 'Per camera' : 'Global' }}</div>
            </div>
            <button @click="deletePolicy(policy.id)" class="text-xs text-red-400 hover:underline">Delete</button>
          </div>
        </div>
        <h3 class="text-sm font-semibold text-gray-400 mb-2">Add Policy</h3>
        <div class="space-y-3">
          <input v-model="newPolicy.name" type="text" placeholder="Policy name" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white text-sm" />
          <input v-model.number="newPolicy.retention_days" type="number" min="1" placeholder="Retention days" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white text-sm" />
          <button @click="createPolicy" :disabled="!newPolicy.name.trim()" class="bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white px-4 py-2 rounded text-sm transition-colors">
            Add Policy
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const form = ref({ cameras: 4, bitrate_mbps: 4, retention_days: 30, motion_ratio: 1.0 })
const result = ref<any>(null)
const policies = ref<any[]>([])
const newPolicy = ref({ name: '', retention_days: 30 })

const calculate = async () => {
  try {
    result.value = await useApi('/api/settings/storage/estimate', { method: 'POST', body: form.value })
  } catch {}
}

const loadPolicies = async () => {
  try {
    policies.value = await useApi<any[]>('/api/settings/retention')
  } catch {}
}

const createPolicy = async () => {
  if (!newPolicy.value.name.trim()) return
  try {
    await useApi('/api/settings/retention', {
      method: 'POST',
      body: newPolicy.value,
    })
    newPolicy.value = { name: '', retention_days: 30 }
    await loadPolicies()
  } catch {}
}

const deletePolicy = async (id: string) => {
  if (!confirm('Delete this policy?')) return
  try {
    await useApi(`/api/settings/retention/${id}`, { method: 'DELETE' })
    await loadPolicies()
  } catch {}
}

onMounted(loadPolicies)
</script>
