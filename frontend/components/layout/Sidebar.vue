<template>
  <div>
    <!-- Mobile hamburger -->
    <button @click="mobileOpen = !mobileOpen" class="fixed top-4 left-4 z-50 lg:hidden bg-white dark:bg-nvr-dark p-2 rounded-lg border border-gray-200 dark:border-nvr-border shadow-md">
      <span class="block w-5 h-0.5 bg-gray-600 dark:bg-gray-300 mb-1"></span>
      <span class="block w-5 h-0.5 bg-gray-600 dark:bg-gray-300 mb-1"></span>
      <span class="block w-5 h-0.5 bg-gray-600 dark:bg-gray-300"></span>
    </button>

    <!-- Mobile overlay -->
    <div v-if="mobileOpen" @click="mobileOpen = false" class="fixed inset-0 bg-black/50 z-30 lg:hidden"></div>

    <!-- Sidebar -->
    <aside class="fixed left-0 top-0 h-full w-64 bg-white dark:bg-nvr-dark border-r border-gray-200 dark:border-nvr-border flex flex-col z-40 transform transition-transform lg:translate-x-0" :class="mobileOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'">
      <div class="p-4 border-b border-gray-200 dark:border-nvr-border">
        <h1 class="text-xl font-bold text-primary-600 dark:text-primary-400">Open-NVR</h1>
        <p class="text-xs text-gray-400 dark:text-gray-500">Network Video Recorder</p>
      </div>
      <nav class="flex-1 p-4 space-y-1">
        <NuxtLink to="/" :class="linkClass('/')">Dashboard</NuxtLink>
        <NuxtLink to="/cameras" :class="linkClass('/cameras')">Cameras</NuxtLink>
        <NuxtLink to="/cameras/discover" :class="linkClass('/cameras/discover')">Discover / Scanner</NuxtLink>
        <NuxtLink to="/cameras/guide" :class="linkClass('/cameras/guide')">Setup Guide</NuxtLink>
        <NuxtLink to="/live" :class="linkClass('/live')">Live View</NuxtLink>
        <NuxtLink to="/playback" :class="linkClass('/playback')">Playback</NuxtLink>
        <NuxtLink to="/events" :class="linkClass('/events')">Events</NuxtLink>
        <NuxtLink to="/settings" :class="linkClass('/settings')">Settings</NuxtLink>
      </nav>
      <div class="p-4 border-t border-gray-200 dark:border-nvr-border">
        <div class="flex items-center justify-between">
          <button @click="toggleTheme" class="p-2 rounded-lg transition-colors hover:bg-gray-200 dark:hover:bg-nvr-card text-sm">
            {{ isDark ? '☀️ Light' : '🌙 Dark' }}
          </button>
          <button @click="doLogout" class="text-sm text-gray-500 dark:text-gray-400 hover:text-red-500 transition-colors">Sign out</button>
        </div>
      </div>
    </aside>
  </div>
</template>

<script setup lang="ts">
const { logout } = useAuth()
const route = useRoute()
const mobileOpen = ref(false)
const isDark = ref(true)

onMounted(() => {
  isDark.value = document.documentElement.classList.contains('dark')
  // Default to dark if no preference
  if (!localStorage.getItem('opennvr-theme')) {
    document.documentElement.classList.add('dark')
    isDark.value = true
  }
})

const toggleTheme = () => {
  isDark.value = !isDark.value
  if (isDark.value) {
    document.documentElement.classList.add('dark')
    localStorage.setItem('opennvr-theme', 'dark')
  } else {
    document.documentElement.classList.remove('dark')
    localStorage.setItem('opennvr-theme', 'light')
  }
}

const linkClass = (path: string) => {
  const active = path === '/' ? route.path === '/' : route.path.startsWith(path)
  return [
    'block px-3 py-2 rounded-lg text-sm font-medium transition-colors',
    active
      ? 'bg-primary-100 dark:bg-primary-600/20 text-primary-700 dark:text-primary-400'
      : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-white dark:hover:bg-nvr-card',
  ]
}

const doLogout = () => logout()
</script>
