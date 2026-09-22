// Open-NVR low-latency live player (MSE over WebSocket, go2rtc protocol).
// Parameters come in the URL fragment so the token never reaches server logs:
//   live-player.html#cam=<id>&token=<jwt>&quality=sub|main&muted=1|0&fit=contain|cover
// State is reported to a host app through a `NVR` JavaScript channel, if any:
//   loading | playing | offline | error:<text>
(function () {
  'use strict';

  const params = new URLSearchParams(location.hash.slice(1));
  const cam = params.get('cam') || '';
  const quality = params.get('quality') === 'sub' ? 'sub' : 'main';
  let token = params.get('token') || '';

  const video = document.getElementById('video');
  const status = document.getElementById('status');
  video.muted = params.get('muted') !== '0';
  if (params.get('fit') === 'cover') video.style.objectFit = 'cover';

  // Keep this list in step with go2rtc's own video-rtc.js.
  const CODECS = ['avc1.640029', 'avc1.64002A', 'avc1.640033', 'hvc1.1.6.L153.B0',
    'mp4a.40.2', 'mp4a.40.5', 'flac', 'opus'];

  let ws = null, ms = null, sb = null, queue = [];
  let failures = 0, lastData = 0, reconnectTimer = null, state = '';

  function report(s, text) {
    if (s === state && !text) return;
    state = s;
    status.textContent = s === 'playing' ? '' : (text || (s === 'loading' ? '' : s));
    try { if (window.NVR && window.NVR.postMessage) window.NVR.postMessage(text ? s + ':' + text : s); } catch (e) { /* no host */ }
  }

  window.nvrSetToken = function (t) { token = t; };
  window.nvrSetMuted = function (m) { video.muted = !!m; if (!m) video.play().catch(function () {}); };
  window.nvrReconnect = function () { failures = 0; restart(0); };

  function supportedCodecs() {
    return CODECS.filter(function (c) {
      return window.MediaSource && MediaSource.isTypeSupported('video/mp4; codecs="' + c + '"');
    }).join();
  }

  function teardown() {
    if (ws) { ws.onclose = ws.onerror = ws.onmessage = null; try { ws.close(); } catch (e) {} }
    ws = null; sb = null; queue = [];
    if (ms && video.src) { URL.revokeObjectURL(video.src); }
    ms = null;
  }

  function restart(delayMs) {
    teardown();
    clearTimeout(reconnectTimer);
    reconnectTimer = setTimeout(connect, delayMs);
  }

  function scheduleReconnect() {
    failures++;
    if (failures >= 3) report('offline');
    restart(Math.min(1000 * Math.pow(2, failures - 1), 10000));
  }

  function connect() {
    if (!cam) { report('error', 'no camera'); return; }
    if (!window.MediaSource) { report('error', 'MSE not supported'); return; }
    report('loading');
    const source = new MediaSource();
    ms = source;
    video.src = URL.createObjectURL(source);
    source.addEventListener('sourceopen', function () {
      if (ms === source) openSocket();   // ignore a source torn down meanwhile
    }, { once: true });
  }

  function openSocket() {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = proto + '//' + location.host + '/api/live/' + encodeURIComponent(cam) +
      '/ws?quality=' + quality + '&access_token=' + encodeURIComponent(token);
    ws = new WebSocket(url);
    ws.binaryType = 'arraybuffer';
    lastData = Date.now();

    ws.onopen = function () {
      ws.send(JSON.stringify({ type: 'mse', value: supportedCodecs() }));
    };
    ws.onmessage = function (ev) {
      if (typeof ev.data === 'string') {
        let msg;
        try { msg = JSON.parse(ev.data); } catch (e) { return; }
        if (msg.type === 'mse') {
          try {
            sb = ms.addSourceBuffer(msg.value);
            sb.mode = 'segments';
            sb.addEventListener('updateend', onUpdateEnd);
          } catch (e) { report('error', 'codec ' + msg.value); }
        } else if (msg.type === 'error') {
          report('error', msg.value);
        }
        return;
      }
      lastData = Date.now();
      queue.push(new Uint8Array(ev.data));
      pump();
    };
    ws.onerror = function () { /* onclose follows */ };
    ws.onclose = scheduleReconnect;
  }

  function pump() {
    if (!sb || sb.updating || queue.length === 0) return;
    let chunk;
    if (queue.length === 1) {
      chunk = queue[0];
    } else {
      let len = 0;
      for (const q of queue) len += q.byteLength;
      chunk = new Uint8Array(len);
      let off = 0;
      for (const q of queue) { chunk.set(q, off); off += q.byteLength; }
    }
    queue = [];
    try {
      sb.appendBuffer(chunk);
    } catch (e) {
      // QuotaExceeded or a broken buffer: start clean.
      restart(500);
    }
  }

  function onUpdateEnd() {
    const b = sb && sb.buffered;
    if (b && b.length) {
      const start = b.start(0), end = b.end(b.length - 1);
      const lag = end - video.currentTime;
      if (video.currentTime < start || lag > 0.7) {
        video.currentTime = Math.max(start, end - 0.3);   // jump to live edge
      }
      // Gently catch up small drifts instead of jumping.
      video.playbackRate = lag > 0.4 ? 1.1 : 1.0;
      if (video.paused) video.play().catch(function () {});
      if (video.currentTime - start > 20 && !sb.updating) {
        sb.remove(start, video.currentTime - 6);
        return; // pump continues on the next updateend
      }
    }
    pump();
  }

  video.addEventListener('playing', function () { failures = 0; report('playing'); });

  // Frozen feed (camera or network stalled): reconnect.
  setInterval(function () {
    if (ws && ws.readyState === WebSocket.OPEN && Date.now() - lastData > 8000) {
      report('loading');
      scheduleReconnect();
    }
  }, 2000);

  // Don't pull video nobody sees; resume at the live edge when visible.
  document.addEventListener('visibilitychange', function () {
    if (document.hidden) { teardown(); clearTimeout(reconnectTimer); }
    else { failures = 0; restart(0); }
  });

  connect();
})();
