<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/settings" class="text-gray-400 hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Users & Security</h1>
    </div>
    <div class="bg-nvr-card rounded-lg p-6 border border-nvr-border">
      <p class="text-gray-400">User management is handled via <a :href="keycloakUrl" target="_blank" class="text-primary-400 hover:underline">Keycloak Admin Console</a>.</p>
      <p class="text-sm text-gray-500 mt-2">Roles: admin, operator, viewer</p>
    </div>
  </div>
</template>

<script setup lang="ts">
const config = useRuntimeConfig()
const keycloakUrl = `${config.public.keycloakUrl}/admin/${config.public.keycloakRealm}/console`
</script>
