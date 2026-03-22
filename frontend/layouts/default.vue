<template>
  <div class="min-h-screen bg-nvr-darker text-gray-100 flex">
    <LayoutSidebar />
    <main class="flex-1 ml-64">
      <div class="p-6">
        <slot />
      </div>
    </main>
  </div>
</template>
