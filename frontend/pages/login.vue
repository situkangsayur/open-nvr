<template>
  <div class="text-center">
    <div class="bg-nvr-card rounded-lg p-8 border border-nvr-border max-w-md mx-auto">
      <h1 class="text-3xl font-bold mb-2">Open-NVR</h1>
      <p class="text-gray-400 mb-6">Network Video Recorder</p>
      <button
        @click="login"
        class="w-full bg-primary-600 hover:bg-primary-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
      >
        Sign in with Keycloak
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'auth' })

const { login } = useAuth()
</script>
