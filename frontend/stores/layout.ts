import { defineStore } from 'pinia'

interface GridLayout {
  id: string
  name: string
  layout_type: string
  camera_positions: any[]
  is_default: boolean
}

export const useLayoutStore = defineStore('layout', {
  state: () => ({
    layouts: [] as GridLayout[],
    currentCols: 2,
  }),

  actions: {
    async fetchAll() {
      try {
        this.layouts = await useApi<GridLayout[]>('/api/layouts')
      } catch {}
    },

    async save(name: string, cols: number) {
      const layout = await useApi<GridLayout>('/api/layouts', {
        method: 'POST',
        body: { name, layout_type: 'grid', camera_positions: [], is_default: false },
      })
      this.layouts.push(layout)
      return layout
    },

    async remove(id: string) {
      await useApi(`/api/layouts/${id}`, { method: 'DELETE' })
      this.layouts = this.layouts.filter(l => l.id !== id)
    },

    setCols(cols: number) {
      this.currentCols = cols
    },
  },
})
