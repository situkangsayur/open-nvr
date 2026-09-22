<template>
  <div>
    <div class="flex flex-wrap items-center justify-between gap-2 mb-3">
      <h1 class="text-xl font-bold">Playback</h1>
      <div v-if="cameras.length > 6" class="sm:hidden w-full">
        <select :value="cameraId" :class="selectClass" class="w-full" @change="selectCamera(($event.target as HTMLSelectElement).value)">
          <option v-for="cam in cameras" :key="cam.id" :value="cam.id">{{ cam.name }}</option>
        </select>
      </div>
    </div>

    <!-- Camera cards -->
    <div v-if="!camerasLoaded" class="text-gray-400 py-10 text-center">Loading cameras...</div>
    <div v-else-if="cameras.length === 0" class="text-center py-16">
      <p class="text-gray-400 mb-4">{{ camerasError || 'No cameras configured' }}</p>
      <NuxtLink to="/cameras" class="bg-primary-600 hover:bg-primary-700 text-white px-4 py-2 rounded-lg text-sm">Cameras</NuxtLink>
    </div>
    <template v-else>
      <div class="flex gap-2 overflow-x-auto pb-2 mb-3 -mx-1 px-1">
        <button
          v-for="cam in cameras"
          :key="cam.id"
          class="shrink-0 w-32 sm:w-40 rounded-lg overflow-hidden border-2 text-left transition-colors bg-white dark:bg-nvr-card"
          :class="cam.id === cameraId ? 'border-primary-500' : 'border-gray-200 dark:border-nvr-border hover:border-primary-300'"
          @click="selectCamera(cam.id)"
        >
          <div class="aspect-video bg-gray-800 relative">
            <img v-if="thumbs[cam.id]" :src="thumbs[cam.id]" alt="" class="w-full h-full object-cover" />
          </div>
          <div class="flex items-center gap-1.5 px-2 py-1.5">
            <span class="w-2 h-2 rounded-full shrink-0" :class="cam.status === 'offline' ? 'bg-red-500' : 'bg-green-500'"></span>
            <span class="text-xs font-medium truncate">{{ cam.name }}</span>
          </div>
        </button>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-4 gap-3">
        <!-- Player -->
        <div class="lg:col-span-3 space-y-3">
          <div class="relative bg-black rounded-lg overflow-hidden aspect-video">
            <video
              ref="videoEl"
              controls
              playsinline
              preload="metadata"
              class="w-full h-full object-contain"
              @loadedmetadata="onLoadedMetadata"
              @timeupdate="onTimeUpdate"
              @ended="advance"
              @error="onVideoError"
              @ratechange="onRateChange"
            />
            <div v-if="!current" class="absolute inset-0 flex items-center justify-center text-gray-400 text-sm text-center p-4 pointer-events-none">
              <span v-if="loadingFiles || loadingDays">Loading recordings...</span>
              <span v-else-if="files.length">Click the timeline or pick a segment below to play</span>
              <span v-else-if="cameraId">No recordings for this day</span>
            </div>
            <!-- Segment still being written -->
            <div v-if="notice" class="absolute inset-0 flex items-center justify-center bg-black/75 p-4">
              <div class="max-w-md text-center text-white space-y-3">
                <p class="text-sm">{{ notice }}</p>
                <div class="flex flex-wrap justify-center gap-2">
                  <NuxtLink :to="{ path: '/live', query: { camera: cameraId } }" class="bg-red-600 hover:bg-red-700 text-white rounded-lg px-3 py-1.5 text-sm">
                    Watch live
                  </NuxtLink>
                  <button v-if="lastComplete" class="bg-white/20 hover:bg-white/30 rounded-lg px-3 py-1.5 text-sm" @click="playLatestFinished">
                    Play latest finished ({{ formatClock(lastComplete.endMs) }})
                  </button>
                  <button class="bg-white/10 hover:bg-white/20 rounded-lg px-3 py-1.5 text-sm" @click="notice = null">Close</button>
                </div>
              </div>
            </div>
            <div v-if="current && positionMs != null" class="absolute top-2 left-2 bg-black/60 text-white text-xs sm:text-sm font-mono rounded px-2 py-1 pointer-events-none">
              {{ formatDate(positionMs) }} {{ formatClock(positionMs, true) }}
            </div>
          </div>

          <!-- Transport -->
          <div class="flex flex-wrap items-center gap-2">
            <button :class="btn" :disabled="!prevFile" title="Previous segment" @click="prevFile && playFile(prevFile)">&#9198; Prev</button>
            <button :class="btn" :disabled="!current" title="Back 10 s" @click="skip(-10)">-10s</button>
            <button :class="btn" :disabled="!current" title="Forward 10 s" @click="skip(10)">+10s</button>
            <button :class="btn" :disabled="!nextFile" title="Next segment" @click="nextFile && playFile(nextFile)">Next &#9197;</button>
            <div class="flex rounded-lg overflow-hidden border border-gray-200 dark:border-nvr-border">
              <button
                v-for="s in SPEEDS"
                :key="s"
                class="px-2 h-8 text-xs transition-colors"
                :class="speed === s ? 'bg-primary-600 text-white' : 'bg-white dark:bg-nvr-card hover:bg-gray-100 dark:hover:bg-nvr-border'"
                @click="speed = s"
              >
                {{ s }}x
              </button>
            </div>
            <a
              v-if="current"
              :href="mediaUrl(current, true)"
              download
              rel="noopener"
              :class="btn"
              class="inline-flex items-center"
              title="Download this segment (MP4)"
            >
              Download
            </a>
          </div>
        </div>

        <!-- Side panel -->
        <div class="space-y-3">
          <div class="bg-white dark:bg-nvr-card rounded-lg p-3 border border-gray-200 dark:border-nvr-border">
            <h2 class="text-xs font-semibold uppercase tracking-wide text-gray-400 mb-2">Day</h2>
            <div class="flex gap-1">
              <button :class="btn" :disabled="!olderDay" title="Previous day" @click="olderDay && selectDate(olderDay)">&#9664;</button>
              <select :value="date" :class="selectClass" class="flex-1 min-w-0" :disabled="!dayOptions.length" @change="selectDate(($event.target as HTMLSelectElement).value)">
                <option v-if="!dayOptions.length" value="">{{ loadingDays ? 'Loading...' : 'No recordings' }}</option>
                <option v-for="d in dayOptions" :key="d.date" :value="d.date">
                  {{ formatDayLabel(d.date) }}{{ d.count ? ` (${d.count})` : '' }}
                </option>
              </select>
              <button :class="btn" :disabled="!newerDay" title="Next day" @click="newerDay && selectDate(newerDay)">&#9654;</button>
            </div>
            <p v-if="currentDay" class="text-[11px] text-gray-500 mt-1.5">
              {{ currentDay.count }} files &middot; {{ formatBytes(currentDay.size_bytes) }}
            </p>
          </div>

          <div class="bg-white dark:bg-nvr-card rounded-lg p-3 border border-gray-200 dark:border-nvr-border">
            <h2 class="text-xs font-semibold uppercase tracking-wide text-gray-400 mb-2">Jump to</h2>
            <div class="grid grid-cols-2 gap-1.5">
              <button v-for="q in QUICK_JUMPS" :key="q.label" :class="btn" :disabled="!cameraId" @click="jumpTo(Date.now() - q.ms)">
                {{ q.label }}
              </button>
            </div>
          </div>

          <div v-if="current" class="bg-white dark:bg-nvr-card rounded-lg p-3 border border-gray-200 dark:border-nvr-border text-sm">
            <h2 class="text-xs font-semibold uppercase tracking-wide text-gray-400 mb-2">Segment</h2>
            <div class="font-mono">{{ formatClock(current.startMs, true) }} &ndash; {{ formatClock(current.endMs, true) }}</div>
            <div class="text-xs text-gray-500 mt-1">
              {{ formatDuration(current.duration_secs || (current.endMs - current.startMs) / 1000) }} &middot; {{ formatBytes(current.size_bytes) }}
            </div>
            <div class="text-[11px] text-gray-400 mt-1 break-all">{{ current.filename }}</div>
          </div>
        </div>
      </div>

      <!-- Timeline -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-3 border border-gray-200 dark:border-nvr-border mt-3">
        <div class="flex flex-wrap items-center justify-between gap-2 mb-2">
          <h2 class="text-xs font-semibold uppercase tracking-wide text-gray-400">
            Timeline <span v-if="date" class="normal-case font-normal">&middot; {{ formatDayLabel(date) }}</span>
          </h2>
          <div class="flex items-center gap-1">
            <button :class="btn" :disabled="zoomHours >= 24" title="Scroll earlier" @click="pan(-1)">&#9664;</button>
            <div class="flex rounded-lg overflow-hidden border border-gray-200 dark:border-nvr-border">
              <button
                v-for="z in ZOOMS"
                :key="z"
                class="px-2 h-8 text-xs transition-colors"
                :class="zoomHours === z ? 'bg-primary-600 text-white' : 'bg-white dark:bg-nvr-card hover:bg-gray-100 dark:hover:bg-nvr-border'"
                @click="setZoom(z)"
              >
                {{ z }}h
              </button>
            </div>
            <button :class="btn" :disabled="zoomHours >= 24" title="Scroll later" @click="pan(1)">&#9654;</button>
          </div>
        </div>

        <div class="relative select-none">
          <div
            ref="barEl"
            class="relative h-12 bg-gray-100 dark:bg-nvr-darker rounded overflow-hidden"
            :class="date ? 'cursor-pointer' : 'opacity-50'"
            @click="onBarClick"
            @pointermove="onBarHover"
            @pointerleave="hoverMs = null"
          >
            <div v-if="futurePct != null" class="absolute inset-y-0 right-0 bg-gray-300/40 dark:bg-white/5" :style="{ left: `${futurePct}%` }"></div>
            <div
              v-for="(b, i) in blocks"
              :key="i"
              class="absolute top-2 bottom-3 bg-primary-500/70 dark:bg-primary-500/60 rounded-sm"
              :style="{ left: `${b.left}%`, width: `${b.width}%` }"
            ></div>
            <div
              v-for="(b, i) in recordingBlocks"
              :key="`rec-${i}`"
              class="absolute top-2 bottom-3 bg-amber-500/70 rounded-sm animate-pulse"
              title="Recording in progress (not playable yet)"
              :style="{ left: `${b.left}%`, width: `${b.width}%` }"
            ></div>
            <div
              v-if="currentBlock"
              class="absolute top-1 bottom-2 bg-primary-300/80 dark:bg-primary-300/70 rounded-sm ring-1 ring-primary-600"
              :style="{ left: `${currentBlock.left}%`, width: `${currentBlock.width}%` }"
            ></div>
            <div
              v-for="t in ticks"
              :key="t.ms"
              class="absolute bottom-0 w-px"
              :class="t.major ? 'h-3 bg-gray-500 dark:bg-gray-400' : 'h-1.5 bg-gray-400 dark:bg-gray-600'"
              :style="{ left: `${t.left}%` }"
            ></div>
            <div v-if="playheadPct != null" class="absolute inset-y-0 w-0.5 bg-red-500 pointer-events-none" :style="{ left: `${playheadPct}%` }"></div>
            <div v-if="hoverPct != null" class="absolute inset-y-0 w-px bg-gray-900/40 dark:bg-white/60 pointer-events-none" :style="{ left: `${hoverPct}%` }"></div>
          </div>
          <!-- Hover time -->
          <div
            v-if="hoverPct != null && hoverMs != null"
            class="absolute -top-7 -translate-x-1/2 bg-gray-900 text-white text-[11px] font-mono rounded px-1.5 py-0.5 pointer-events-none whitespace-nowrap"
            :style="{ left: `${Math.min(96, Math.max(4, hoverPct))}%` }"
          >
            {{ formatClock(hoverMs, zoomHours <= 2) }}
          </div>
          <!-- Labels -->
          <div class="relative h-4 mt-1 text-[10px] text-gray-500 dark:text-gray-400">
            <span
              v-for="(t, i) in labelTicks"
              :key="t.ms"
              class="absolute -translate-x-1/2 font-mono"
              :class="i % 2 === 1 ? 'hidden sm:inline' : ''"
              :style="{ left: `${t.left}%` }"
            >
              {{ formatClock(t.ms) }}
            </span>
          </div>
        </div>
        <p v-if="filesError" class="text-xs text-red-400 mt-2">{{ filesError }}</p>
      </div>

      <!-- Segment list -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-3 border border-gray-200 dark:border-nvr-border mt-3">
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-xs font-semibold uppercase tracking-wide text-gray-400">Segments ({{ files.length }})</h2>
          <button v-if="date" :class="btn" :disabled="loadingFiles" @click="loadFiles()">Refresh</button>
        </div>
        <div v-if="loadingFiles && !files.length" class="text-sm text-gray-400 py-4 text-center">Loading...</div>
        <div v-else-if="!files.length" class="text-sm text-gray-400 py-4 text-center">No segments</div>
        <div v-else ref="listEl" class="max-h-80 overflow-y-auto divide-y divide-gray-100 dark:divide-nvr-border">
          <div
            v-for="f in files"
            :key="f.filename"
            :data-file="f.filename"
            class="flex items-center gap-2 px-2 py-1.5 cursor-pointer transition-colors"
            :class="current?.filename === f.filename ? 'bg-primary-600/15' : 'hover:bg-gray-50 dark:hover:bg-nvr-darker'"
            @click="playFile(f)"
          >
            <span class="w-4 text-primary-500 text-xs">{{ current?.filename === f.filename ? '&#9654;' : '' }}</span>
            <span class="font-mono text-sm">{{ formatClock(f.startMs, true) }} &ndash; {{ formatClock(f.endMs, true) }}</span>
            <span class="text-xs text-gray-500 hidden sm:inline">{{ formatDuration(f.duration_secs || (f.endMs - f.startMs) / 1000) }}</span>
            <span v-if="!f.complete" class="text-[10px] uppercase font-semibold text-amber-500">recording&hellip;</span>
            <span class="text-xs text-gray-500 ml-auto">{{ formatBytes(f.size_bytes) }}</span>
            <span v-if="!f.complete" class="w-5"></span>
            <a
              v-else
              :href="mediaUrl(f, true)"
              download
              rel="noopener"
              class="text-xs text-primary-500 hover:underline px-1"
              title="Download"
              @click.stop
            >
              &#8681;
            </a>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import type { RecordingDay, TimedFile } from '~/composables/useRecordingFiles'

