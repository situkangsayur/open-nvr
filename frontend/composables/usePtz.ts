/**
 * PTZ commands: `POST /api/cameras/{id}/ptz {action, speed}`.
 *
 * Commands for one camera are sent strictly in order, so a quick press/release
 * can never deliver `stop` before the move it is meant to stop. Failures are
 * shown as a toast (rate limited so a held button does not spam them).
 */

export type PtzAction =
  | 'pan_left'
  | 'pan_right'
  | 'tilt_up'
  | 'tilt_down'
  | 'zoom_in'
  | 'zoom_out'
  | 'stop'
  | 'home'

/** Shared by the on-screen pad and the keyboard shortcuts. */
const speed = ref(0.5)
const chains = new Map<string, Promise<unknown>>()
let lastToastAt = 0

const describeError = (e: any): string => {
  const body = e?.data
  if (body && typeof body === 'object' && body.error) return String(body.error)
  if (typeof body === 'string' && body.trim()) return body.trim().slice(0, 200)
  const status = e?.statusCode ?? e?.status ?? e?.response?.status
  if (status) return `HTTP ${status}`
  return 'server unreachable'
}

export const usePtz = () => {
  const toast = useToast()

  const send = (cameraId: string, action: PtzAction, value = speed.value): Promise<boolean> => {
    const previous = chains.get(cameraId) ?? Promise.resolve()
    const job = previous
      .catch(() => {})
      .then(async () => {
        try {
          await useApi(`/api/cameras/${encodeURIComponent(cameraId)}/ptz`, {
            method: 'POST',
            body: { action, speed: Math.min(1, Math.max(0, value)) },
          })
          return true
        } catch (e) {
          if (Date.now() - lastToastAt > 2500) {
            lastToastAt = Date.now()
            toast.error(`PTZ ${action.replace('_', ' ')} failed: ${describeError(e)}`)
          }
          return false
        }
      })
    chains.set(cameraId, job)
    job.finally(() => {
      if (chains.get(cameraId) === job) chains.delete(cameraId)
    })
    return job
  }

  const move = (cameraId: string, action: PtzAction) => send(cameraId, action)
  const stop = (cameraId: string) => send(cameraId, 'stop', 0)
  const home = (cameraId: string) => send(cameraId, 'home')

  return { speed, send, move, stop, home }
}
