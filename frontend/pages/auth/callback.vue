<template>
  <div class="min-h-screen flex items-center justify-center bg-nvr-darker">
    <p class="text-gray-400">Authenticating...</p>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'auth' })

const { handleCallback } = useAuth()
onMounted(() => handleCallback())
</script>
