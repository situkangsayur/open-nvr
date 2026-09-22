<template>
  <div class="min-h-screen bg-gray-50 text-gray-900 dark:bg-nvr-darker dark:text-gray-100 flex">
    <LayoutSidebar />
    <main class="flex-1 transition-all" :class="sidebarCollapsed ? 'lg:ml-16' : 'lg:ml-64'">
      <!-- Extra top padding below lg keeps content clear of the fixed menu button -->
      <div class="px-3 pb-4 pt-16 sm:px-6 sm:pb-6 lg:pt-6">
        <slot />
      </div>
    </main>
    <LayoutToastContainer />
  </div>
</template>

<script setup lang="ts">
const sidebarCollapsed = ref(false)
provide('sidebarCollapsed', sidebarCollapsed)
</script>