interface Cam {
  id: string
  name: string
  status?: string
}

const SPEEDS = [0.5, 1, 2, 4, 8]
const ZOOMS = [24, 6, 2, 1]
const QUICK_JUMPS = [
  { label: '5 min ago', ms: 5 * 60e3 },
  { label: '15 min ago', ms: 15 * 60e3 },
  { label: '1 hour ago', ms: 60 * 60e3 },
  { label: '3 hours ago', ms: 3 * 60 * 60e3 },
]
/** Tick spacing / label spacing in minutes, per zoom level (hours shown). */
const TICKS: Record<number, { step: number; label: number }> = {
  24: { step: 60, label: 180 },
  6: { step: 30, label: 60 },
  2: { step: 10, label: 30 },
  1: { step: 5, label: 15 },
}
const CAMERA_KEY = 'opennvr-playback-camera'

const btn =
  'h-8 px-2.5 rounded-lg text-xs border border-gray-200 dark:border-nvr-border bg-white dark:bg-nvr-card hover:bg-gray-100 dark:hover:bg-nvr-border disabled:opacity-40 disabled:cursor-not-allowed'
const selectClass =
  'h-8 bg-gray-50 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded-lg px-2 text-sm text-gray-900 dark:text-white'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const { fetchDays, fetchFiles, mediaUrl } = useRecordingFiles()
