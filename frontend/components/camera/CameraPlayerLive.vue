<template>
  <div class="relative w-full h-full bg-black overflow-hidden select-none">
    <!-- Live video (MSE). Always rendered: the MediaSource must stay attached. -->
    <video
      ref="videoEl"
      muted
      autoplay
      playsinline
      disablepictureinpicture
      class="absolute inset-0 w-full h-full"
      :class="fit === 'cover' ? 'object-cover' : 'object-contain'"
    />

    <!-- Snapshot placeholder until the first live frame -->
    <img
      v-if="snapshot && !isPlaying"
      :src="snapshot"
      alt=""
      class="absolute inset-0 w-full h-full"
      :class="fit === 'cover' ? 'object-cover' : 'object-contain'"
    />

    <!-- Header: status + name -->
    <div
      v-if="showName"
      class="absolute top-0 inset-x-0 flex items-center gap-1.5 bg-gradient-to-b from-black/70 to-transparent pointer-events-none"
      :class="compact ? 'px-1.5 pt-1 pb-3' : 'px-3 pt-2 pb-5'"
    >
      <span class="shrink-0 rounded-full" :class="[dotClass, compact ? 'w-2 h-2' : 'w-2.5 h-2.5']" :title="statusText"></span>
      <span class="truncate text-white font-medium drop-shadow" :class="compact ? 'text-[11px]' : 'text-sm'">{{ camera.name }}</span>
    </div>

    <!-- Actions slot (top right), e.g. enlarge -->
    <div class="absolute top-0 right-0 flex gap-1" :class="compact ? 'p-1' : 'p-2'" @click.stop @dblclick.stop>
      <slot name="actions" />
    </div>

    <!-- Connecting indicator -->
    <div
      v-if="state === 'connecting' && !isPlaying"
      class="absolute inset-0 flex items-center justify-center pointer-events-none"
    >
      <div class="flex items-center gap-2 bg-black/50 text-gray-200 rounded-full px-3 py-1" :class="compact ? 'text-[10px]' : 'text-xs'">
        <span class="w-3 h-3 border-2 border-gray-300 border-t-transparent rounded-full animate-spin"></span>
        {{ error ? 'Reconnecting' : 'Connecting' }}
      </div>
    </div>

    <!-- Error / offline placeholder -->
    <div
      v-if="showProblem"
      class="absolute inset-0 flex flex-col items-center justify-center text-center gap-1.5 p-2"
      :class="snapshot ? 'bg-black/60' : 'bg-gray-900'"
      @click.stop
      @dblclick.stop
    >
      <div class="text-gray-300 font-medium" :class="compact ? 'text-[11px]' : 'text-sm'">
        {{ state === 'unsupported' ? 'Not supported' : camera.status === 'offline' ? 'Camera offline' : 'No video' }}
      </div>
      <div v-if="error" class="text-gray-400 max-w-xs line-clamp-2" :class="compact ? 'text-[10px]' : 'text-xs'">{{ error }}</div>
      <button
        v-if="state !== 'unsupported'"
        class="mt-1 bg-primary-600 hover:bg-primary-700 text-white rounded"
        :class="compact ? 'text-[10px] px-2 py-0.5' : 'text-xs px-3 py-1'"
        @click.stop="retry"
      >
        Retry
      </button>
    </div>

    <!-- Bottom bar: mute + LIVE -->
    <div class="absolute bottom-0 inset-x-0 flex items-end justify-between pointer-events-none" :class="compact ? 'p-1' : 'p-2'">
      <button
        v-if="isPlaying && hasAudio"
        class="pointer-events-auto text-white bg-black/50 hover:bg-black/70 rounded transition-colors"
        :class="compact ? 'text-[10px] px-1.5 py-0.5' : 'text-xs px-2 py-1'"
        :title="muted ? 'Unmute' : 'Mute'"
        @click.stop="toggleMute"
        @dblclick.stop
      >
        {{ muted ? 'Sound off' : 'Sound on' }}
      </button>
      <span v-else></span>
      <span
        v-if="isPlaying"
        class="bg-red-600 text-white rounded font-bold tracking-wide"
        :class="compact ? 'text-[9px] px-1 py-px' : 'text-[10px] px-1.5 py-0.5'"
      >
        LIVE
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { LiveQuality } from '~/composables/useMsePlayer'

interface LiveCamera {
  id: string
  name: string
  status?: string
}

const props = withDefaults(
  defineProps<{
    camera: LiveCamera
    quality?: LiveQuality
    showName?: boolean
    /** Smaller overlays for dense grids. */
    compact?: boolean
    fit?: 'contain' | 'cover'
  }>(),
  { quality: 'main', showName: true, compact: false, fit: 'contain' },
)

const videoEl = ref<HTMLVideoElement | null>(null)
const muted = ref(true)
const snapshot = ref<string | null>(null)

const { state, error, hasAudio, start } = useMsePlayer({
  video: videoEl,
  cameraId: () => props.camera.id,
  quality: () => props.quality,
})
const { getCached, fetchSnapshot } = useSnapshotCache()

const isPlaying = computed(() => state.value === 'playing')
const showProblem = computed(() => state.value === 'error' || state.value === 'unsupported')

const dotClass = computed(() => {
  if (isPlaying.value) return 'bg-green-500'
  if (state.value === 'connecting' || state.value === 'idle') return props.camera.status === 'offline' ? 'bg-red-500' : 'bg-yellow-400'
  return 'bg-red-500'
})
const statusText = computed(() => (isPlaying.value ? 'Live' : state.value === 'error' ? 'Error' : props.camera.status || 'Connecting'))

const loadSnapshot = async () => {
  const id = props.camera.id
  snapshot.value = getCached(id)
  if (snapshot.value) return
  const url = await fetchSnapshot(id)
  // Only useful while live video has not arrived yet.
  if (url && id === props.camera.id && !isPlaying.value) snapshot.value = url
}

const begin = () => {
  if (videoEl.value) videoEl.value.muted = muted.value
  start()
  loadSnapshot()
}

const retry = () => begin()

const toggleMute = () => {
  muted.value = !muted.value
  if (videoEl.value) {
    videoEl.value.muted = muted.value
    if (!muted.value) videoEl.value.play().catch(() => {})
  }
}

onMounted(begin)

watch(
  () => [props.camera.id, props.quality] as const,
  ([id], [oldId]) => {
    if (id !== oldId) snapshot.value = null
    begin()
  },
)

defineExpose({ retry })
</script>
