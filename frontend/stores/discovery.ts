import { defineStore } from 'pinia'

interface DiscoveredDevice {
  ip: string
  mac: string | null
  brand: string | null
  model: string | null
  name: string | null
  protocols: string[]
  rtsp_url: string | null
  onvif_url: string | null
  http_url: string | null
  // UI state
  showEdit?: boolean
  editName?: string
  editBrand?: string
  editUrl?: string
  editProtocol?: string
  editUsername?: string
  editPassword?: string
}

export const useDiscoveryStore = defineStore('discovery', {
  state: () => ({
    devices: [] as DiscoveredDevice[],
    scanning: false,
    lastScanAt: null as string | null,
    subnets: '',
    pingResults: [] as any[],
  }),

  getters: {
    hasResults: (state) => state.devices.length > 0,
    cameraDevices: (state) => state.devices.filter(d =>
      d.protocols.length > 0 || d.brand !== null
    ),
  },

  actions: {
    setDevices(devices: any[]) {
      this.devices = devices.map(d => ({
        ...d,
        showEdit: false,
        editName: d.name || `Camera ${d.ip}`,
        editBrand: d.brand || '',
        editUrl: d.rtsp_url || `rtsp://${d.ip}:554/stream1`,
        editProtocol: d.protocols?.[0] || 'rtsp',
        editUsername: '',
        editPassword: '',
      }))
      this.lastScanAt = new Date().toISOString()
    },

    removeDevice(ip: string) {
      this.devices = this.devices.filter(d => d.ip !== ip)
    },

    setScanning(v: boolean) {
      this.scanning = v
    },

    setSubnets(s: string) {
      this.subnets = s
    },

    setPingResults(results: any[]) {
      this.pingResults = results
    },

    clear() {
      this.devices = []
      this.lastScanAt = null
    },
  },
})