const { getSnapshot } = useSnapshotCache()

const cameras = ref<Cam[]>([])
const camerasLoaded = ref(false)
const camerasError = ref('')
const thumbs = reactive<Record<string, string>>({})
const cameraId = ref('')
const days = ref<RecordingDay[]>([])
const date = ref('')
const files = ref<TimedFile[]>([])
const loadingDays = ref(false)
const loadingFiles = ref(false)
const filesError = ref('')
const current = ref<TimedFile | null>(null)
const positionMs = ref<number | null>(null)
const speed = ref(1)
const zoomHours = ref(24)
const viewStartMs = ref(0)
const hoverMs = ref<number | null>(null)
/** Explanation shown over the player when the requested time is not playable yet. */
const notice = ref<string | null>(null)
const nowMs = ref(Date.now())

const videoEl = ref<HTMLVideoElement | null>(null)
const barEl = ref<HTMLElement | null>(null)
const listEl = ref<HTMLElement | null>(null)

let pendingSeek = 0
let pendingAutoplay = true
let retriedCurrent = false
let daysSeq = 0
let filesSeq = 0

// ---- formatting ------------------------------------------------------------------

const formatDate = (ms: number) => new Date(ms).toLocaleDateString([], { day: '2-digit', month: 'short' })
const formatDayLabel = (key: string) => {
  const label = localDayStart(key).toLocaleDateString([], { weekday: 'short', day: 'numeric', month: 'short', year: 'numeric' })
  const today = localDateKey(new Date())
  if (key === today) return `Today, ${label}`
  if (key === shiftDateKey(today, -1)) return `Yesterday, ${label}`
  return label
}
const formatDuration = (secs: number) => {
  const s = Math.max(0, Math.round(secs))
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const r = s % 60
  return h ? `${h}:${String(m).padStart(2, '0')}:${String(r).padStart(2, '0')}` : `${m}:${String(r).padStart(2, '0')}`
}

