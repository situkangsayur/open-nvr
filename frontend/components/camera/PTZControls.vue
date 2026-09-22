<template>
  <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border">
    <h3 class="text-sm font-semibold text-gray-400 mb-3">PTZ Control</h3>
    <!-- Same touch-capable pad as the live view (press and hold to move). -->
    <PtzPanel :camera-id="cameraId" />
  </div>
</template>

<script setup lang="ts">
defineProps<{ cameraId: string }>()
</script>
