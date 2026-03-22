<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">Settings</h1>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <NuxtLink to="/settings/storage" class="bg-nvr-card rounded-lg p-6 border border-nvr-border hover:border-primary-500 transition-colors">
        <h2 class="text-lg font-semibold mb-2">Storage</h2>
        <p class="text-sm text-gray-400">Manage storage, retention policies, and estimate capacity</p>
      </NuxtLink>
      <NuxtLink to="/settings/users" class="bg-nvr-card rounded-lg p-6 border border-nvr-border hover:border-primary-500 transition-colors">
        <h2 class="text-lg font-semibold mb-2">Users & Security</h2>
        <p class="text-sm text-gray-400">Manage users, roles, and audit trail</p>
      </NuxtLink>
    </div>
  </div>
</template>