// ---- day / navigation state ------------------------------------------------------

const dayStartMs = computed(() => (date.value ? localDayStart(date.value).getTime() : 0))
const dayEndMs = computed(() => (date.value ? localDayEnd(date.value).getTime() : 0))
const daySpanMs = computed(() => Math.max(1, dayEndMs.value - dayStartMs.value))

const dayOptions = computed<RecordingDay[]>(() => {
  if (!date.value || days.value.some((d) => d.date === date.value)) return days.value
  return [...days.value, { date: date.value, count: 0, size_bytes: 0 }].sort((a, b) => (a.date < b.date ? 1 : -1))
})
const currentDay = computed(() => days.value.find((d) => d.date === date.value) ?? null)
/** days are newest first, so "older" is the next entry. */
const olderDay = computed(() => days.value.find((d) => d.date < date.value)?.date ?? null)
const newerDay = computed(() => [...days.value].reverse().find((d) => d.date > date.value)?.date ?? null)

/** Files that can be played now (the one ffmpeg is still writing cannot). */
const playable = computed(() => files.value.filter((f) => f.complete))
const lastComplete = computed(() => playable.value[playable.value.length - 1] ?? null)

const prevFile = computed(() => {
  const cur = current.value
  if (!cur) return null
  return [...playable.value].reverse().find((f) => f.startMs < cur.startMs) ?? null
})
const nextFile = computed(() => {
  const cur = current.value
  if (!cur) return null
  return playable.value.find((f) => f.startMs > cur.startMs) ?? null
})

