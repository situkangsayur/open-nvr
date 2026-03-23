<template>
  <div class="bg-nvr-card rounded-lg p-4 border border-nvr-border">
    <h3 class="text-sm font-semibold text-gray-400 mb-3">PTZ Control</h3>
    <div class="grid grid-cols-3 gap-1 w-36 mx-auto mb-3">
      <div></div>
      <button @mousedown="sendPtz('tilt_up')" @mouseup="sendPtz('stop')" class="ptz-btn">&#9650;</button>
      <div></div>
      <button @mousedown="sendPtz('pan_left')" @mouseup="sendPtz('stop')" class="ptz-btn">&#9664;</button>
      <button @click="sendPtz('home')" class="ptz-btn text-xs">H</button>
      <button @mousedown="sendPtz('pan_right')" @mouseup="sendPtz('stop')" class="ptz-btn">&#9654;</button>
      <div></div>
      <button @mousedown="sendPtz('tilt_down')" @mouseup="sendPtz('stop')" class="ptz-btn">&#9660;</button>
      <div></div>
    </div>
    <div class="flex gap-2 justify-center">
      <button @mousedown="sendPtz('zoom_in')" @mouseup="sendPtz('stop')" class="ptz-btn text-xs px-3">Z+</button>
      <button @mousedown="sendPtz('zoom_out')" @mouseup="sendPtz('stop')" class="ptz-btn text-xs px-3">Z-</button>
    </div>
    <div class="mt-3">
      <label class="text-xs text-gray-500">Speed</label>
      <input v-model.number="speed" type="range" min="0.1" max="1" step="0.1" class="w-full" />
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ cameraId: string }>()
const speed = ref(0.5)

const sendPtz = async (action: string) => {
  try {
    await useApi(`/api/cameras/${props.cameraId}/ptz`, {
      method: 'POST',
      body: { action, speed: speed.value },
    })
  } catch (e) {
    console.error('PTZ command failed', e)
  }
}
</script>

<style scoped>
.ptz-btn {
  @apply bg-nvr-darker border border-nvr-border rounded p-2 text-center hover:bg-primary-600/30 active:bg-primary-600 transition-colors select-none;
}
</style>
