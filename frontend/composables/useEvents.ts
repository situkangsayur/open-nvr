interface NvrEvent {
  type: string
  camera_id?: string
  event_type?: string
  severity?: string
  timestamp: string
  data?: any
}

const events = ref<NvrEvent[]>([])
let ws: WebSocket | null = null

export const useEvents = () => {
  const { apiUrl } = useServerConfig()
  const connected = ref(false)

  const connect = () => {
    if (!import.meta.client || ws) return

    const wsUrl = apiUrl.value.replace(/^http/, 'ws')
    ws = new WebSocket(`${wsUrl}/ws/events`)

    ws.onopen = () => {
      connected.value = true
    }

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        if (data.type !== 'ping') {
          events.value.unshift(data)
          // Keep only last 100 events
          if (events.value.length > 100) {
            events.value = events.value.slice(0, 100)
          }

          // Browser notification for critical events
          if (data.severity === 'critical' && Notification.permission === 'granted') {
            new Notification('Open-NVR Alert', {
              body: `${data.event_type}: ${data.camera_id || 'System'}`,
              icon: '/favicon.ico',
            })
          }
        }
      } catch {}
    }

    ws.onclose = () => {
      connected.value = false
      ws = null
      // Reconnect after 5 seconds
      setTimeout(connect, 5000)
    }
  }

  const requestNotificationPermission = () => {
    if (import.meta.client && 'Notification' in window) {
      Notification.requestPermission()
    }
  }

  return { events, connected, connect, requestNotificationPermission }
}
