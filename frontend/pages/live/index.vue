<template>
  <div>
    <!-- ================= Single camera view ================= -->
    <template v-if="singleCamera">
      <div class="flex flex-wrap items-center gap-2 mb-3">
        <button class="px-3 py-1.5 rounded-lg text-sm border border-gray-200 dark:border-nvr-border bg-white dark:bg-nvr-card hover:bg-gray-100 dark:hover:bg-nvr-border" @click="exitSingle">
          &larr; Grid
        </button>
        <div class="flex items-center gap-2 min-w-0 flex-1">
          <span class="w-2.5 h-2.5 rounded-full shrink-0" :class="singleCamera.status === 'offline' ? 'bg-red-500' : 'bg-green-500'"></span>
          <h1 class="text-lg font-bold truncate">{{ singleCamera.name }}</h1>
        </div>
        <div class="flex items-center gap-1.5">
          <button v-if="cameras.length > 1" :class="iconBtn" title="Previous camera" @click="stepSingle(-1)">&#9664;</button>
          <button v-if="cameras.length > 1" :class="iconBtn" title="Next camera" @click="stepSingle(1)">&#9654;</button>
          <button class="px-3 py-1.5 rounded-lg text-sm bg-primary-600 hover:bg-primary-700 text-white" title="Open recordings of this camera" @click="openRecordings(singleCamera.id)">
            Recordings
          </button>
          <button v-if="canFullscreen" :class="iconBtn" title="Fullscreen" @click="toggleFullscreen">&#x26F6;</button>
          <button v-if="singleCamera.ptz_capable" :class="iconBtn" class="lg:hidden" :title="showPtz ? 'Hide PTZ' : 'Show PTZ'" @click="showPtz = !showPtz">PTZ</button>
        </div>
      </div>

      <div class="flex flex-col lg:flex-row gap-3">
        <div
          ref="singleWrap"
          class="relative flex-1 min-w-0 bg-black rounded-lg overflow-hidden aspect-video lg:aspect-auto lg:h-[calc(100vh-8.5rem)]"
          @dblclick="exitSingle"
        >
          <CameraPlayerLive :key="singleCamera.id" :camera="singleCamera" quality="main" />
        </div>
        <aside v-if="singleCamera.ptz_capable && (showPtz || isLarge)" class="lg:w-60 shrink-0">
          <PtzPanel :camera-id="singleCamera.id" :camera-name="singleCamera.name" size="large" />
        </aside>
      </div>
    </template>

    <!-- ================= Grid view ================= -->
    <template v-else>
      <div class="flex flex-wrap items-center justify-between gap-2 mb-3">
        <h1 class="text-xl font-bold">Live View</h1>
        <div class="flex flex-wrap items-center gap-2">
          <div class="flex rounded-lg overflow-hidden border border-gray-200 dark:border-nvr-border" role="group" aria-label="Layout">
            <button
              v-for="opt in LAYOUTS"
              :key="opt.value"
              class="px-2.5 h-8 text-xs font-medium transition-colors"
              :class="layout === opt.value ? 'bg-primary-600 text-white' : 'bg-white dark:bg-nvr-card text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-nvr-border'"
              :title="opt.title"
              @click="setLayout(opt.value)"
            >
              {{ opt.label }}
            </button>
          </div>
          <div v-if="pageCount > 1" class="flex items-center gap-1">
            <button :class="iconBtn" :disabled="page === 0" title="Previous page" @click="page--">&#9664;</button>
            <span class="text-xs text-gray-500 dark:text-gray-400 w-12 text-center">{{ page + 1 }} / {{ pageCount }}</span>
            <button :class="iconBtn" :disabled="page >= pageCount - 1" title="Next page" @click="page++">&#9654;</button>
          </div>
        </div>
      </div>

      <div v-if="!loaded" class="text-center py-20 text-gray-400">Loading cameras...</div>

      <div v-else-if="loadError && cameras.length === 0" class="text-center py-20">
        <p class="text-red-400 mb-4">{{ loadError }}</p>
        <button class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg text-sm" @click="loadCameras">Retry</button>
      </div>

      <div v-else-if="cameras.length === 0" class="text-center py-20">
        <p class="text-gray-400 text-lg mb-4">No cameras configured</p>
        <NuxtLink to="/cameras" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg text-sm">Add Camera</NuxtLink>
      </div>

      <div v-else class="mx-auto" :style="gridBoxStyle">
        <div class="grid gap-1.5" :style="{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }">
          <div
            v-for="camera in visibleCameras"
            :key="camera.id"
            class="relative aspect-video rounded-md overflow-hidden border-2 cursor-pointer transition-colors"
            :class="selectedId === camera.id ? 'border-primary-500' : 'border-transparent hover:border-primary-300/60'"
            @click="selectedId = camera.id"
            @dblclick="enterSingle(camera.id)"
          >
            <CameraPlayerLive :camera="camera" :quality="tileQuality" :compact="cols >= 3">
              <template #actions>
                <button
                  class="text-white bg-black/50 hover:bg-black/80 rounded leading-none"
                  :class="cols >= 3 ? 'text-[11px] px-1.5 py-1' : 'text-xs px-2 py-1.5'"
                  title="Enlarge"
                  @click="enterSingle(camera.id)"
                >
                  &#x2922;
                </button>
              </template>
            </CameraPlayerLive>
          </div>
          <!-- Keep the grid shape on partially filled pages -->
          <div v-for="n in fillerCount" :key="`empty-${n}`" class="aspect-video rounded-md bg-gray-200 dark:bg-nvr-dark border-2 border-transparent"></div>
        </div>
        <p class="hidden md:block text-[11px] text-gray-400 dark:text-gray-500 mt-2">
          Click a tile to select it (PTZ, arrow keys) &middot; double-click to enlarge &middot; Esc to go back
        </p>
      </div>

      <!-- Floating PTZ for the selected camera -->
      <div v-if="selectedCamera?.ptz_capable" class="fixed bottom-4 right-4 z-30">
        <PtzPanel
          v-if="showPtz"
          :camera-id="selectedCamera.id"
          :camera-name="selectedCamera.name"
          closable
          @close="showPtz = false"
        />
        <button v-else class="bg-black/75 text-white rounded-full px-4 py-2 text-sm shadow-lg" @click="showPtz = true">PTZ</button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import type { PtzAction } from '~/composables/usePtz'

