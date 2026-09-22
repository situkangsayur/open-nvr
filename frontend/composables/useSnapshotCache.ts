/**
 * One-shot camera snapshots, used only as a placeholder until live video (or
 * nothing at all) is available. Cached across component mounts so switching
 * layouts shows the last picture immediately instead of a black tile.
 */

const CACHE_TTL_MS = 60_000

const cache = new Map<string, { url: string; at: number }>()
const inflight = new Map<string, Promise<string | null>>()

export const useSnapshotCache = () => {
  const { apiUrl } = useServerConfig()
  const { token } = useAuth()

  const getCached = (cameraId: string): string | null => {
    const hit = cache.get(cameraId)
    return hit && Date.now() - hit.at < CACHE_TTL_MS ? hit.url : null
  }

  /** Fetches a fresh snapshot (deduplicated per camera); resolves to an object URL or null. */
  const fetchSnapshot = (cameraId: string): Promise<string | null> => {
    if (!import.meta.client) return Promise.resolve(null)
    const running = inflight.get(cameraId)
    if (running) return running

    const job = (async () => {
      try {
        const headers: Record<string, string> = {}
        if (token.value) headers.Authorization = `Bearer ${token.value}`
        const resp = await fetch(
          `${apiUrl.value}/api/cameras/${encodeURIComponent(cameraId)}/snapshot?t=${Date.now()}`,
          { headers },
        )
        if (!resp.ok || !resp.headers.get('content-type')?.includes('image')) return null
        const url = URL.createObjectURL(await resp.blob())
        // Old URLs are intentionally not revoked: a mounted <img> may still show one.
        // Snapshots are small and replaced rarely, so the leak is bounded.
        cache.set(cameraId, { url, at: Date.now() })
        return url
      } catch {
        return null
      } finally {
        inflight.delete(cameraId)
      }
    })()
    inflight.set(cameraId, job)
    return job
  }

  /** Cached snapshot if fresh, otherwise a new fetch. */
  const getSnapshot = async (cameraId: string) => getCached(cameraId) ?? (await fetchSnapshot(cameraId))

  return { getCached, fetchSnapshot, getSnapshot }
}