// ---- timeline geometry -------------------------------------------------------------

const spanMs = computed(() => Math.min(daySpanMs.value, zoomHours.value * 3600e3))
const viewEndMs = computed(() => viewStartMs.value + spanMs.value)

const clampView = (start: number) => Math.min(Math.max(start, dayStartMs.value), dayEndMs.value - spanMs.value)
const pct = (ms: number) => ((ms - viewStartMs.value) / spanMs.value) * 100
const inView = (ms: number | null) => ms != null && ms >= viewStartMs.value && ms <= viewEndMs.value

const clipBlock = (startMs: number, endMs: number) => {
  const s = Math.max(startMs, viewStartMs.value)
  const e = Math.min(endMs, viewEndMs.value)
  return e > s ? { left: pct(s), width: Math.max(0.3, pct(e) - pct(s)) } : null
}

/** Files still being written, drawn separately as "recording". */
const recordingBlocks = computed(() =>
  files.value
    .filter((f) => !f.complete)
    .map((f) => clipBlock(f.startMs, Math.max(f.endMs, Math.min(nowMs.value, f.startMs + 30 * 60e3))))
    .filter((b): b is { left: number; width: number } => b !== null),
)

/** Contiguous runs of playable files (gaps under 10 s merged), clipped to the view. */
const blocks = computed(() => {
  const out: { left: number; width: number }[] = []
  let runStart = -1
  let runEnd = -1
  const push = () => {
    if (runStart < 0) return
    const s = Math.max(runStart, viewStartMs.value)
    const e = Math.min(runEnd, viewEndMs.value)
    if (e > s) out.push({ left: pct(s), width: Math.max(0.2, pct(e) - pct(s)) })
  }
  for (const f of playable.value) {
    if (runStart >= 0 && f.startMs - runEnd <= 10_000) {
      runEnd = Math.max(runEnd, f.endMs)
    } else {
      push()
      runStart = f.startMs
      runEnd = f.endMs
    }
  }
  push()
  return out
})

