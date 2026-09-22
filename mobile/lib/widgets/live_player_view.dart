import 'dart:async';

import 'package:flutter/material.dart';
import 'package:webview_flutter/webview_flutter.dart';
import 'package:webview_flutter_android/webview_flutter_android.dart';

import '../models/camera.dart';
import '../services/api_client.dart';
import '../theme.dart';

/// What the player page last told us over the `NVR` JavaScript channel.
enum LiveState { loading, playing, error, offline, pageMissing, unknown }

/// Low-latency live view: the backend's `live-player.html` (MSE over a
/// WebSocket, well under a second behind real time) inside a WebView.
///
/// HLS was simple but ran 6–15 s behind; for CCTV that is too late to be
/// useful. The page handles decoding and reconnects itself; this widget only
/// builds its URL, keeps its token fresh and draws a status overlay on top.
///
/// The WebView swallows touches, so callers that need taps (grid tiles) wrap
/// this in an [IgnorePointer] and put their own gesture detector around it.
class LivePlayerView extends StatefulWidget {
  const LivePlayerView({
    super.key,
    required this.camera,
    required this.api,
    this.mainQuality = false,
    this.muted = true,
    this.compact = false,
    this.cover = false,
    this.onStateChanged,
  });

  final Camera camera;
  final ApiClient api;

  /// Main stream (full resolution) for a single/enlarged view, sub stream for
  /// grid tiles.
  final bool mainQuality;
  final bool muted;

  /// Smaller overlay text for grid tiles.
  final bool compact;

  /// Crop to fill the box (grid tiles) instead of letterboxing.
  final bool cover;
  final ValueChanged<LiveState>? onStateChanged;

  @override
  State<LivePlayerView> createState() => LivePlayerViewState();
}

class LivePlayerViewState extends State<LivePlayerView> with WidgetsBindingObserver {
  late final WebViewController _controller;
  LiveState _state = LiveState.loading;
  String? _message;
  String? _loadedToken;
  bool _paused = false;
  bool _heardFromPage = false;
  Timer? _silenceTimer;
  int _loadGeneration = 0;

  LiveState get state => _state;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    widget.api.auth.addListener(_onAuthChanged);

    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setBackgroundColor(Colors.black)
      ..addJavaScriptChannel('NVR', onMessageReceived: (m) => _onPageMessage(m.message))
      ..setNavigationDelegate(
        NavigationDelegate(
          onPageFinished: (url) => _onPageFinished(url),
          onHttpError: (error) {
            final uri = error.request?.uri;
            if (uri != null && uri.path.endsWith('live-player.html')) {
              _setState(
                LiveState.pageMissing,
                'Player tidak tersedia di server (HTTP ${error.response?.statusCode ?? '?'})',
              );
            }
          },
          onWebResourceError: (error) {
            if (error.isForMainFrame ?? false) {
              _setState(LiveState.error, 'Tidak bisa memuat player: ${error.description}');
            }
          },
        ),
      );

    final platform = _controller.platform;
    if (platform is AndroidWebViewController) {
      // Muted autoplay must start without a tap, or every tile would sit on
      // a play button.
      platform.setMediaPlaybackRequiresUserGesture(false);
    }
    _controller.enableZoom(false);

