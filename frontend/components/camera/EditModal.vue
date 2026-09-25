<template>
  <div v-if="open" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border w-full max-w-lg max-h-[90vh] overflow-y-auto">
      <h2 class="text-xl font-bold mb-4">Edit Camera</h2>

      <div v-if="loading" class="text-gray-400 text-sm py-8">Loading camera...</div>
      <div v-else-if="loadError" class="text-sm py-8">
        <p class="text-red-400">{{ loadError }}</p>
        <div class="flex justify-end gap-3 mt-6">
          <button type="button" @click="$emit('close')" class="px-4 py-2 text-gray-400 hover:text-white transition-colors">Close</button>
        </div>
      </div>

      <form v-else @submit.prevent="save">
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name</label>
            <input v-model="form.name" type="text" required class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Stream URL</label>
            <input v-model="form.stream_url" type="text" required placeholder="rtsp://..." class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Main RTSP stream. A wrong path connects but never delivers frames.</p>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Sub Stream URL</label>
            <input v-model="form.sub_stream_url" type="text" placeholder="rtsp://... (optional)" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Leave empty &mdash; the server makes its own low-resolution grid stream.</p>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ONVIF URL</label>
            <input v-model="form.onvif_url" type="text" placeholder="http://<camera-ip>:8899 (optional)" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Usually detected automatically, e.g. http://&lt;camera-ip&gt;:8899</p>
          </div>

          <div class="border-t border-gray-200 dark:border-nvr-border pt-4">
            <div class="flex flex-wrap items-center gap-2">
              <p class="text-sm font-medium text-gray-700 dark:text-gray-300">Camera login</p>
              <span
                class="text-xs px-2 py-0.5 rounded font-medium"
                :class="original?.has_credentials ? 'bg-green-500/10 text-green-400' : 'bg-gray-500/10 text-gray-400'"
              >
                {{ original?.has_credentials ? 'A camera login is stored' : 'No login stored' }}
              </span>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 mb-3">
              The camera's own admin account. Authenticated ONVIF unlocks PTZ and camera configuration.
            </p>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
                <input v-model="form.username" type="text" autocomplete="off" :placeholder="original?.has_credentials ? 'unchanged' : 'admin'" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Password</label>
                <input v-model="form.password" type="password" autocomplete="new-password" placeholder="unchanged" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white" />
              </div>
            </div>
            <p v-if="original?.has_credentials" class="text-xs text-gray-500 dark:text-gray-400 mt-2">
              Each half is kept unless you replace it: leave the password empty to keep the current password, leave the
              username empty to keep the current username. The stored login is never sent back to the browser.
            </p>
            <p v-else class="text-xs text-gray-500 dark:text-gray-400 mt-2">
              Enter the camera's username and password to store a login for this camera.
            </p>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 border-t border-gray-200 dark:border-nvr-border pt-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Recording</label>
              <select v-model="form.recording_mode" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white">
                <option value="continuous">Continuous</option>
                <option value="motion">Motion</option>
                <option value="disabled">Disabled</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Connection</label>
              <select v-model="form.connection_type" class="w-full bg-gray-100 dark:bg-nvr-darker border border-gray-200 dark:border-nvr-border rounded px-3 py-2 text-gray-900 dark:text-white">
                <option value="ethernet">Ethernet</option>
                <option value="wifi">WiFi</option>
              </select>
            </div>
          </div>

          <!-- PTZ check -->
          <div class="border-t border-gray-200 dark:border-nvr-border pt-4">
            <div class="flex flex-wrap items-center gap-3">
              <button type="button" @click="testPtz" :disabled="ptzTesting" class="bg-gray-100 dark:bg-nvr-darker hover:bg-gray-200 dark:hover:bg-nvr-border disabled:opacity-50 text-gray-700 dark:text-gray-300 px-3 py-1.5 rounded border border-gray-200 dark:border-nvr-border transition-colors text-sm whitespace-nowrap">
                {{ ptzTesting ? 'Checking...' : 'Test PTZ' }}
              </button>
              <span v-if="ptzResult" class="text-xs" :class="ptzResult.ok ? 'text-green-400' : 'text-red-400'">
                {{ ptzResult.message }}
              </span>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
              Save first &mdash; the check uses the credentials already stored on the server.
            </p>
          </div>
        </div>

        <div class="flex flex-wrap justify-end gap-3 mt-6">
          <button type="button" @click="$emit('close')" class="px-4 py-2 text-gray-400 hover:text-white transition-colors">Cancel</button>
          <button type="submit" :disabled="saving" class="bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white px-4 py-2 rounded-lg transition-colors">
            {{ saving ? 'Saving...' : 'Save' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ open: boolean; cameraId: string | null }>()
const emit = defineEmits<{ close: []; saved: [camera: any] }>()

const { success, error: showError, warning } = useToast()

const loading = ref(false)
const saving = ref(false)
const loadError = ref('')
const original = ref<any>(null)

const blank = () => ({
  name: '',
  stream_url: '',
  sub_stream_url: '',
  onvif_url: '',
  username: '',
  password: '',
  recording_mode: 'continuous',
  connection_type: 'ethernet',
})

const form = ref(blank())

const ptzTesting = ref(false)
const ptzResult = ref<{ ok: boolean; message: string } | null>(null)

/** Pull a human message out of an $fetch error, whatever shape the API used. */
const apiMessage = (e: any, fallback: string): string => {
  const body = e?.data ?? e?.response?._data
  const err = body?.error
  if (typeof err === 'string' && err) return err
  if (err && typeof err.message === 'string' && err.message) return err.message
  if (typeof body?.message === 'string' && body.message) return body.message
  return fallback
}

const load = async () => {
  if (!props.cameraId) return
  loading.value = true
  loadError.value = ''
  ptzResult.value = null
  try {
    const camera = await useApi<any>(`/api/cameras/${props.cameraId}`)
    original.value = camera
    form.value = {
      name: camera.name ?? '',
      stream_url: camera.stream_url ?? '',
      sub_stream_url: camera.sub_stream_url ?? '',
      onvif_url: camera.onvif_url ?? '',
      // The API never returns the stored camera login, so these start empty.
      username: '',
      password: '',
      recording_mode: camera.recording_mode ?? 'continuous',
      connection_type: camera.connection_type ?? 'ethernet',
    }
  } catch (e: any) {
    original.value = null
    form.value = blank()
    loadError.value = apiMessage(e, 'Failed to load camera')
  }
  loading.value = false
}

watch(
  () => (props.open ? props.cameraId : null),
  (id) => { if (id) load() },
  { immediate: true },
)

const buildPayload = (): Record<string, any> => {
  const o = original.value ?? {}
  const payload: Record<string, any> = {}

  const name = form.value.name.trim()
  if (name && name !== o.name) payload.name = name

  const streamUrl = form.value.stream_url.trim()
  if (streamUrl && streamUrl !== o.stream_url) payload.stream_url = streamUrl

  const sub = form.value.sub_stream_url.trim()
  if (sub !== (o.sub_stream_url ?? '')) payload.sub_stream_url = sub

  const onvif = form.value.onvif_url.trim()
  if (onvif !== (o.onvif_url ?? '')) payload.onvif_url = onvif

  if (form.value.recording_mode !== o.recording_mode) payload.recording_mode = form.value.recording_mode
  if (form.value.connection_type !== o.connection_type) payload.connection_type = form.value.connection_type

  // Credentials: each half replaces only itself, the server keeps the other
  // half of the stored login. Empty means "leave it alone", so never send one.
  const username = form.value.username.trim()
  const password = form.value.password
  if (username) payload.username = username
  if (password) payload.password = password

  return payload
}

const save = async () => {
  if (!props.cameraId) return
  const payload = buildPayload()
  if (Object.keys(payload).length === 0) {
    warning('Nothing changed')
    return
  }

  saving.value = true
  try {
    const camera = await useApi<any>(`/api/cameras/${props.cameraId}`, { method: 'PUT', body: payload })
    success('Camera updated')
    emit('saved', camera)
    emit('close')
  } catch (e: any) {
    showError(apiMessage(e, 'Failed to update camera'))
  }
  saving.value = false
}

const testPtz = async () => {
  if (!props.cameraId) return
  ptzTesting.value = true
  ptzResult.value = null
  try {
    const result = await useApi<{ supported: boolean }>(`/api/cameras/${props.cameraId}/ptz`)
    if (result?.supported) {
      ptzResult.value = { ok: true, message: 'PTZ ready' }
      success('PTZ ready')
    } else {
      ptzResult.value = { ok: false, message: 'No ONVIF PTZ service found on this camera' }
      warning('No ONVIF PTZ service found on this camera')
    }
  } catch (e: any) {
    const message = apiMessage(e, 'PTZ check failed')
    ptzResult.value = { ok: false, message }
    showError(message)
  }
  ptzTesting.value = false
}
</script>
