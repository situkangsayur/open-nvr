<template>
  <div class="bg-white dark:bg-nvr-card rounded-lg p-4 border border-gray-200 dark:border-nvr-border">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm text-gray-500 dark:text-gray-400">{{ title }}</p>
        <p class="text-2xl font-bold mt-1">{{ value }}</p>
      </div>
      <span class="text-2xl">{{ icon }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ title: string; value: number; icon: string }>()
</script>
