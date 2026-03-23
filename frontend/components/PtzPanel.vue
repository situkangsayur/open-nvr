<template>
  <div class="bg-black/70 backdrop-blur-sm rounded-xl p-3 select-none" :class="size === 'large' ? 'p-4' : ''">
    <!-- Camera name -->
    <div v-if="cameraName" class="text-center text-white text-xs mb-2 opacity-70">{{ cameraName }}</div>

    <!-- Direction pad -->
    <div class="grid grid-cols-3 gap-1 mx-auto" :class="size === 'large' ? 'w-36' : 'w-28'">
      <div></div>
      <button @mousedown="send('tilt_up')" @mouseup="send('stop')" @mouseleave="send('stop')" :class="btnClass">&#9650;</button>
      <div></div>
      <button @mousedown="send('pan_left')" @mouseup="send('stop')" @mouseleave="send('stop')" :class="btnClass">&#9664;</button>
      <button @click="send('home')" :class="btnClass" class="text-xs">H</button>
      <button @mousedown="send('pan_right')" @mouseup="send('stop')" @mouseleave="send('stop')" :class="btnClass">&#9654;</button>
      <div></div>
      <button @mousedown="send('tilt_down')" @mouseup="send('stop')" @mouseleave="send('stop')" :class="btnClass">&#9660;</button>
      <div></div>
    </div>

    <!-- Zoom -->
    <div class="flex gap-1 justify-center mt-2">
      <button @mousedown="send('zoom_out')" @mouseup="send('stop')" @mouseleave="send('stop')" :class="zoomClass">
        <span class="text-lg font-bold">-</span>
      </button>
      <div class="text-white text-xs flex items-center px-2 opacity-60">Zoom</div>
      <button @mousedown="send('zoom_in')" @mouseup="send('stop')" @mouseleave="send('stop')" :class="zoomClass">
        <span class="text-lg font-bold">+</span>
      </button>
    </div>

    <!-- Keyboard hint -->
    <div class="text-center text-gray-400 text-[9px] mt-2 opacity-50">
      Arrow keys + / -
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  cameraId: string
  size?: 'small' | 'medium' | 'large'
  cameraName?: string
}>()

const btnClass = computed(() => {
  const base = 'bg-white/20 hover:bg-white/40 active:bg-primary-600 text-white rounded text-center transition-colors'
  return props.size === 'large' ? `${base} p-3 text-lg` : `${base} p-2 text-sm`
})

const zoomClass = computed(() => {
  const base = 'bg-white/20 hover:bg-white/40 active:bg-primary-600 text-white rounded flex items-center justify-center transition-colors'
  return props.size === 'large' ? `${base} w-14 h-10` : `${base} w-10 h-8`
})

const send = async (action: string) => {
  try {
    await useApi(`/api/cameras/${props.cameraId}/ptz`, {
      method: 'POST',
      body: { action, speed: 0.5 },
    })
  } catch {}
}
</script>
