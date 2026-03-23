<template>
  <div class="min-h-screen bg-gray-50 text-gray-900 dark:bg-nvr-darker dark:text-gray-100 flex">
    <LayoutSidebar />
    <main class="flex-1 ml-64">
      <div class="p-6">
        <slot />
      </div>
    </main>
    <LayoutToastContainer />
  </div>
</template>