interface LiveCamera {
  id: string
  name: string
  status?: string
  ptz_capable?: boolean
}

type LayoutMode = 'auto' | 1 | 4 | 9 | 16

const LAYOUTS: { value: LayoutMode; label: string; title: string }[] = [
  { value: 'auto', label: 'Auto', title: 'Fit all cameras' },
  { value: 1, label: '1', title: '1 camera' },
  { value: 4, label: '4', title: '2 x 2' },
  { value: 9, label: '9', title: '3 x 3' },
  { value: 16, label: '16', title: '4 x 4' },
]
const LAYOUT_KEY = 'opennvr-live-layout'
const PTZ_KEY = 'opennvr-live-ptz-visible'

const KEY_ACTIONS: Record<string, PtzAction> = {
  ArrowUp: 'tilt_up',
  ArrowDown: 'tilt_down',
  ArrowLeft: 'pan_left',
  ArrowRight: 'pan_right',
  '+': 'zoom_in',
  '=': 'zoom_in',
  '-': 'zoom_out',
}

const route = useRoute()
const router = useRouter()
const ptz = usePtz()

const iconBtn =
  'h-8 min-w-8 px-2 rounded-lg text-xs border border-gray-200 dark:border-nvr-border bg-white dark:bg-nvr-card hover:bg-gray-100 dark:hover:bg-nvr-border disabled:opacity-40'

