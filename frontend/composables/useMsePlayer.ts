/**
 * Low-latency live player: fMP4 over WebSocket into Media Source Extensions.
 *
 * Speaks go2rtc's MSE protocol (the backend proxies `/api/live/{id}/ws` to
 * go2rtc), mirroring what go2rtc's own `video-rtc.js` does:
 *   1. client -> `{"type":"mse","value":"<codecs the browser supports>"}`
 *   2. server -> `{"type":"mse","value":"video/mp4; codecs=\"...\""}`
 *   3. server -> binary fMP4 chunks, appended to one SourceBuffer
 *   `{"type":"error","value":"..."}` reports a failure.
 *
 * Latency is held under ~1 s by jumping to the live edge whenever playback
 * falls behind, instead of letting the buffer grow like native HLS does.
 */

export type LiveQuality = 'main' | 'sub'
export type MseState = 'idle' | 'connecting' | 'playing' | 'error' | 'unsupported'

/** Candidate codecs, identical to go2rtc's video-rtc.js list. */
const CODECS = [
  'avc1.640029', // H.264 high 4.1
  'avc1.64002A', // H.264 high 4.2
  'avc1.640033', // H.264 high 5.1
  'hvc1.1.6.L153.B0', // H.265 main 5.1
  'mp4a.40.2', // AAC LC
  'mp4a.40.5', // AAC HE
  'flac',
  'opus',
]

/** Jump to the live edge when playback lags the newest buffered frame by more than this. */
const MAX_LAG_SECS = 1.0
/** Where to land after a jump, relative to the live edge. */
const LIVE_EDGE_OFFSET_SECS = 0.3
/** Buffered media older than this (relative to the live edge) is dropped. */
const KEEP_BUFFER_SECS = 10
/** Give up on a connection that has not produced data for this long. */
const CONNECT_TIMEOUT_MS = 15000
const STALL_TIMEOUT_MS = 10000
/** A client that cannot keep up (hidden tab, slow device) is reset rather than buffering forever. */
const MAX_PENDING_BYTES = 16 * 1024 * 1024
const MIN_RETRY_MS = 1000
const MAX_RETRY_MS = 30000

type MediaSourceCtor = typeof MediaSource

/** iOS 17+ only exposes ManagedMediaSource; everything else uses MediaSource. */
const getMediaSourceCtor = (): { ctor: MediaSourceCtor; managed: boolean } | null => {
  if (!import.meta.client) return null
  const w = window as unknown as { ManagedMediaSource?: MediaSourceCtor; MediaSource?: MediaSourceCtor }
  if (w.ManagedMediaSource) return { ctor: w.ManagedMediaSource, managed: true }
  if (w.MediaSource) return { ctor: w.MediaSource, managed: false }
  return null
}

export const isMseSupported = () => getMediaSourceCtor() !== null

const supportedCodecs = (ctor: MediaSourceCtor): string =>
  CODECS.filter((codec) => {
    try {
      return ctor.isTypeSupported(`video/mp4; codecs="${codec}"`)
    } catch {
      return false
    }
  }).join(',')

export interface MsePlayerOptions {
  video: Ref<HTMLVideoElement | null>
  cameraId: () => string
  quality: () => LiveQuality
}

