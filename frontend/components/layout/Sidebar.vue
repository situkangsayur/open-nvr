<template>
  <aside class="fixed left-0 top-0 h-full w-64 bg-white dark:bg-nvr-dark border-r border-gray-200 dark:border-nvr-border flex flex-col z-40">
    <div class="p-4 border-b border-gray-200 dark:border-nvr-border">
      <h1 class="text-xl font-bold text-primary-400">Open-NVR</h1>
      <p class="text-xs text-gray-400 dark:text-gray-500">Network Video Recorder</p>
    </div>
    <nav class="flex-1 p-4 space-y-1">
      <SidebarLink to="/" label="Dashboard" />
      <SidebarLink to="/cameras" label="Cameras" />
      <SidebarLink to="/live" label="Live View" />
      <SidebarLink to="/playback" label="Playback" />
      <SidebarLink to="/events" label="Events" />
      <SidebarLink to="/settings" label="Settings" />
    </nav>
    <div class="p-4 border-t border-gray-200 dark:border-nvr-border">
      <div class="flex items-center justify-between">
        <LayoutThemeToggle />
        <button @click="logout" class="text-sm text-gray-400 hover:text-white dark:hover:text-white transition-colors">Sign out</button>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
const { logout } = useAuth()
</script>
