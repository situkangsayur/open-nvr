<template>
  <div class="bg-black/75 backdrop-blur-sm rounded-xl select-none text-white" :class="size === 'large' ? 'p-4' : 'p-3'" @contextmenu.prevent>
    <div class="flex items-center justify-between gap-2 mb-2">
      <div class="text-xs opacity-80 truncate">{{ cameraName || 'PTZ' }}</div>
      <button v-if="closable" class="text-xs opacity-70 hover:opacity-100 px-1" title="Hide PTZ" @click="$emit('close')">&#10005;</button>
    </div>

    <!-- Direction pad -->
    <div class="grid grid-cols-3 gap-1.5 mx-auto" :class="size === 'large' ? 'w-44' : 'w-36'">
      <div></div>
      <button v-bind="hold('tilt_up')" :class="btnClass('tilt_up')" aria-label="Tilt up">&#9650;</button>
      <div></div>
      <button v-bind="hold('pan_left')" :class="btnClass('pan_left')" aria-label="Pan left">&#9664;</button>
      <button :class="btnClass('home')" class="!text-xs" aria-label="Home position" @click="goHome">Home</button>
      <button v-bind="hold('pan_right')" :class="btnClass('pan_right')" aria-label="Pan right">&#9654;</button>
      <div></div>
      <button v-bind="hold('tilt_down')" :class="btnClass('tilt_down')" aria-label="Tilt down">&#9660;</button>
      <div></div>
    </div>

    <!-- Zoom -->
    <div class="flex gap-1.5 justify-center items-center mt-2">
      <button v-bind="hold('zoom_out')" :class="btnClass('zoom_out')" class="w-14" aria-label="Zoom out">&minus;</button>
      <span class="text-xs opacity-70 px-1">Zoom</span>
      <button v-bind="hold('zoom_in')" :class="btnClass('zoom_in')" class="w-14" aria-label="Zoom in">+</button>
    </div>

    <!-- Speed -->
    <label class="flex items-center gap-2 mt-3 text-[11px] opacity-80">
      <span>Speed</span>
      <input v-model.number="speed" type="range" min="0.1" max="1" step="0.1" class="flex-1 accent-primary-500" />
      <span class="w-6 text-right">{{ Math.round(speed * 100) }}</span>
    </label>

    <div v-if="showHint" class="hidden md:block text-center text-[10px] mt-2 opacity-50">Keyboard: arrows, + / &minus;</div>
  </div>
</template>

<script setup lang="ts">
import type { PtzAction } from '~/composables/usePtz'

const props = withDefaults(
  defineProps<{
    cameraId: string
    cameraName?: string
    size?: 'medium' | 'large'
    closable?: boolean
    showHint?: boolean
  }>(),
  { size: 'medium', closable: false, showHint: true },
)
defineEmits<{ close: [] }>()

const { speed, move, stop, home } = usePtz()
/** The action currently held down, and on which camera. */
const active = ref<PtzAction | null>(null)
let activeCamera: string | null = null

const press = (e: PointerEvent, action: PtzAction) => {
  // Left mouse button only; any touch/pen contact counts.
  if (e.pointerType === 'mouse' && e.button !== 0) return
  e.preventDefault()
  try {
    ;(e.currentTarget as Element).setPointerCapture(e.pointerId)
  } catch {}
  if (active.value) release()
  active.value = action
  activeCamera = props.cameraId
  move(props.cameraId, action)
}

const release = () => {
  if (!active.value || !activeCamera) return
  const cam = activeCamera
  active.value = null
  activeCamera = null
  stop(cam)
}

/** Press-and-hold bindings; `touch-action: none` stops the browser from scrolling/zooming instead. */
const hold = (action: PtzAction) => ({
  style: { touchAction: 'none' },
  onPointerdown: (e: PointerEvent) => press(e, action),
  onPointerup: release,
  onPointercancel: release,
  onPointerleave: release,
  onLostpointercapture: release,
})

const goHome = () => home(props.cameraId)

const btnClass = (action: PtzAction) => {
  const base = 'rounded text-center transition-colors flex items-center justify-center font-semibold'
  const colour = active.value === action ? 'bg-primary-600' : 'bg-white/20 hover:bg-white/35 active:bg-primary-600'
  const dims = props.size === 'large' ? 'h-12 text-lg' : 'h-10 text-base'
  return `${base} ${colour} ${dims}`
}

// Never leave a camera moving: switching cameras or leaving the page stops it.
watch(() => props.cameraId, release)
onBeforeUnmount(release)
</script>
