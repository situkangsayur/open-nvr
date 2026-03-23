<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">Dashboard</h1>
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
      <DashboardStatCard title="Total Cameras" :value="stats.totalCameras" icon="📹" />
      <DashboardStatCard title="Online" :value="stats.onlineCameras" icon="🟢" />
      <DashboardStatCard title="Offline" :value="stats.offlineCameras" icon="🔴" />
      <DashboardStatCard title="Alerts" :value="stats.unresolvedAlerts" icon="⚠️" />
    </div>
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div class="bg-nvr-card rounded-lg p-4 border border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Recent Audit Logs</h2>
        <div v-if="auditLogs.length === 0" class="text-gray-400 text-sm">No recent activity</div>
        <div v-for="log in auditLogs" :key="log.id" class="flex justify-between items-center py-2 border-b border-nvr-border last:border-0">
          <div>
            <span class="text-sm font-medium">{{ log.action }}</span>
            <span class="text-xs text-gray-400 ml-2">{{ log.resource_type }}</span>
          </div>
          <span class="text-xs text-gray-500">{{ formatDate(log.created_at) }}</span>
        </div>
      </div>
      <div class="bg-nvr-card rounded-lg p-4 border border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Network Security Events</h2>
        <div v-if="networkEvents.length === 0" class="text-gray-400 text-sm">No unresolved events</div>
        <div v-for="event in networkEvents" :key="event.id" class="flex justify-between items-center py-2 border-b border-nvr-border last:border-0">
          <div>
            <span :class="severityClass(event.severity)" class="text-xs font-medium px-2 py-0.5 rounded">{{ event.severity }}</span>
            <span class="text-sm ml-2">{{ event.event_type }}</span>
          </div>
          <span class="text-xs text-gray-500">{{ formatDate(event.created_at) }}</span>
        </div>
      </div>
    </div>

    <!-- System Info -->
    <div class="bg-nvr-card rounded-lg p-4 border border-nvr-border mt-6">
      <h2 class="text-lg font-semibold mb-4">System Information</h2>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
        <div>
          <span class="text-gray-400">Keycloak</span>
          <p class="text-primary-400">{{ config.public.keycloakUrl }}</p>
        </div>
        <div>
          <span class="text-gray-400">API</span>
          <p class="text-primary-400">{{ config.public.apiUrl }}</p>
        </div>
        <div>
          <span class="text-gray-400">Cameras</span>
          <p>{{ stats.totalCameras }} configured</p>
        </div>
        <div>
          <span class="text-gray-400">Alerts</span>
          <p :class="stats.unresolvedAlerts > 0 ? 'text-red-400' : 'text-green-400'">
            {{ stats.unresolvedAlerts }} unresolved
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const { $api } = useNuxtApp()
const config = useRuntimeConfig()
const stats = ref({ totalCameras: 0, onlineCameras: 0, offlineCameras: 0, unresolvedAlerts: 0 })
const auditLogs = ref<any[]>([])
const networkEvents = ref<any[]>([])

const formatDate = (date: string) => new Date(date).toLocaleString()

const severityClass = (severity: string) => ({
  'bg-blue-500/20 text-blue-400': severity === 'info',
  'bg-yellow-500/20 text-yellow-400': severity === 'warning',
  'bg-red-500/20 text-red-400': severity === 'critical',
})

onMounted(async () => {
  try {
    const cameras = await useApi<any[]>('/api/cameras')
    stats.value.totalCameras = cameras.length
    stats.value.onlineCameras = cameras.filter((c: any) => c.status === 'online').length
    stats.value.offlineCameras = cameras.filter((c: any) => c.status === 'offline').length
  } catch (e) {
    console.error('Failed to load dashboard data', e)
  }
  try {
    auditLogs.value = await useApi<any[]>('/api/audit/logs?limit=10')
  } catch (e) { /* ignore */ }
  try {
    networkEvents.value = await useApi<any[]>('/api/audit/network-events')
    stats.value.unresolvedAlerts = networkEvents.value.length
  } catch (e) { /* ignore */ }
})
</script>
