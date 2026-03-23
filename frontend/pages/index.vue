<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">Dashboard</h1>
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
      <DashboardStatCard title="Total Cameras" :value="stats.totalCameras" icon="📹" />
      <DashboardStatCard title="Online" :value="stats.onlineCameras" icon="🟢" />
      <DashboardStatCard title="Offline" :value="stats.offlineCameras" icon="🔴" />
      <DashboardStatCard title="Alerts" :value="stats.unresolvedAlerts" icon="⚠️" />
    </div>
    <!-- Network Warning -->
    <div v-if="stats.totalCameras > 0 && stats.onlineCameras === 0" class="mb-6 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
      <h3 class="text-sm font-semibold text-yellow-700 dark:text-yellow-400 mb-1">All cameras offline</h3>
      <p class="text-xs text-yellow-600 dark:text-yellow-500">
        Server cannot reach any cameras. Possible causes:
      </p>
      <ul class="text-xs text-yellow-600 dark:text-yellow-500 list-disc list-inside mt-1">
        <li>WiFi AP Isolation is blocking device-to-device communication</li>
        <li>Cameras are on a different subnet/VLAN</li>
        <li>Camera RTSP is not enabled (check V360 Pro / XMEye app settings)</li>
        <li>Server ethernet cable not connected (recommended for NVR)</li>
      </ul>
      <div class="mt-2">
        <NuxtLink to="/cameras/guide" class="text-xs text-yellow-700 dark:text-yellow-400 underline">View Setup Guide</NuxtLink>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Recent Audit Logs</h2>
        <div v-if="auditLogs.length === 0" class="text-gray-400 text-sm">No recent activity</div>
        <div v-for="log in auditLogs" :key="log.id" class="flex justify-between items-center py-2 border-b border-gray-100 dark:border-nvr-border last:border-0">
          <div>
            <span class="text-sm font-medium">{{ log.action }}</span>
            <span class="text-xs text-gray-400 ml-2">{{ log.resource_type }}</span>
          </div>
          <span class="text-xs text-gray-500">{{ formatDate(log.created_at) }}</span>
        </div>
      </div>
      <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Network Security Events</h2>
        <div v-if="networkEvents.length === 0" class="text-gray-400 text-sm">No unresolved events</div>
        <div v-for="event in networkEvents" :key="event.id" class="flex justify-between items-center py-2 border-b border-gray-100 dark:border-nvr-border last:border-0">
          <div>
            <span :class="severityClass(event.severity)" class="text-xs font-medium px-2 py-0.5 rounded">{{ event.severity }}</span>
            <span class="text-sm ml-2">{{ event.event_type }}</span>
          </div>
          <span class="text-xs text-gray-500">{{ formatDate(event.created_at) }}</span>
        </div>
      </div>
    </div>

    <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border mt-6">
      <h2 class="text-lg font-semibold mb-4">System Information</h2>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
        <div>
          <span class="text-gray-500 dark:text-gray-400">Keycloak</span>
          <p class="text-primary-600 dark:text-primary-400 text-xs break-all">{{ config.public.keycloakUrl }}</p>
        </div>
        <div>
          <span class="text-gray-500 dark:text-gray-400">API</span>
          <p class="text-primary-600 dark:text-primary-400 text-xs break-all">{{ config.public.apiUrl }}</p>
        </div>
        <div>
          <span class="text-gray-500 dark:text-gray-400">Cameras</span>
          <p>{{ stats.totalCameras }} configured</p>
        </div>
        <div>
          <span class="text-gray-500 dark:text-gray-400">Alerts</span>
          <p :class="stats.unresolvedAlerts > 0 ? 'text-red-500' : 'text-green-500'">
            {{ stats.unresolvedAlerts }} unresolved
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const config = useRuntimeConfig()
const stats = ref({ totalCameras: 0, onlineCameras: 0, offlineCameras: 0, unresolvedAlerts: 0 })
const auditLogs = ref<any[]>([])
const networkEvents = ref<any[]>([])

const formatDate = (date: string) => new Date(date).toLocaleString()

const severityClass = (severity: string) => ({
  'bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400': severity === 'info',
  'bg-yellow-100 dark:bg-yellow-500/20 text-yellow-600 dark:text-yellow-400': severity === 'warning',
  'bg-red-100 dark:bg-red-500/20 text-red-600 dark:text-red-400': severity === 'critical',
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
  } catch {}
  try {
    networkEvents.value = await useApi<any[]>('/api/audit/network-events')
    stats.value.unresolvedAlerts = networkEvents.value.length
  } catch {}
})
</script>