const currentBlock = computed(() => {
  const f = current.value
  if (!f) return null
  const s = Math.max(f.startMs, viewStartMs.value)
  const e = Math.min(f.endMs, viewEndMs.value)
  if (e <= s) return null
  return { left: pct(s), width: Math.max(0.3, pct(e) - pct(s)) }
})

const ticks = computed(() => {
  if (!date.value) return []
  const { step, label } = TICKS[zoomHours.value] ?? TICKS[24]
  const stepMs = step * 60e3
  const out: { ms: number; left: number; major: boolean }[] = []
  const first = dayStartMs.value + Math.ceil((viewStartMs.value - dayStartMs.value) / stepMs) * stepMs
  for (let t = first; t <= viewEndMs.value; t += stepMs) {
    const minutes = Math.round((t - dayStartMs.value) / 60e3)
    out.push({ ms: t, left: pct(t), major: minutes % label === 0 })
  }
  return out
})
/** Major ticks that get a text label (not the very edges, which would clip). */
const labelTicks = computed(() => ticks.value.filter((t) => t.major && t.left > 1 && t.left < 99))

const playheadPct = computed(() => (inView(positionMs.value) ? pct(positionMs.value!) : null))
const hoverPct = computed(() => (inView(hoverMs.value) ? pct(hoverMs.value!) : null))
const futurePct = computed(() => {
  if (!date.value || nowMs.value >= viewEndMs.value) return null
  return Math.max(0, pct(nowMs.value))
})

const setZoom = (hours: number) => {
  const centre = positionMs.value ?? (viewStartMs.value + spanMs.value / 2)
  zoomHours.value = hours
  viewStartMs.value = clampView(centre - spanMs.value / 2)
}
const pan = (dir: number) => {
  viewStartMs.value = clampView(viewStartMs.value + (dir * spanMs.value) / 2)
}

const timeAtPointer = (clientX: number): number | null => {
  const el = barEl.value
  if (!el || !date.value) return null
  const rect = el.getBoundingClientRect()
  if (rect.width <= 0) return null
  const frac = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width))
  return viewStartMs.value + frac * spanMs.value
}

const onBarClick = (e: MouseEvent) => {
  const t = timeAtPointer(e.clientX)
  if (t != null) playAt(t)
}
const onBarHover = (e: PointerEvent) => {
  if (e.pointerType !== 'mouse') return
  hoverMs.value = timeAtPointer(e.clientX)
}

// Keep the playhead visible when zoomed in.
watch(positionMs, (ms) => {
  if (ms == null || zoomHours.value >= 24 || !date.value) return
  if (ms < viewStartMs.value || ms > viewEndMs.value) viewStartMs.value = clampView(ms - spanMs.value / 2)
})

// ---- loading -------------------------------------------------------------------------

const stopVideo = () => {
  const v = videoEl.value
  current.value = null
  positionMs.value = null
  notice.value = null
  if (!v) return
  try {
    v.pause()
    v.removeAttribute('src')
    v.load()
  } catch {}
}

const syncQuery = () => {
  if (route.query.camera === cameraId.value && route.query.t === undefined) return
  router.replace({ query: { camera: cameraId.value } })
}

const describe = (e: any) => e?.data?.error || (e?.statusCode ? `HTTP ${e.statusCode}` : 'server unreachable')

const loadFiles = async (): Promise<boolean> => {
  if (!cameraId.value || !date.value) return false
  const seq = ++filesSeq
  loadingFiles.value = true
  filesError.value = ''
  try {
    // Start a little before midnight so a file spanning midnight is included.
    const list = await fetchFiles(cameraId.value, new Date(dayStartMs.value - 10 * 60e3), new Date(dayEndMs.value))
    if (seq !== filesSeq) return false
    files.value = list.filter((f) => f.endMs > dayStartMs.value && f.startMs < dayEndMs.value)
    return true
  } catch (e) {
    if (seq !== filesSeq) return false
    files.value = []
    filesError.value = `Could not load recordings: ${describe(e)}`
    return false
  } finally {
    if (seq === filesSeq) loadingFiles.value = false
  }
}