const cameras = ref<LiveCamera[]>([])
const loaded = ref(false)
const loadError = ref('')
const layout = ref<LayoutMode>(4)
const page = ref(0)
const selectedId = ref<string | null>(null)
const showPtz = ref(true)
const singleWrap = ref<HTMLElement | null>(null)
const isNarrow = ref(false)
const isLarge = ref(true)
const canFullscreen = ref(false)

// ---- layout / paging ---------------------------------------------------------

const pageSize = computed(() => (layout.value === 'auto' ? Math.max(1, cameras.value.length) : layout.value))
const pageCount = computed(() => Math.max(1, Math.ceil(cameras.value.length / pageSize.value)))
const visibleCameras = computed(() => cameras.value.slice(page.value * pageSize.value, (page.value + 1) * pageSize.value))

/** Slots the grid is shaped for (a 3x3 layout keeps 9 slots even with 5 cameras). */
const slotCount = computed(() => (layout.value === 'auto' ? visibleCameras.value.length : pageSize.value))
const cols = computed(() => {
  const c = Math.ceil(Math.sqrt(Math.max(1, slotCount.value)))
  return isNarrow.value ? Math.min(c, 2) : c
})
const rows = computed(() => Math.ceil(Math.max(1, isNarrow.value ? visibleCameras.value.length : slotCount.value) / cols.value))
const fillerCount = computed(() =>
  isNarrow.value || cameras.value.length <= pageSize.value ? 0 : Math.max(0, slotCount.value - visibleCameras.value.length),
)
/** Sub stream once 4+ tiles share the screen; main stream otherwise. */
const tileQuality = computed(() => (visibleCameras.value.length >= 4 ? 'sub' : 'main'))

/** On wide screens, cap the grid width so every row fits in the viewport without scrolling. */
const gridBoxStyle = computed(() => {
  if (isNarrow.value) return {}
  const ratio = (16 * cols.value) / (9 * rows.value)
  return { maxWidth: `calc((100vh - 9rem) * ${ratio.toFixed(4)})` }
})

const setLayout = (value: LayoutMode) => {
  layout.value = value
  page.value = 0
  try {
    localStorage.setItem(LAYOUT_KEY, String(value))
  } catch {}
}

watch(pageCount, (n) => {
  if (page.value > n - 1) page.value = n - 1
})

// ---- selection / single view -------------------------------------------------

const singleId = computed(() => (typeof route.query.camera === 'string' ? route.query.camera : null))
const singleCamera = computed(() => (singleId.value ? cameras.value.find((c) => c.id === singleId.value) ?? null : null))
const selectedCamera = computed(() => cameras.value.find((c) => c.id === selectedId.value) ?? null)

const enterSingle = (id: string) => {
  selectedId.value = id
  // push, so the browser/Android back button returns to the grid
  router.push({ query: { ...route.query, camera: id } })
}

const exitSingle = () => {
  if (import.meta.client && document.fullscreenElement) document.exitFullscreen().catch(() => {})
  const back = import.meta.client ? window.history.state?.back : null
  if (typeof back === 'string' && back.split('?')[0] === '/live' && !back.includes('camera=')) {
    router.back()
  } else {
    const { camera: _drop, ...rest } = route.query
    router.replace({ query: rest })
  }
}

const stepSingle = (delta: number) => {
  if (!singleCamera.value) return
  const idx = cameras.value.findIndex((c) => c.id === singleCamera.value!.id)
  const next = cameras.value[(idx + delta + cameras.value.length) % cameras.value.length]
  selectedId.value = next.id
  router.replace({ query: { ...route.query, camera: next.id } })
}

// When returning from the single view, show the page that contains that camera.
watch(singleId, (id, oldId) => {
  if (!id && oldId) {
    const idx = cameras.value.findIndex((c) => c.id === oldId)
    if (idx >= 0) page.value = Math.floor(idx / pageSize.value)
  }
})

/** Playback of this camera, starting a few minutes back (the newest file is still being written). */
const openRecordings = (cameraId: string) =>
  navigateTo({ path: '/playback', query: { camera: cameraId, t: new Date(Date.now() - 5 * 60 * 1000).toISOString() } })

