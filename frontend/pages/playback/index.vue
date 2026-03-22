<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">Playback</h1>

    <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
      <!-- Controls -->
      <div class="bg-nvr-card rounded-lg p-4 border border-nvr-border">
        <h2 class="text-sm font-semibold text-gray-400 mb-3">Camera</h2>
        <select v-model="selectedCamera" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white text-sm mb-4">
          <option value="">Select camera...</option>
          <option v-for="cam in cameras" :key="cam.id" :value="cam.id">{{ cam.name }}</option>
        </select>

        <h2 class="text-sm font-semibold text-gray-400 mb-3">Date Range</h2>
        <div class="space-y-2 mb-4">
          <input v-model="startDate" type="datetime-local" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white text-sm" />
          <input v-model="endDate" type="datetime-local" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-2 text-white text-sm" />
        </div>

        <button @click="loadTimeline" :disabled="!selectedCamera" class="w-full bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white py-2 rounded-lg text-sm transition-colors">
          Load Timeline
        </button>

        <h2 class="text-sm font-semibold text-gray-400 mt-6 mb-3">Speed</h2>
        <div class="flex gap-2">
          <button v-for="speed in [0.5, 1, 2, 4, 8]" :key="speed"
            @click="playbackSpeed = speed"
            :class="playbackSpeed === speed ? 'bg-primary-600' : 'bg-nvr-darker'"
            class="flex-1 py-1 rounded text-xs border border-nvr-border">
            {{ speed }}x
          </button>
        </div>
      </div>

      <!-- Video + Timeline -->
      <div class="lg:col-span-3 space-y-4">
        <div class="bg-nvr-card rounded-lg border border-nvr-border overflow-hidden">
          <div class="aspect-video bg-black flex items-center justify-center">
            <video ref="playbackVideo" controls class="w-full h-full object-contain" />
            <div v-if="!currentSegmentUrl" class="absolute text-gray-500 text-sm">
              Select a recording to play
            </div>
          </div>
        </div>

        <!-- Timeline -->
        <div class="bg-nvr-card rounded-lg p-4 border border-nvr-border">
          <h2 class="text-sm font-semibold text-gray-400 mb-3">Timeline</h2>
          <div v-if="!timeline" class="text-gray-500 text-sm">Load a timeline first</div>
          <div v-else>
            <div class="relative h-12 bg-nvr-darker rounded overflow-hidden">
              <!-- Recording blocks -->
              <div v-for="rec in timeline.recordings" :key="rec.recording_id"
                @click="loadRecording(rec.recording_id)"
                class="absolute top-0 h-full cursor-pointer hover:opacity-80 transition-opacity"
                :class="rec.recording_type === 'continuous' ? 'bg-primary-600/50' : 'bg-yellow-600/50'"
                :style="timelineStyle(rec)">
              </div>
              <!-- Event markers -->
              <div v-for="evt in timeline.events" :key="evt.id"
                class="absolute top-0 w-1 h-full bg-red-500"
                :style="{ left: timelinePosition(evt.occurred_at) + '%' }"
                :title="`${evt.event_type} (${evt.confidence?.toFixed(2) || 'N/A'})`">
              </div>
            </div>
            <div class="flex justify-between text-xs text-gray-500 mt-1">
              <span>{{ formatDateTime(startDate) }}</span>
              <span>{{ formatDateTime(endDate) }}</span>
            </div>
          </div>
        </div>

        <!-- Recordings list -->
        <div v-if="recordings.length > 0" class="bg-nvr-card rounded-lg p-4 border border-nvr-border">
          <h2 class="text-sm font-semibold text-gray-400 mb-3">Recordings ({{ recordings.length }})</h2>
          <div class="space-y-2 max-h-64 overflow-y-auto">
            <div v-for="rec in recordings" :key="rec.id"
              @click="loadRecording(rec.id)"
              class="flex justify-between items-center p-2 rounded cursor-pointer hover:bg-nvr-darker transition-colors"
              :class="currentRecordingId === rec.id ? 'bg-primary-600/20 border border-primary-500' : ''">
              <div>
                <span class="text-sm">{{ formatDateTime(rec.start_time) }}</span>
                <span class="text-xs text-gray-400 ml-2">{{ rec.recording_type }}</span>
              </div>
              <div class="text-xs text-gray-500">
                {{ rec.duration_secs ? formatDuration(rec.duration_secs) : 'Recording...' }}
                <span v-if="rec.has_audio" class="ml-1 text-primary-400" title="Has audio">&#9835;</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const cameras = ref<any[]>([])
const selectedCamera = ref('')
const startDate = ref('')
const endDate = ref('')
const playbackSpeed = ref(1)
const timeline = ref<any>(null)
const recordings = ref<any[]>([])
const currentRecordingId = ref('')
const currentSegmentUrl = ref('')
const playbackVideo = ref<HTMLVideoElement | null>(null)

onMounted(async () => {
  try {
    cameras.value = await useApi<any[]>('/api/cameras')
  } catch {}

  // Default to last 24 hours
  const now = new Date()
  const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000)
  endDate.value = now.toISOString().slice(0, 16)
  startDate.value = yesterday.toISOString().slice(0, 16)
})

watch(playbackSpeed, (speed) => {
  if (playbackVideo.value) {
    playbackVideo.value.playbackRate = speed
  }
})

const loadTimeline = async () => {
  if (!selectedCamera.value) return
  try {
    const start = new Date(startDate.value).toISOString()
    const end = new Date(endDate.value).toISOString()
    timeline.value = await useApi<any>(`/api/timeline?camera_id=${selectedCamera.value}&start=${start}&end=${end}`)
    recordings.value = await useApi<any[]>(`/api/recordings?camera_id=${selectedCamera.value}&start=${start}&end=${end}`)
  } catch (e) {
    console.error('Failed to load timeline', e)
  }
}

const loadRecording = async (recordingId: string) => {
  currentRecordingId.value = recordingId
  try {
    const segments = await useApi<any[]>(`/api/recordings/${recordingId}/segments`)
    if (segments.length > 0 && segments[0].download_url) {
      currentSegmentUrl.value = segments[0].download_url
      if (playbackVideo.value) {
        playbackVideo.value.src = segments[0].download_url
        playbackVideo.value.playbackRate = playbackSpeed.value
        playbackVideo.value.play()
      }
    }
  } catch (e) {
    console.error('Failed to load segments', e)
  }
}

const timelineStyle = (rec: any) => {
  const start = new Date(startDate.value).getTime()
  const end = new Date(endDate.value).getTime()
  const total = end - start
  const recStart = new Date(rec.start_time).getTime()
  const recEnd = rec.end_time ? new Date(rec.end_time).getTime() : Date.now()
  const left = Math.max(0, (recStart - start) / total * 100)
  const width = Math.min(100 - left, (recEnd - recStart) / total * 100)
  return { left: `${left}%`, width: `${width}%` }
}

const timelinePosition = (time: string) => {
  const start = new Date(startDate.value).getTime()
  const end = new Date(endDate.value).getTime()
  return Math.max(0, Math.min(100, (new Date(time).getTime() - start) / (end - start) * 100))
}

const formatDateTime = (dt: string) => {
  if (!dt) return ''
  return new Date(dt).toLocaleString()
}

const formatDuration = (secs: number) => {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = secs % 60
  if (h > 0) return `${h}h${m}m`
  if (m > 0) return `${m}m${s}s`
  return `${s}s`
}
</script>
