<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-nvr-darker">
    <p class="text-gray-400">Authenticating...</p>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'auth' })

const { handleCallback } = useAuth()
onMounted(() => handleCallback())
</script>
