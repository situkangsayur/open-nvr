<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/settings" class="text-gray-400 hover:text-gray-700 dark:hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">System Capacity</h1>
    </div>

    <div v-if="loading" class="text-gray-400">Loading system info...</div>
    <div v-else-if="data" class="space-y-6">
      <!-- System Info -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Server Specifications</h2>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">Hostname</span>
            <span class="font-medium">{{ data.system.hostname }}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">OS</span>
            <span class="font-medium text-xs">{{ data.system.os || data.system.kernel }}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">CPU</span>
            <span class="font-medium text-xs">{{ data.system.cpu_model }}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">Cores / Threads</span>
            <span class="font-medium">{{ data.system.cpu_cores }} / {{ data.system.cpu_threads }}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">RAM Total</span>
            <span class="font-medium">{{ data.system.ram_total_gb }} GB</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">RAM Available</span>
            <span class="font-medium" :class="data.system.ram_available_gb < 2 ? 'text-red-500' : 'text-green-500'">{{ data.system.ram_available_gb }} GB</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">Disk Total</span>
            <span class="font-medium">{{ data.system.disk_total_gb }} GB</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">Disk Available</span>
            <span class="font-medium" :class="data.system.disk_available_gb < 50 ? 'text-red-500' : 'text-green-500'">{{ data.system.disk_available_gb }} GB</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">GPU</span>
            <span class="font-medium">{{ data.system.gpu || 'None' }}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">Load Average</span>
            <span class="font-medium">{{ data.system.load_avg.map((v: number) => v.toFixed(1)).join(' / ') }}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">Uptime</span>
            <span class="font-medium">{{ Math.floor(data.system.uptime_hours / 24) }}d {{ Math.floor(data.system.uptime_hours % 24) }}h</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400 block">ffmpeg</span>
            <span class="font-medium" :class="data.system.ffmpeg_available ? 'text-green-500' : 'text-red-500'">{{ data.system.ffmpeg_available ? 'Installed' : 'Not installed' }}</span>
          </div>
        </div>

        <!-- Usage bars -->
        <div class="mt-6 space-y-3">
          <div>
            <div class="flex justify-between text-xs mb-1">
              <span class="text-gray-500 dark:text-gray-400">RAM Usage</span>
              <span>{{ data.system.ram_used_percent }}%</span>
            </div>
            <div class="w-full bg-gray-200 dark:bg-nvr-darker rounded-full h-2.5">
              <div class="h-2.5 rounded-full" :class="data.system.ram_used_percent > 90 ? 'bg-red-500' : data.system.ram_used_percent > 70 ? 'bg-yellow-500' : 'bg-green-500'" :style="`width: ${data.system.ram_used_percent}%`"></div>
            </div>
          </div>
          <div>
            <div class="flex justify-between text-xs mb-1">
              <span class="text-gray-500 dark:text-gray-400">Disk Usage</span>
              <span>{{ data.system.disk_used_percent }}%</span>
            </div>
            <div class="w-full bg-gray-200 dark:bg-nvr-darker rounded-full h-2.5">
              <div class="h-2.5 rounded-full" :class="data.system.disk_used_percent > 90 ? 'bg-red-500' : data.system.disk_used_percent > 70 ? 'bg-yellow-500' : 'bg-green-500'" :style="`width: ${data.system.disk_used_percent}%`"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- Camera Capacity -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-semibold mb-4">Camera Capacity Estimate</h2>
        <div class="grid grid-cols-2 md:grid-cols-3 gap-6">
          <div class="text-center p-4 bg-gray-50 dark:bg-nvr-darker rounded-lg">
            <div class="text-3xl font-bold text-primary-600 dark:text-primary-400">{{ data.capacity.max_cameras_1080p_continuous }}</div>
            <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">1080p Continuous</div>
          </div>
          <div class="text-center p-4 bg-gray-50 dark:bg-nvr-darker rounded-lg">
            <div class="text-3xl font-bold text-primary-600 dark:text-primary-400">{{ data.capacity.max_cameras_720p_continuous }}</div>
            <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">720p Continuous</div>
          </div>
          <div class="text-center p-4 bg-gray-50 dark:bg-nvr-darker rounded-lg">
            <div class="text-3xl font-bold text-green-500">{{ data.capacity.max_cameras_1080p_motion }}</div>
            <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">1080p Motion-Only</div>
          </div>
        </div>

        <div class="mt-6 grid grid-cols-2 gap-4 text-sm">
          <div class="flex justify-between p-2 bg-gray-50 dark:bg-nvr-darker rounded">
            <span class="text-gray-500 dark:text-gray-400">CPU per camera</span>
            <span>~{{ data.capacity.cpu_per_camera_percent }}%</span>
          </div>
          <div class="flex justify-between p-2 bg-gray-50 dark:bg-nvr-darker rounded">
            <span class="text-gray-500 dark:text-gray-400">RAM per camera</span>
            <span>~{{ data.capacity.ram_per_camera_mb }} MB</span>
          </div>
          <div class="flex justify-between p-2 bg-gray-50 dark:bg-nvr-darker rounded">
            <span class="text-gray-500 dark:text-gray-400">Storage per camera/day</span>
            <span>~{{ data.capacity.disk_per_camera_per_day_gb }} GB</span>
          </div>
          <div class="flex justify-between p-2 bg-gray-50 dark:bg-nvr-darker rounded">
            <span class="text-gray-500 dark:text-gray-400">Bandwidth per camera</span>
            <span>~{{ data.capacity.network_bandwidth_per_camera_mbps }} Mbps</span>
          </div>
          <div class="flex justify-between p-2 bg-gray-50 dark:bg-nvr-darker rounded">
            <span class="text-gray-500 dark:text-gray-400">Recording days (1080p max load)</span>
            <span>~{{ data.capacity.estimated_recording_days_1080p }} days</span>
          </div>
          <div class="flex justify-between p-2 bg-gray-50 dark:bg-nvr-darker rounded">
            <span class="text-gray-500 dark:text-gray-400">Recording days (720p max load)</span>
            <span>~{{ data.capacity.estimated_recording_days_720p }} days</span>
          </div>
        </div>
      </div>

      <!-- Recommendations -->
      <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-6 border border-blue-200 dark:border-blue-800">
        <h2 class="text-lg font-semibold mb-3 text-blue-700 dark:text-blue-400">Recommendations</h2>
        <ul class="space-y-2">
          <li v-for="(rec, i) in data.recommendations" :key="i" class="flex gap-2 text-sm text-blue-600 dark:text-blue-300">
            <span class="flex-shrink-0">&#8226;</span>
            <span>{{ rec }}</span>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const data = ref<any>(null)
const loading = ref(true)

onMounted(async () => {
  try {
    data.value = await useApi<any>('/api/system/capacity')
  } catch (e) {
    console.error('Failed to load capacity', e)
  }
  loading.value = false
})
</script>