const selectDate = async (key: string, target?: number) => {
  if (!key) return
  if (key !== date.value) {
    date.value = key
    files.value = []
    zoomHours.value = target != null ? zoomHours.value : 24
    viewStartMs.value = dayStartMs.value
  }
  if (target != null && zoomHours.value < 24) viewStartMs.value = clampView(target - spanMs.value / 2)
  const ok = await loadFiles()
  if (ok && target != null && date.value === key) playAt(target)
}

const selectCamera = async (id: string, target?: number) => {
  if (!id) return
  const seq = ++daysSeq
  if (id !== cameraId.value) {
    cameraId.value = id
    stopVideo()
    days.value = []
    files.value = []
    date.value = ''
  }
  try {
    localStorage.setItem(CAMERA_KEY, id)
  } catch {}
  syncQuery()

  loadingDays.value = true
  try {
    const list = await fetchDays(id)
    if (seq !== daysSeq) return
    days.value = list
  } catch (e) {
    if (seq !== daysSeq) return
    days.value = []
    filesError.value = `Could not load recording days: ${describe(e)}`
  } finally {
    if (seq === daysSeq) loadingDays.value = false
  }

  const key = target != null ? localDateKey(new Date(target)) : date.value || days.value[0]?.date
  if (key) await selectDate(key, target)
}

// ---- playback ------------------------------------------------------------------------

const playFile = (file: TimedFile, offsetSecs = 0, opts: { autoplay?: boolean; retry?: boolean } = {}) => {
  const v = videoEl.value
  if (!v) return
  if (!file.complete) {
    showInProgress(file)
    return
  }
  notice.value = null
  current.value = file
  if (!opts.retry) retriedCurrent = false
  pendingSeek = Math.max(0, offsetSecs)
  pendingAutoplay = opts.autoplay ?? true
  positionMs.value = file.startMs + pendingSeek * 1000
  v.src = mediaUrl(file)
  v.load()
  nextTick(() => {
    listEl.value?.querySelector(`[data-file="${CSS.escape(file.filename)}"]`)?.scrollIntoView({ block: 'nearest' })
  })
}

/** Play whatever covers `ms`; otherwise the nearest following (or last) segment. */
const playAt = (ms: number) => {
  const list = files.value
  if (!list.length) {
    toast.info('No recordings on this day')
    return
  }
  const cover = list.find((f) => f.startMs <= ms && ms < f.endMs)
  if (cover) {
    playFile(cover, (ms - cover.startMs) / 1000)
    return
  }
  const next = list.find((f) => f.startMs > ms)
  if (next && !next.complete) {
    showInProgress(next)
    return
  }
  if (next) {
    if (next.startMs - ms > 15_000) toast.info(`No recording at ${formatClock(ms)}; playing from ${formatClock(next.startMs)}`)
    playFile(next)
    return
  }
  const last = lastComplete.value
  if (!last) {
    toast.info('No finished recordings on this day yet')
    return
  }
  toast.info(`Latest recording ends at ${formatClock(last.endMs)}`)
  playFile(last, Math.max(0, (last.endMs - last.startMs) / 1000 - 15))
}

const showInProgress = (file: TimedFile) => {
  videoEl.value?.pause()
  notice.value = `The recording from ${formatClock(file.startMs)} is still being written and becomes playable once that file closes (a few minutes). Watch live for what is happening now.`
}

const playLatestFinished = () => {
  const last = lastComplete.value
  if (last) playFile(last, Math.max(0, (last.endMs - last.startMs) / 1000 - 30))
}

const jumpTo = async (ms: number) => {
  if (!cameraId.value) return
  const key = localDateKey(new Date(ms))
  if (key !== date.value) await selectDate(key, ms)
  else playAt(ms)
}

/** Auto-advance: next segment, newly written segments, then the next day. */
const advance = async () => {
  const cur = current.value
  if (!cur) return
  let next = files.value.find((f) => f.startMs > cur.startMs)
  if (!next) {
    await loadFiles()
    next = files.value.find((f) => f.startMs > cur.startMs)
  }
  if (!next && newerDay.value) {
    await selectDate(newerDay.value)
    next = files.value.find((f) => f.startMs > cur.startMs)
  }
  if (next && !next.complete) {
    // Caught up with the file ffmpeg is writing right now.
    showInProgress(next)
  } else if (next) {
    playFile(next)
  } else {
    toast.info('End of recordings')
  }
}

