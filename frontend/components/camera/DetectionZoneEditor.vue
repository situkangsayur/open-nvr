<template>
  <div class="relative">
    <canvas
      ref="canvas"
      class="w-full cursor-crosshair border border-nvr-border rounded"
      :width="canvasWidth"
      :height="canvasHeight"
      @click="addPoint"
      @mousemove="updateCursor"
    />
    <div class="mt-3 flex gap-2">
      <button @click="clearPoints" class="text-xs bg-nvr-darker border border-nvr-border px-3 py-1 rounded hover:bg-nvr-border transition-colors">Clear</button>
      <button @click="undo" :disabled="points.length === 0" class="text-xs bg-nvr-darker border border-nvr-border px-3 py-1 rounded hover:bg-nvr-border disabled:opacity-50 transition-colors">Undo</button>
      <button @click="saveZone" :disabled="points.length < 3" class="text-xs bg-primary-600 px-3 py-1 rounded hover:bg-primary-700 disabled:opacity-50 transition-colors">Save Zone</button>
    </div>
    <div class="mt-2">
      <input v-model="zoneName" type="text" placeholder="Zone name" class="w-full bg-nvr-darker border border-nvr-border rounded px-3 py-1.5 text-white text-sm" />
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ cameraId: string; existingZones?: any[] }>()
const emit = defineEmits<{ saved: [] }>()

const canvas = ref<HTMLCanvasElement | null>(null)
const canvasWidth = 640
const canvasHeight = 360
const points = ref<{ x: number; y: number }[]>([])
const cursorPos = ref<{ x: number; y: number } | null>(null)
const zoneName = ref('')

const addPoint = (e: MouseEvent) => {
  const rect = canvas.value!.getBoundingClientRect()
  const x = (e.clientX - rect.left) / rect.width
  const y = (e.clientY - rect.top) / rect.height
  points.value.push({ x: Math.max(0, Math.min(1, x)), y: Math.max(0, Math.min(1, y)) })
  draw()
}

const updateCursor = (e: MouseEvent) => {
  const rect = canvas.value!.getBoundingClientRect()
  cursorPos.value = {
    x: (e.clientX - rect.left) / rect.width,
    y: (e.clientY - rect.top) / rect.height,
  }
  draw()
}

const clearPoints = () => { points.value = []; draw() }
const undo = () => { points.value.pop(); draw() }

const draw = () => {
  const ctx = canvas.value?.getContext('2d')
  if (!ctx) return

  ctx.clearRect(0, 0, canvasWidth, canvasHeight)
  ctx.fillStyle = '#0f172a'
  ctx.fillRect(0, 0, canvasWidth, canvasHeight)

  // Draw existing zones
  if (props.existingZones) {
    for (const zone of props.existingZones) {
      ctx.beginPath()
      ctx.strokeStyle = '#475569'
      ctx.fillStyle = 'rgba(71, 85, 105, 0.2)'
      for (let i = 0; i < zone.polygon.length; i++) {
        const p = zone.polygon[i]
        const px = p.x * canvasWidth, py = p.y * canvasHeight
        if (i === 0) ctx.moveTo(px, py)
        else ctx.lineTo(px, py)
      }
      ctx.closePath()
      ctx.fill()
      ctx.stroke()
    }
  }

  // Draw current polygon
  if (points.value.length > 0) {
    ctx.beginPath()
    ctx.strokeStyle = '#3b82f6'
    ctx.fillStyle = 'rgba(59, 130, 246, 0.2)'
    ctx.lineWidth = 2

    for (let i = 0; i < points.value.length; i++) {
      const p = points.value[i]
      const px = p.x * canvasWidth, py = p.y * canvasHeight
      if (i === 0) ctx.moveTo(px, py)
      else ctx.lineTo(px, py)
    }

    if (cursorPos.value && points.value.length > 0) {
      ctx.lineTo(cursorPos.value.x * canvasWidth, cursorPos.value.y * canvasHeight)
    }

    if (points.value.length >= 3) {
      ctx.closePath()
      ctx.fill()
    }
    ctx.stroke()

    // Draw points
    for (const p of points.value) {
      ctx.beginPath()
      ctx.arc(p.x * canvasWidth, p.y * canvasHeight, 4, 0, Math.PI * 2)
      ctx.fillStyle = '#3b82f6'
      ctx.fill()
    }
  }
}

const saveZone = async () => {
  if (points.value.length < 3 || !zoneName.value.trim()) return
  try {
    await useApi(`/api/cameras/${props.cameraId}/zones`, {
      method: 'POST',
      body: {
        name: zoneName.value,
        polygon: points.value,
        detection_types: ['motion'],
        sensitivity: 0.5,
      },
    })
    points.value = []
    zoneName.value = ''
    emit('saved')
  } catch (e) {
    console.error('Failed to save zone', e)
  }
}

onMounted(draw)
</script>
