<template>
  <button @click="themeStore.toggle()" class="p-2 rounded-lg transition-colors hover:bg-gray-200 dark:hover:bg-nvr-card" :title="themeStore.mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'">
    <span v-if="themeStore.mode === 'dark'" class="text-yellow-400 text-sm">SUN</span>
    <span v-else class="text-gray-600 text-sm">MOON</span>
  </button>
</template>

<script setup lang="ts">
const themeStore = useThemeStore()
</script>