export const useMsePlayer = (opts: MsePlayerOptions) => {
  const { apiUrl } = useServerConfig()
  const { token } = useAuth()

  const state = ref<MseState>('idle')
  const error = ref('')
  /** True once the server announced an audio codec for this stream. */
  const hasAudio = ref(false)

  let ws: WebSocket | null = null
  let ms: MediaSource | null = null
  let sb: SourceBuffer | null = null
  let objectUrl = ''
  let pending: Uint8Array[] = []
  let pendingBytes = 0
  /** Bumped on every teardown so callbacks from an old connection are ignored. */
  let generation = 0
  let retryTimer: ReturnType<typeof setTimeout> | null = null
  let tickTimer: ReturnType<typeof setInterval> | null = null
  let retryDelay = MIN_RETRY_MS
  let failures = 0
  let connectedAt = 0
  let lastDataAt = 0
  /** Whether the owner wants the stream running (false after stop()). */
  let wanted = false
  let videoListenersOn: HTMLVideoElement | null = null

  const buildUrl = () => {
    const base = apiUrl.value.replace(/^http/i, 'ws')
    const params = new URLSearchParams({ quality: opts.quality() })
    if (token.value) params.set('access_token', token.value)
    return `${base}/api/live/${encodeURIComponent(opts.cameraId())}/ws?${params}`
  }

  // ---- video element events -------------------------------------------------

  const onPlaying = () => {
    if (!wanted) return
    state.value = 'playing'
    error.value = ''
    failures = 0
    retryDelay = MIN_RETRY_MS
  }

  const attachVideoListeners = (video: HTMLVideoElement) => {
    if (videoListenersOn === video) return
    detachVideoListeners()
    video.addEventListener('playing', onPlaying)
    videoListenersOn = video
  }

  const detachVideoListeners = () => {
    videoListenersOn?.removeEventListener('playing', onPlaying)
    videoListenersOn = null
  }

  // ---- buffer management ----------------------------------------------------

  const appendNow = (data: Uint8Array) => {
    if (!sb) return
    try {
      sb.appendBuffer(data as BufferSource)
    } catch (e: any) {
      if (e?.name === 'QuotaExceededError') {
        // Drop everything but the newest data and try again on next updateend.
        pending.unshift(data)
        pendingBytes += data.byteLength
        trimBuffer(true)
        return
      }
      reconnect(`Playback error: ${e?.message || e}`)
    }
  }

  /** Appends everything queued as a single chunk (fewer, larger appends). */
  const flushPending = () => {
    if (!sb || sb.updating || pending.length === 0) return
    let data: Uint8Array
    if (pending.length === 1) {
      data = pending[0]
    } else {
      data = new Uint8Array(pendingBytes)
      let offset = 0
      for (const chunk of pending) {
        data.set(chunk, offset)
        offset += chunk.byteLength
      }
    }
    pending = []
    pendingBytes = 0
    appendNow(data)
  }

  const onBinary = (buf: ArrayBuffer) => {
    lastDataAt = Date.now()
    const chunk = new Uint8Array(buf)
    if (!sb || sb.updating || pending.length > 0) {
      pending.push(chunk)
      pendingBytes += chunk.byteLength
      if (pendingBytes > MAX_PENDING_BYTES) reconnect('Player fell behind, reconnecting')
      return
    }
    appendNow(chunk)
  }

  const trimBuffer = (aggressive = false) => {
    if (!sb || sb.updating) return
    const buffered = sb.buffered
    if (!buffered.length) return
    const start = buffered.start(0)
    const end = buffered.end(buffered.length - 1)
    const keep = aggressive ? 2 : KEEP_BUFFER_SECS
    if (end - start > keep + 2) {
      try {
        sb.remove(start, end - keep)
      } catch {}
    }
  }

  /** Keeps the playhead close to the newest frame. */
  const chaseLiveEdge = () => {
    const video = opts.video.value
    if (!video || !sb) return
    const buffered = video.buffered
    if (!buffered.length) return
    const lastStart = buffered.start(buffered.length - 1)
    const end = buffered.end(buffered.length - 1)
    const lag = end - video.currentTime
    if (video.currentTime < lastStart || lag > MAX_LAG_SECS) {
      video.currentTime = Math.max(lastStart, end - LIVE_EDGE_OFFSET_SECS)
    }
  }

  const onUpdateEnd = () => {
    if (pending.length) {
      flushPending()
      return
    }
    trimBuffer()
    chaseLiveEdge()
  }

  // ---- protocol ---------------------------------------------------------------

  const onText = (text: string) => {
    let msg: { type?: string; value?: unknown }
    try {
      msg = JSON.parse(text)
    } catch {
      return
    }
    if (msg.type === 'mse') {
      const mime = String(msg.value || '')
      if (!ms || ms.readyState !== 'open' || sb) return
      try {
        sb = ms.addSourceBuffer(mime)
        sb.mode = 'segments'
        sb.addEventListener('updateend', onUpdateEnd)
      } catch (e: any) {
        fail(`Browser cannot play this stream (${mime})`)
        return
      }
      hasAudio.value = /mp4a|opus|flac/i.test(mime)
      flushPending()
    } else if (msg.type === 'error') {
      reconnect(String(msg.value || 'Stream error'))
    }
  }

  // ---- lifecycle --------------------------------------------------------------

  const clearRetry = () => {
    if (retryTimer) {
      clearTimeout(retryTimer)
      retryTimer = null
    }
  }

  const teardown = () => {
    generation++
    clearRetry()
    if (tickTimer) {
      clearInterval(tickTimer)
      tickTimer = null
    }
    if (ws) {
      ws.onopen = ws.onmessage = ws.onclose = ws.onerror = null
      try {
        ws.close()
      } catch {}
      ws = null
    }
    if (sb) {
      sb.removeEventListener('updateend', onUpdateEnd)
      sb = null
    }
    if (ms && ms.readyState === 'open') {
      try {
        ms.endOfStream()
      } catch {}
    }
    ms = null
    const video = opts.video.value
    if (video) {
      try {
        video.pause()
        video.removeAttribute('src')
        video.srcObject = null
        video.load()
      } catch {}
    }
    if (objectUrl) {
      URL.revokeObjectURL(objectUrl)
      objectUrl = ''
    }
    pending = []
    pendingBytes = 0
  }

  /** Unrecoverable in this browser; no automatic retry. */
  const fail = (message: string) => {
    teardown()
    state.value = 'error'
    error.value = message
  }

  /** Drop the connection and try again with exponential backoff. */
  const reconnect = (message: string) => {
    teardown()
    if (!wanted) return
    failures++
    // One quiet retry after a drop; show the error once it keeps failing.
    state.value = failures > 1 ? 'error' : 'connecting'
    error.value = message
    retryTimer = setTimeout(connect, retryDelay)
    retryDelay = Math.min(retryDelay * 2, MAX_RETRY_MS)
  }

  const tick = () => {
    const video = opts.video.value
    if (!video || !ws) return
    const now = Date.now()
    if (!lastDataAt && now - connectedAt > CONNECT_TIMEOUT_MS) {
      reconnect('No video from camera (timeout)')
      return
    }
    if (lastDataAt && now - lastDataAt > STALL_TIMEOUT_MS) {
      reconnect('Stream stalled, reconnecting')
      return
    }
    // Safety net: data queued while no updateend is due would otherwise sit forever.
    if (sb && !sb.updating && pending.length) flushPending()
    chaseLiveEdge()
    if (video.paused && video.buffered.length) video.play().catch(() => {})
  }

  const connect = () => {
    teardown()
    if (!wanted) return
    const video = opts.video.value
    if (!video) return
    const mse = getMediaSourceCtor()
    if (!mse) {
      state.value = 'unsupported'
      error.value = 'This browser cannot play live video (no Media Source Extensions)'
      return
    }
    if (import.meta.client && document.hidden) {
      // Resumed by the visibilitychange handler.
      state.value = 'idle'
      return
    }

    const gen = generation
    if (state.value !== 'error') state.value = 'connecting'
    connectedAt = Date.now()
    lastDataAt = 0
    hasAudio.value = false
    attachVideoListeners(video)

    let socket: WebSocket
    try {
      socket = new WebSocket(buildUrl())
    } catch (e: any) {
      reconnect(`Cannot open live connection: ${e?.message || e}`)
      return
    }
    socket.binaryType = 'arraybuffer'
    ws = socket

    socket.onopen = () => {
      if (gen !== generation) return
      const source = new mse.ctor()
      ms = source
      source.addEventListener(
        'sourceopen',
        () => {
          if (gen !== generation || socket.readyState !== WebSocket.OPEN) return
          socket.send(JSON.stringify({ type: 'mse', value: supportedCodecs(mse.ctor) }))
        },
        { once: true },
      )
      if (mse.managed) {
        video.disableRemotePlayback = true
        video.srcObject = source
      } else {
        objectUrl = URL.createObjectURL(source)
        video.srcObject = null
        video.src = objectUrl
      }
      video.play().catch(() => {})
    }
    socket.onmessage = (ev) => {
      if (gen !== generation) return
      if (typeof ev.data === 'string') onText(ev.data)
      else if (ev.data instanceof ArrayBuffer) onBinary(ev.data)
    }
    socket.onclose = () => {
      if (gen !== generation) return
      reconnect(lastDataAt ? 'Connection lost' : 'Cannot connect to live stream')
    }
    socket.onerror = () => {
      // A close event always follows; reconnect happens there.
    }

    tickTimer = setInterval(tick, 1000)
  }

  const onVisibility = () => {
    if (!wanted) return
    if (document.hidden) {
      // No point decoding video nobody sees; it also keeps the buffer from piling up.
      teardown()
      state.value = 'idle'
    } else {
      failures = 0
      retryDelay = MIN_RETRY_MS
      connect()
    }
  }

  /** Start (or restart) the stream, resetting the backoff. */
  const start = () => {
    wanted = true
    failures = 0
    retryDelay = MIN_RETRY_MS
    error.value = ''
    state.value = 'connecting'
    connect()
  }

  const stop = () => {
    wanted = false
    teardown()
    detachVideoListeners()
    state.value = 'idle'
  }

  if (import.meta.client) {
    document.addEventListener('visibilitychange', onVisibility)
  }

  onBeforeUnmount(() => {
    if (import.meta.client) document.removeEventListener('visibilitychange', onVisibility)
    stop()
  })

  return { state, error, hasAudio, start, stop }
}
