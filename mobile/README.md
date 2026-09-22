# Open-NVR mobile (Android, phone + tablet)

Flutter client for the self-hosted Open-NVR backend. One APK serves phones and
tablets (layout switches at a shortest side of 600 dp).

## Screens

- **Live** (home after login): 1 / 4 / 9 / 16 tile grid, swipe or arrows to page
  through cameras, layout remembered. Tap a tile for the single-camera view.
- **Single camera**: main-quality stream, mute toggle, fullscreen/landscape,
  PTZ pad (hold to move, release to stop, zoom, home) when the camera supports
  it, shortcut to its recordings.
- **Kamera**: camera cards with snapshot, status, live and recordings links.
- **Rekaman**: camera + day (only days with recordings), 24 h timeline, segment
  list, in-app player (seek, prev/next segment, auto-continue, 1x/2x/4x).

## How live video works

Live view loads the backend's `GET {api}/live-player.html#cam=…&token=…&quality=sub|main&muted=1|0[&fit=cover]`
in a WebView (MSE over WebSocket, sub-second latency). The page reports its
state over the `NVR` JavaScript channel (`playing`, `loading`, `error:<text>`,
`offline`); the app pushes refreshed Keycloak tokens with
`window.nvrSetToken()`, toggles sound with `window.nvrSetMuted()` and
reconnects with `window.nvrReconnect()`. HLS (`/api/hls/{id}/stream.m3u8`) is
only a fallback offered when the page is missing. Recordings are MP4 files
played by ExoPlayer with `?access_token=` in the URL.

## Build

Server addresses are not kept in the repository; pass them at build time
(they only seed the login screen and can be changed there):

```bash
flutter pub get
flutter analyze && flutter test
flutter build apk --release \
  --dart-define=NVR_LAN_HOST=<lan-ip> \
  --dart-define=NVR_VPN_HOST=<wireguard-ip>
# -> build/app/outputs/flutter-apk/app-release.apk
```
