/**
 * Continuous-recording archive: ffmpeg MP4 files (~5 min each) on the server's
 * disk, exposed by `/api/recordings/files/...`.
 */

export interface RecordingDay {
  /** Local date (in the tz offset we asked for), YYYY-MM-DD. */
  date: string
  count: number
  size_bytes: number
}

export interface RecordingFile {
  filename: string
  start: string
  end: string
  duration_secs: number
  size_bytes: number
  url: string
  /**
   * False for the file ffmpeg is still writing: its MP4 index is only written
   * on close, so it cannot be played yet. Missing is treated as complete.
   */
  complete?: boolean
}

/** A file with parsed times, ready for timeline math. */
export interface TimedFile extends RecordingFile {
  startMs: number
  endMs: number
  complete: boolean
}

const pad = (n: number) => String(n).padStart(2, '0')

/** YYYY-MM-DD of a moment in the browser's local time. */
export const localDateKey = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`

/** Local midnight at the start of a YYYY-MM-DD date. */
export const localDayStart = (key: string): Date => {
  const [y, m, d] = key.split('-').map(Number)
  return new Date(y, (m || 1) - 1, d || 1)
}

/** Local midnight at the start of the following day (DST-safe, unlike +24h). */
export const localDayEnd = (key: string): Date => {
  const start = localDayStart(key)
  return new Date(start.getFullYear(), start.getMonth(), start.getDate() + 1)
}

export const shiftDateKey = (key: string, days: number) => {
  const start = localDayStart(key)
  return localDateKey(new Date(start.getFullYear(), start.getMonth(), start.getDate() + days))
}

export const formatBytes = (bytes: number) => {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)))
  return `${(bytes / 1024 ** i).toFixed(i >= 2 ? 1 : 0)} ${units[i]}`
}

export const formatClock = (ms: number, withSeconds = false) =>
  new Date(ms).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    ...(withSeconds ? { second: '2-digit' } : {}),
    hour12: false,
  })

const toTimed = (f: RecordingFile): TimedFile | null => {
  if (!f || typeof f.url !== 'string') return null
  const startMs = Date.parse(f.start)
  if (!Number.isFinite(startMs)) return null
  let endMs = Date.parse(f.end)
  if (!Number.isFinite(endMs) || endMs <= startMs) {
    endMs = startMs + Math.max(1, Number(f.duration_secs) || 0) * 1000
  }
  const complete = f.complete !== false
  // A file being written may report no/partial end; it covers up to "now".
  if (!complete) endMs = Math.max(endMs, Math.min(Date.now(), startMs + 30 * 60e3))
  return { ...f, startMs, endMs, complete }
}

export const useRecordingFiles = () => {
  const { apiUrl } = useServerConfig()
  const { token } = useAuth()

  const fetchDays = async (cameraId: string): Promise<RecordingDay[]> => {
    const tz = -new Date().getTimezoneOffset()
    const res = await useApi<{ days?: RecordingDay[] }>(
      `/api/recordings/files/${encodeURIComponent(cameraId)}/days?tz_offset_minutes=${tz}`,
    )
    const days = Array.isArray(res?.days) ? res.days.filter((d) => /^\d{4}-\d{2}-\d{2}$/.test(d?.date)) : []
    return days.sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : 0))
  }

  const fetchFiles = async (cameraId: string, from: Date, to: Date): Promise<TimedFile[]> => {
    const params = new URLSearchParams({ from: from.toISOString(), to: to.toISOString() })
    const res = await useApi<{ files?: RecordingFile[] }>(
      `/api/recordings/files/${encodeURIComponent(cameraId)}?${params}`,
    )
    const files = Array.isArray(res?.files) ? res.files : []
    return files
      .map(toTimed)
      .filter((f): f is TimedFile => f !== null)
      .sort((a, b) => a.startMs - b.startMs)
  }

  /** Playable (or downloadable) URL; the token rides in the query since <video> cannot send headers. */
  const mediaUrl = (file: RecordingFile, download = false) => {
    const path = file.url
    const base = /^https?:\/\//i.test(path) ? path : `${apiUrl.value}${path.startsWith('/') ? '' : '/'}${path}`
    const params = new URLSearchParams()
    if (token.value) params.set('access_token', token.value)
    if (download) params.set('download', '1')
    const qs = params.toString()
    if (!qs) return base
    return `${base}${base.includes('?') ? '&' : '?'}${qs}`
  }

  return { fetchDays, fetchFiles, mediaUrl }
}