const skip = (secs: number) => {
  const v = videoEl.value
  if (!v || !current.value) return
  const t = v.currentTime + secs
  if (t < 0 && prevFile.value) {
    const p = prevFile.value
    playFile(p, Math.max(0, (p.endMs - p.startMs) / 1000 + t))
  } else if (Number.isFinite(v.duration) && t > v.duration && nextFile.value) {
    playFile(nextFile.value, t - v.duration)
  } else {
    v.currentTime = Math.max(0, Number.isFinite(v.duration) ? Math.min(t, v.duration - 0.1) : t)
  }
}

const onLoadedMetadata = () => {
  const v = videoEl.value
  if (!v) return
  if (pendingSeek > 0) {
    v.currentTime = Number.isFinite(v.duration) ? Math.min(pendingSeek, Math.max(0, v.duration - 0.5)) : pendingSeek
  }
  pendingSeek = 0
  v.defaultPlaybackRate = speed.value
  v.playbackRate = speed.value
  if (pendingAutoplay) v.play().catch(() => {})
}

const onTimeUpdate = () => {
  const v = videoEl.value
  if (v && current.value) positionMs.value = current.value.startMs + v.currentTime * 1000
}

const onVideoError = () => {
  const v = videoEl.value
  const cur = current.value
  if (!v || !cur || !v.getAttribute('src')) return
  if (!retriedCurrent) {
    // Most likely the token in the URL expired; reload with a fresh one where we were.
    retriedCurrent = true
    const at = positionMs.value != null ? (positionMs.value - cur.startMs) / 1000 : 0
    playFile(cur, at, { retry: true })
    return
  }
  toast.error(`Cannot play segment ${formatClock(cur.startMs)} (file missing or still being written?)`)
}

const onRateChange = () => {
  const v = videoEl.value
  if (v && SPEEDS.includes(v.playbackRate) && v.playbackRate !== speed.value) speed.value = v.playbackRate
}

watch(speed, (s) => {
  const v = videoEl.value
  if (!v) return
  v.defaultPlaybackRate = s
  v.playbackRate = s
})

// Opened from elsewhere (e.g. the live view's "Recordings" button) while already on this page.
watch(
  () => [route.query.camera, route.query.t] as const,
  ([cam, t]) => {
    if (!camerasLoaded.value || typeof cam !== 'string' || !cam) return
    const target = typeof t === 'string' ? Date.parse(t) : NaN
    if (cam === cameraId.value && !Number.isFinite(target)) return
    selectCamera(cam, Number.isFinite(target) ? target : undefined)
  },
)

// ---- startup ---------------------------------------------------------------------------

const loadThumbs = async () => {
  for (const cam of cameras.value.slice(0, 24)) {
    const url = await getSnapshot(cam.id)
    if (url) thumbs[cam.id] = url
  }
}

let nowTimer: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  nowTimer = setInterval(() => (nowMs.value = Date.now()), 30_000)
  try {
    const list = await useApi<Cam[]>('/api/cameras')
    cameras.value = Array.isArray(list) ? list : []
  } catch (e) {
    camerasError.value = `Could not load cameras: ${describe(e)}`
  }
  camerasLoaded.value = true
  if (!cameras.value.length) return

  const qCam = typeof route.query.camera === 'string' ? route.query.camera : ''
  const qT = typeof route.query.t === 'string' ? Date.parse(route.query.t) : NaN
  let saved = ''
  try {
    saved = localStorage.getItem(CAMERA_KEY) || ''
  } catch {}
  const initial =
    cameras.value.find((c) => c.id === qCam) ?? cameras.value.find((c) => c.id === saved) ?? cameras.value[0]

  await nextTick() // video element is rendered once cameras exist
  selectCamera(initial.id, Number.isFinite(qT) ? qT : undefined)
  loadThumbs()
})

onBeforeUnmount(() => {
  if (nowTimer) clearInterval(nowTimer)
  stopVideo()
})
</script>