const toggleFullscreen = async () => {
  const el = singleWrap.value as (HTMLElement & { webkitRequestFullscreen?: () => void }) | null
  if (!el) return
  try {
    if (document.fullscreenElement) await document.exitFullscreen()
    else if (el.requestFullscreen) await el.requestFullscreen()
    else el.webkitRequestFullscreen?.()
  } catch {}
}

watch(showPtz, (v) => {
  try {
    localStorage.setItem(PTZ_KEY, v ? '1' : '0')
  } catch {}
})

// ---- keyboard PTZ (hold to move, release to stop) ------------------------------

let heldKey: string | null = null
let heldCamera: string | null = null

const releaseKey = () => {
  if (heldKey && heldCamera) ptz.stop(heldCamera)
  heldKey = null
  heldCamera = null
}

const onKeydown = (e: KeyboardEvent) => {
  const target = e.target as HTMLElement | null
  if (target && (['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName) || target.isContentEditable)) return
  if (e.key === 'Escape' && singleCamera.value) {
    exitSingle()
    return
  }
  const action = KEY_ACTIONS[e.key]
  if (!action) return
  const cam = singleCamera.value ?? selectedCamera.value
  if (!cam?.ptz_capable) return
  e.preventDefault()
  if (e.repeat || heldKey) return
  heldKey = e.key
  heldCamera = cam.id
  ptz.move(cam.id, action)
}

const onKeyup = (e: KeyboardEvent) => {
  if (!heldKey) return
  // '+' may be released as '=' (shift released first); treat any mapped key as a release.
  if (e.key === heldKey || KEY_ACTIONS[e.key]) releaseKey()
}

// ---- data ----------------------------------------------------------------------

const loadCameras = async () => {
  try {
    const list = await useApi<LiveCamera[]>('/api/cameras')
    cameras.value = Array.isArray(list) ? list : []
    loadError.value = ''
    if (!selectedId.value || !cameras.value.some((c) => c.id === selectedId.value)) {
      selectedId.value = singleId.value ?? cameras.value[0]?.id ?? null
    }
  } catch (e: any) {
    loadError.value = `Could not load cameras (${e?.statusCode ? `HTTP ${e.statusCode}` : 'server unreachable'})`
  } finally {
    loaded.value = true
  }
}

let refreshTimer: ReturnType<typeof setInterval> | null = null
let narrowMq: MediaQueryList | null = null
let largeMq: MediaQueryList | null = null
const onMq = () => {
  isNarrow.value = !!narrowMq?.matches
  isLarge.value = !!largeMq?.matches
}

onMounted(() => {
  try {
    const saved = localStorage.getItem(LAYOUT_KEY)
    if (saved === 'auto') layout.value = 'auto'
    else if (saved && [1, 4, 9, 16].includes(Number(saved))) layout.value = Number(saved) as LayoutMode
    showPtz.value = localStorage.getItem(PTZ_KEY) !== '0'
  } catch {}

  narrowMq = window.matchMedia('(max-width: 639px)')
  largeMq = window.matchMedia('(min-width: 1024px)')
  onMq()
  narrowMq.addEventListener('change', onMq)
  largeMq.addEventListener('change', onMq)

  const el = document.documentElement as HTMLElement & { webkitRequestFullscreen?: unknown }
  canFullscreen.value = !!(el.requestFullscreen || el.webkitRequestFullscreen)

  window.addEventListener('keydown', onKeydown)
  window.addEventListener('keyup', onKeyup)
  window.addEventListener('blur', releaseKey)

  loadCameras()
  // Keeps names/status/ptz flags fresh; players are keyed by id so they are not restarted.
  refreshTimer = setInterval(loadCameras, 30000)
})

onBeforeUnmount(() => {
  releaseKey()
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('keyup', onKeyup)
  window.removeEventListener('blur', releaseKey)
  narrowMq?.removeEventListener('change', onMq)
  largeMq?.removeEventListener('change', onMq)
  if (refreshTimer) clearInterval(refreshTimer)
})
</script>
