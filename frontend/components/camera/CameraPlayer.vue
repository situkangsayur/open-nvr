<template>
  <div class="w-full h-full flex items-center justify-center bg-black relative">
    <video ref="videoEl" autoplay muted playsinline class="w-full h-full object-contain" />
    <div v-if="!connected" class="absolute inset-0 flex items-center justify-center">
      <span class="text-gray-500 text-sm">{{ statusMessage }}</span>
    </div>
    <div v-if="connected" class="absolute bottom-2 right-2">
      <span class="text-xs text-green-400 bg-black/60 px-2 py-0.5 rounded">LIVE</span>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ cameraId: string }>()
const videoEl = ref<HTMLVideoElement | null>(null)
const connected = ref(false)
const statusMessage = ref('Connecting...')
const config = useRuntimeConfig()

let ws: WebSocket | null = null
let mediaSource: MediaSource | null = null
let sourceBuffer: SourceBuffer | null = null
let queue: Uint8Array[] = []

onMounted(() => {
  connectStream()
})

onUnmounted(() => {
  cleanup()
})

function cleanup() {
  if (ws) {
    ws.close()
    ws = null
  }
  if (mediaSource && mediaSource.readyState === 'open') {
    try { mediaSource.endOfStream() } catch {}
  }
  mediaSource = null
  sourceBuffer = null
  queue = []
}

function connectStream() {
  statusMessage.value = 'Connecting...'
  const wsUrl = config.public.apiUrl.replace(/^http/, 'ws')

  try {
    ws = new WebSocket(`${wsUrl}/ws/stream/${props.cameraId}`)
    ws.binaryType = 'arraybuffer'

    ws.onopen = () => {
      connected.value = true
      statusMessage.value = ''
      initMediaSource()
    }

    ws.onmessage = (event) => {
      const data = new Uint8Array(event.data)
      if (sourceBuffer && !sourceBuffer.updating) {
        try {
          sourceBuffer.appendBuffer(data)
        } catch {
          queue.push(data)
        }
      } else {
        queue.push(data)
      }
    }

    ws.onclose = () => {
      connected.value = false
      statusMessage.value = 'Disconnected. Reconnecting...'
      setTimeout(connectStream, 3000)
    }

    ws.onerror = () => {
      connected.value = false
      statusMessage.value = 'Connection error'
    }
  } catch {
    statusMessage.value = 'Failed to connect'
  }
}

function initMediaSource() {
  if (!videoEl.value || !window.MediaSource) {
    statusMessage.value = 'MediaSource API not supported'
    return
  }

  mediaSource = new MediaSource()
  videoEl.value.src = URL.createObjectURL(mediaSource)

  mediaSource.addEventListener('sourceopen', () => {
    try {
      sourceBuffer = mediaSource!.addSourceBuffer('video/mp4; codecs="avc1.640028"')
      sourceBuffer.addEventListener('updateend', () => {
        if (queue.length > 0 && sourceBuffer && !sourceBuffer.updating) {
          const next = queue.shift()!
          try {
            sourceBuffer.appendBuffer(next)
          } catch {}
        }
      })
    } catch {
      statusMessage.value = 'Codec not supported'
    }
  })
}
</script>
