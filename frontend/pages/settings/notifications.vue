<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/settings" class="text-gray-400 hover:text-white dark:hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Notifications & Alerts</h1>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Alert Rules -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Alert Rules</h2>

        <div class="space-y-3 mb-6">
          <div v-for="rule in alertRules" :key="rule.id" class="flex items-center justify-between p-3 bg-gray-50 dark:bg-nvr-darker rounded-lg">
            <div>
              <div class="text-sm font-medium">{{ rule.name }}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400">
                {{ rule.event_type }} &middot; Severity: {{ rule.min_severity }}
              </div>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" v-model="rule.enabled" class="sr-only peer" @change="toggleRule(rule)" />
              <div class="w-9 h-5 bg-gray-300 dark:bg-gray-600 peer-checked:bg-primary-600 rounded-full transition-colors after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-4"></div>
            </label>
          </div>
        </div>

        <div v-if="alertRules.length === 0" class="text-gray-400 text-sm mb-4">No alert rules configured.</div>

        <h3 class="text-sm font-semibold text-gray-500 dark:text-gray-400 mb-2">Add Rule</h3>
        <div class="space-y-3">
          <input v-model="newRule.name" type="text" placeholder="Rule name" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white text-sm" />
          <select v-model="newRule.event_type" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white text-sm">
            <option value="motion">Motion Detection</option>
            <option value="human">Human Detection</option>
            <option value="vehicle">Vehicle Detection</option>
            <option value="unauthorized_access">Unauthorized Access</option>
            <option value="camera_offline">Camera Offline</option>
          </select>
          <select v-model="newRule.min_severity" class="w-full bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white text-sm">
            <option value="info">Info</option>
            <option value="warning">Warning</option>
            <option value="critical">Critical</option>
          </select>
          <button @click="addRule" :disabled="!newRule.name.trim()" class="bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white px-4 py-2 rounded text-sm transition-colors">
            Add Rule
          </button>
        </div>
      </div>

      <!-- Notification Channels -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Notification Channels</h2>

        <div class="space-y-4">
          <div class="p-4 bg-gray-50 dark:bg-nvr-darker rounded-lg">
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-sm font-medium">Browser Notifications</h3>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="channels.browser" class="sr-only peer" />
                <div class="w-9 h-5 bg-gray-300 dark:bg-gray-600 peer-checked:bg-primary-600 rounded-full transition-colors after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-4"></div>
              </label>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400">Show desktop notifications for alerts</p>
          </div>

          <div class="p-4 bg-gray-50 dark:bg-nvr-darker rounded-lg">
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-sm font-medium">Sound Alert</h3>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="channels.sound" class="sr-only peer" />
                <div class="w-9 h-5 bg-gray-300 dark:bg-gray-600 peer-checked:bg-primary-600 rounded-full transition-colors after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-4"></div>
              </label>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400">Play sound on critical events</p>
          </div>

          <div class="p-4 bg-gray-50 dark:bg-nvr-darker rounded-lg">
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-sm font-medium">NATS Events (System)</h3>
              <span class="text-xs text-green-400">Active</span>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400">Events published to NATS message broker for external integrations</p>
          </div>
        </div>

        <div class="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg text-xs text-blue-600 dark:text-blue-400">
          Configure webhook or email notifications by connecting to NATS subjects:
          <code class="block mt-1 text-xs">opennvr.detection.*, opennvr.network.security, opennvr.camera.*.status</code>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const alertRules = ref<any[]>([
  { id: '1', name: 'Motion on all cameras', event_type: 'motion', min_severity: 'info', enabled: true },
  { id: '2', name: 'Camera offline alert', event_type: 'camera_offline', min_severity: 'critical', enabled: true },
  { id: '3', name: 'Unauthorized access', event_type: 'unauthorized_access', min_severity: 'warning', enabled: true },
])

const channels = ref({
  browser: false,
  sound: false,
})

const newRule = ref({ name: '', event_type: 'motion', min_severity: 'warning' })

const addRule = () => {
  if (!newRule.value.name.trim()) return
  alertRules.value.push({
    id: Date.now().toString(),
    ...newRule.value,
    enabled: true,
  })
  newRule.value = { name: '', event_type: 'motion', min_severity: 'warning' }
}

const toggleRule = (rule: any) => {
  // Would persist to backend in real implementation
}
</script>
