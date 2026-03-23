/**
 * Global HLS stream cache — persists across component mount/unmount.
 * Prevents re-requesting HLS start on every layout change.
 */
const hlsStarted = new Set<string>()
const snapshotCache = new Map<string, { url: string; timestamp: number }>()

export const useHlsCache = () => {
  const markStarted = (cameraId: string) => hlsStarted.add(cameraId)
  const isStarted = (cameraId: string) => hlsStarted.has(cameraId)

  const cacheSnapshot = (cameraId: string, blobUrl: string) => {
    const old = snapshotCache.get(cameraId)
    if (old) URL.revokeObjectURL(old.url)
    snapshotCache.set(cameraId, { url: blobUrl, timestamp: Date.now() })
  }

  const getCachedSnapshot = (cameraId: string): string | null => {
    const cached = snapshotCache.get(cameraId)
    if (cached && Date.now() - cached.timestamp < 30000) return cached.url
    return null
  }

  return { markStarted, isStarted, cacheSnapshot, getCachedSnapshot }
}
