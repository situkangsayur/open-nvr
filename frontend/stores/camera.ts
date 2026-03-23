import { defineStore } from 'pinia'

interface Camera {
  id: string
  name: string
  brand: string | null
  model: string | null
  protocol_type: string
  stream_url: string
  sub_stream_url: string | null
  onvif_url: string | null
  ptz_capable: boolean
  audio_capable: boolean
  group_id: string | null
  status: string
  connection_type: string
  recording_mode: string
  created_at: string
  updated_at: string
}

export const useCameraStore = defineStore('camera', {
  state: () => ({
    cameras: [] as Camera[],
    loading: false,
    selectedId: null as string | null,
  }),

  getters: {
    selected: (state) => state.cameras.find(c => c.id === state.selectedId),
    online: (state) => state.cameras.filter(c => c.status === 'online'),
    offline: (state) => state.cameras.filter(c => c.status === 'offline'),
    byGroup: (state) => (groupId: string) => state.cameras.filter(c => c.group_id === groupId),
  },

  actions: {
    async fetchAll() {
      this.loading = true
      try {
        this.cameras = await useApi<Camera[]>('/api/cameras')
      } catch (e) {
        console.error('Failed to fetch cameras', e)
      } finally {
        this.loading = false
      }
    },

    async create(data: any) {
      const camera = await useApi<Camera>('/api/cameras', { method: 'POST', body: data })
      this.cameras.push(camera)
      return camera
    },

    async update(id: string, data: any) {
      const camera = await useApi<Camera>(`/api/cameras/${id}`, { method: 'PUT', body: data })
      const idx = this.cameras.findIndex(c => c.id === id)
      if (idx >= 0) this.cameras[idx] = camera
      return camera
    },

    async remove(id: string) {
      await useApi(`/api/cameras/${id}`, { method: 'DELETE' })
      this.cameras = this.cameras.filter(c => c.id !== id)
    },

    async testConnection(id: string) {
      return await useApi<any>(`/api/cameras/${id}/test`, { method: 'POST' })
    },

    select(id: string | null) {
      this.selectedId = id
    },
  },
})
