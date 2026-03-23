<template>
  <div class="fixed top-4 right-4 z-50 space-y-2">
    <TransitionGroup name="toast">
      <div v-for="toast in toasts" :key="toast.id"
        :class="toastClass(toast.type)"
        class="px-4 py-3 rounded-lg shadow-lg text-sm max-w-sm cursor-pointer border"
        @click="remove(toast.id)">
        {{ toast.message }}
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
const { toasts, remove } = useToast()

const toastClass = (type: string) => ({
  'bg-green-900/90 border-green-700 text-green-200': type === 'success',
  'bg-red-900/90 border-red-700 text-red-200': type === 'error',
  'bg-blue-900/90 border-blue-700 text-blue-200': type === 'info',
  'bg-yellow-900/90 border-yellow-700 text-yellow-200': type === 'warning',
})
</script>

<style scoped>
.toast-enter-active { transition: all 0.3s ease-out; }
.toast-leave-active { transition: all 0.2s ease-in; }
.toast-enter-from { transform: translateX(100%); opacity: 0; }
.toast-leave-to { transform: translateX(100%); opacity: 0; }
</style>
