<template>
  <NuxtLink :to="to" :class="[
    'block px-3 py-2 rounded-lg text-sm font-medium transition-colors',
    isActive ? 'bg-primary-600/20 text-primary-400' : 'text-gray-400 hover:text-white hover:bg-nvr-card',
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
