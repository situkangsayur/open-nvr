<template>
  <NuxtLink :to="to" :class="[
    'block px-3 py-2 rounded-lg text-sm font-medium transition-colors',
    isActive ? 'bg-primary-600/20 text-primary-600 dark:text-primary-400' : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white dark:bg-nvr-card',
  ]">
    {{ label }}
  </NuxtLink>
</template>

<script setup lang="ts">
const props = defineProps<{ to: string; label: string }>()
const route = useRoute()
const isActive = computed(() => {
  if (props.to === '/') return route.path === '/'
  return route.path.startsWith(props.to)
})
</script>
