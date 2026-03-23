<template>
  <div>
    <div class="flex items-center gap-4 mb-6">
      <NuxtLink to="/cameras" class="text-gray-400 hover:text-gray-700 dark:hover:text-white">&larr; Back</NuxtLink>
      <h1 class="text-2xl font-bold">Camera Setup Guide</h1>
    </div>

    <div class="space-y-6 max-w-3xl">
      <!-- V360 Pro / XMEye -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-bold mb-3 text-primary-600 dark:text-primary-400">V360 Pro / XMEye Cameras</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">V360 Pro cameras use P2P by default. You need to enable RTSP to connect via Open-NVR.</p>
        <div class="space-y-3 text-sm">
          <div class="flex gap-3">
            <span class="bg-primary-100 dark:bg-primary-600/20 text-primary-700 dark:text-primary-400 w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold flex-shrink-0">1</span>
            <p>Open <strong>V360 Pro</strong> or <strong>XMEye</strong> app on your phone</p>
          </div>
          <div class="flex gap-3">
            <span class="bg-primary-100 dark:bg-primary-600/20 text-primary-700 dark:text-primary-400 w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold flex-shrink-0">2</span>
            <p>Select your camera → tap <strong>Settings</strong> (gear icon)</p>
          </div>
          <div class="flex gap-3">
            <span class="bg-primary-100 dark:bg-primary-600/20 text-primary-700 dark:text-primary-400 w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold flex-shrink-0">3</span>
            <p>Go to <strong>Network Settings</strong> → find <strong>RTSP</strong> option</p>
          </div>
          <div class="flex gap-3">
            <span class="bg-primary-100 dark:bg-primary-600/20 text-primary-700 dark:text-primary-400 w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold flex-shrink-0">4</span>
            <p><strong>Enable RTSP</strong> and note the port (usually 554)</p>
          </div>
          <div class="flex gap-3">
            <span class="bg-primary-100 dark:bg-primary-600/20 text-primary-700 dark:text-primary-400 w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold flex-shrink-0">5</span>
            <p>Go to <strong>Cameras → Discover</strong> in Open-NVR and scan your subnet</p>
          </div>
        </div>
        <div class="mt-4 p-3 bg-gray-50 dark:bg-nvr-darker rounded text-xs font-mono">
          <p class="font-bold mb-1">Common RTSP URLs for V360/XMEye:</p>
          <p>rtsp://IP:554/user=admin&amp;password=&amp;channel=1&amp;stream=0.sdp</p>
          <p>rtsp://IP:554/ch0_0.h264</p>
          <p>rtsp://IP:554/live/ch00_1</p>
          <p>rtsp://IP:554/stream1</p>
        </div>
      </div>

      <!-- Hikvision -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-bold mb-3 text-primary-600 dark:text-primary-400">Hikvision</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">Hikvision cameras have RTSP enabled by default.</p>
        <div class="p-3 bg-gray-50 dark:bg-nvr-darker rounded text-xs font-mono">
          <p>rtsp://admin:password@IP:554/Streaming/Channels/101 (main)</p>
          <p>rtsp://admin:password@IP:554/Streaming/Channels/102 (sub)</p>
        </div>
      </div>

      <!-- Dahua / Imou -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-bold mb-3 text-primary-600 dark:text-primary-400">Dahua / Imou</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">Imou cameras may need RTSP enabled via Imou Life app. Dahua cameras have RTSP by default.</p>
        <div class="p-3 bg-gray-50 dark:bg-nvr-darker rounded text-xs font-mono">
          <p>rtsp://admin:password@IP:554/cam/realmonitor?channel=1&amp;subtype=0 (main)</p>
          <p>rtsp://admin:password@IP:554/cam/realmonitor?channel=1&amp;subtype=1 (sub)</p>
        </div>
      </div>

      <!-- Reolink -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-bold mb-3 text-primary-600 dark:text-primary-400">Reolink</h2>
        <div class="p-3 bg-gray-50 dark:bg-nvr-darker rounded text-xs font-mono">
          <p>rtsp://admin:password@IP:554/h264Preview_01_main</p>
          <p>rtsp://admin:password@IP:554/h264Preview_01_sub</p>
        </div>
      </div>

      <!-- TP-Link Tapo -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-bold mb-3 text-primary-600 dark:text-primary-400">TP-Link Tapo</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">Enable in Tapo app → Camera Settings → Advanced → Camera Account (set username/password).</p>
        <div class="p-3 bg-gray-50 dark:bg-nvr-darker rounded text-xs font-mono">
          <p>rtsp://username:password@IP:554/stream1 (main)</p>
          <p>rtsp://username:password@IP:554/stream2 (sub)</p>
        </div>
      </div>

      <!-- Generic ONVIF -->
      <div class="bg-white dark:bg-nvr-card rounded-lg p-6 border border-gray-200 dark:border-nvr-border">
        <h2 class="text-lg font-bold mb-3 text-primary-600 dark:text-primary-400">Generic ONVIF Camera</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">Most modern IP cameras support ONVIF. Use discovery to find them automatically.</p>
        <div class="p-3 bg-gray-50 dark:bg-nvr-darker rounded text-xs font-mono">
          <p>ONVIF URL: http://IP:80/onvif/device_service</p>
          <p>RTSP: Usually auto-discovered via ONVIF GetStreamUri</p>
        </div>
      </div>

      <!-- Manual Add -->
      <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-6 border border-blue-200 dark:border-blue-800">
        <h2 class="text-lg font-bold mb-3 text-blue-700 dark:text-blue-400">Adding Cameras Manually</h2>
        <ol class="space-y-2 text-sm text-blue-600 dark:text-blue-300 list-decimal list-inside">
          <li>Go to <strong>Cameras</strong> page</li>
          <li>Click <strong>"Add Camera"</strong> button</li>
          <li>Enter camera name, RTSP URL, and credentials</li>
          <li>Select protocol (RTSP, ONVIF, MJPEG)</li>
          <li>Click <strong>"Add"</strong> to save</li>
          <li>Use <strong>"Test"</strong> button to verify connection</li>
        </ol>
      </div>
    </div>
  </div>
</template>