    _load();
  }

  @override
  void didUpdateWidget(covariant LivePlayerView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.camera.id != widget.camera.id ||
        oldWidget.mainQuality != widget.mainQuality ||
        oldWidget.cover != widget.cover) {
      // After this frame: _load reports state to the parent, which must not
      // setState while it is still building us.
      scheduleMicrotask(_load);
    } else if (oldWidget.muted != widget.muted) {
      _applyMuted(widget.muted);
    }
  }

  /// Toggle sound in place through the page's hook; only reload (a second of
  /// re-buffering) if the page is too old to have it.
  Future<void> _applyMuted(bool muted) async {
    try {
      final result = await _controller.runJavaScriptReturningResult(
        "(function(){ if (window.nvrSetMuted) { window.nvrSetMuted($muted); return 'ok'; } return 'no'; })()",
      );
      if (result.toString().contains('ok')) return;
    } catch (_) {}
    if (mounted) await _load();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    widget.api.auth.removeListener(_onAuthChanged);
    _silenceTimer?.cancel();
    // Drop the stream right away rather than whenever the WebView is GC'd.
    _controller.loadRequest(Uri.parse('about:blank')).catchError((Object _) {});
    super.dispose();
  }

  /// Reconnect the stream: through the page's own hook when it has one (no
  /// page reload), otherwise by loading the page again. Either way with a
  /// token that is good for a while.
  Future<void> reload() async {
    if (_state == LiveState.pageMissing || _paused) return _load();
    final token = await widget.api.auth.freshToken();
    if (!mounted) return;
    if (token != null && token != _loadedToken) _sendToken(token);
    try {
      final result = await _controller.runJavaScriptReturningResult(
        "(function(){ if (window.nvrReconnect) { window.nvrReconnect(); return 'ok'; } return 'no'; })()",
      );
      if (result.toString().contains('ok')) {
        _setState(LiveState.loading, null);
        return;
      }
    } catch (_) {}
    if (mounted) await _load();
  }

  Future<void> _load() async {
    final generation = ++_loadGeneration;
    _heardFromPage = false;
    _silenceTimer?.cancel();
    _setState(LiveState.loading, null);

    final token = await widget.api.auth.freshToken();
    if (!mounted || generation != _loadGeneration || _paused) return;
    if (token == null) {
      _setState(LiveState.error, 'Sesi berakhir');
      return;
    }
    _loadedToken = token;
    await _controller.loadRequest(
      widget.api.livePlayer(
        cameraId: widget.camera.id,
        token: token,
        mainQuality: widget.mainQuality,
        muted: widget.muted,
        cover: widget.cover,
      ),
    );
  }

  void _onPageFinished(String url) {
    if (!url.contains('live-player.html')) return;
    // If the page never reports over the channel (older build), do not leave
    // a spinner covering video that may well be playing.
    _silenceTimer?.cancel();
    _silenceTimer = Timer(const Duration(seconds: 10), () {
      if (!_heardFromPage && _state == LiveState.loading) _setState(LiveState.unknown, null);
    });
  }

  void _onPageMessage(String message) {
    _heardFromPage = true;
    if (message == 'playing') {
      _setState(LiveState.playing, null);
    } else if (message == 'loading') {
      _setState(LiveState.loading, null);
    } else if (message == 'offline') {
      _setState(LiveState.offline, null);
    } else if (message.startsWith('error')) {
      final text = message.length > 6 ? message.substring(6) : 'Gagal memutar';
      _setState(LiveState.error, text);
      // The page retries on its own; make sure its next try has a live token.
      _pushFreshToken();
    }
  }

  /// Auth refreshed somewhere (keep-alive timer, another request): hand the
  /// new token to the page so its next reconnect is accepted.
  void _onAuthChanged() {
    final token = widget.api.auth.accessToken;
    if (token != null && token != _loadedToken) _sendToken(token);
  }

  Future<void> _pushFreshToken() async {
    final token = await widget.api.auth.freshToken();
    if (!mounted || token == null || token == _loadedToken) return;
    _sendToken(token);
  }

  void _sendToken(String token) {
    if (_paused) return;
    _loadedToken = token;
    // JWTs are base64url + dots, so single quotes cannot break out.
    _controller
        .runJavaScript("window.nvrSetToken && window.nvrSetToken('$token');")
        .catchError((Object _) {});
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    // A backgrounded app should not keep pulling video over the tunnel.
    if (state == AppLifecycleState.paused || state == AppLifecycleState.hidden) {
      if (!_paused) {
        _paused = true;
        _loadGeneration++;
        _silenceTimer?.cancel();
        _controller.loadRequest(Uri.parse('about:blank')).catchError((Object _) {});
      }
    } else if (state == AppLifecycleState.resumed && _paused) {
      _paused = false;
      _load();
    }
  }

  void _setState(LiveState state, String? message) {
    if (!mounted || (state == _state && message == _message)) return;
    final changed = state != _state;
    setState(() {
      _state = state;
      _message = message;
    });
    if (changed) widget.onStateChanged?.call(state);
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      fit: StackFit.expand,
      children: [
        const ColoredBox(color: Colors.black),
        WebViewWidget(controller: _controller),
        _overlay(),
      ],
    );
  }

  Widget _overlay() {
    final small = widget.compact;
    switch (_state) {
      case LiveState.playing:
      case LiveState.unknown:
        return const SizedBox.shrink();
      case LiveState.loading:
        return Center(
          child: SizedBox(
            height: small ? 20 : 32,
            width: small ? 20 : 32,
            child: CircularProgressIndicator(strokeWidth: small ? 2 : 3),
          ),
        );
      case LiveState.offline:
        return _cover(Icons.videocam_off, 'Kamera offline');
      case LiveState.pageMissing:
        return _cover(Icons.error_outline, _message ?? 'Player tidak tersedia');
      case LiveState.error:
        // Leave the last frame visible; the page is already retrying.
        return Align(
          alignment: Alignment.bottomCenter,
          child: Container(
            width: double.infinity,
            color: Colors.black54,
            padding: EdgeInsets.symmetric(horizontal: 8, vertical: small ? 2 : 6),
            child: Text(
              _message ?? 'Gangguan, menyambung ulang…',
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              textAlign: TextAlign.center,
              style: TextStyle(color: NvrColors.warning, fontSize: small ? 10 : 12),
            ),
          ),
        );
    }
  }

  Widget _cover(IconData icon, String text) {
    final small = widget.compact;
    return ColoredBox(
      color: Colors.black87,
      child: Center(
        child: Padding(
          padding: const EdgeInsets.all(8),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, color: NvrColors.textSecondary, size: small ? 22 : 36),
              if (!small || text.length < 40) ...[
                const SizedBox(height: 6),
                Text(
                  text,
                  textAlign: TextAlign.center,
                  maxLines: 3,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(color: NvrColors.textSecondary, fontSize: small ? 10 : 12),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
