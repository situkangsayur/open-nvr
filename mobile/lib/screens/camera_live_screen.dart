import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../models/camera.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/hls_player_view.dart';
import '../widgets/live_player_view.dart';
import '../widgets/ptz_controls.dart';
import 'home_shell.dart';
import 'recordings_screen.dart';

/// Full view of one camera: main-quality low-latency stream, mute, PTZ when the
/// camera has it, and a shortcut to its recordings.
class CameraLiveScreen extends StatefulWidget {
  const CameraLiveScreen({super.key, required this.camera, required this.services});

  final Camera camera;
  final AppServices services;

  @override
  State<CameraLiveScreen> createState() => _CameraLiveScreenState();
}

class _CameraLiveScreenState extends State<CameraLiveScreen> {
  // Keeps the same WebView (and its open stream) when the layout switches
  // between portrait, landscape and fullscreen.
  final _playerKey = GlobalKey<LivePlayerViewState>();

  bool _muted = true;
  bool _fullscreen = false;
  bool _showOverlay = true;
  bool _showPtzOverlay = false;
  bool _useHls = false;
  late bool _ptz = widget.camera.ptzCapable;
  LiveState _state = LiveState.loading;

  Camera get _camera => widget.services.store.byId(widget.camera.id) ?? widget.camera;

  @override
  void initState() {
    super.initState();
    if (!_ptz) {
      // ptz_capable may lag behind auto-detection; ask the camera itself.
      widget.services.api.ptzSupported(widget.camera.id).then((ok) {
        if (mounted && ok) setState(() => _ptz = true);
      });
    }
  }

  @override
  void dispose() {
    // Leave the device as we found it, whatever state the screen was in.
    SystemChrome.setPreferredOrientations(DeviceOrientation.values);
    SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
    super.dispose();
  }

  void _toggleFullscreen() {
    setState(() {
      _fullscreen = !_fullscreen;
      _showOverlay = true;
    });
    if (_fullscreen) {
      if (!isTablet(context)) {
        SystemChrome.setPreferredOrientations([
          DeviceOrientation.landscapeLeft,
          DeviceOrientation.landscapeRight,
        ]);
      }
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
    } else {
      SystemChrome.setPreferredOrientations(DeviceOrientation.values);
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
    }
  }

  void _openRecordings() {
    if (_fullscreen) _toggleFullscreen();
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => RecordingsScreen(
          services: widget.services,
          active: true,
          initialCamera: _camera,
          standalone: true,
        ),
      ),
    );
  }

  Widget _player() {
    if (_useHls) {
      return HlsPlayerView(camera: _camera, api: widget.services.api, muted: _muted);
    }
    return LivePlayerView(
      key: _playerKey,
      camera: _camera,
      api: widget.services.api,
      mainQuality: true,
      muted: _muted,
      onStateChanged: (s) => setState(() => _state = s),
    );
  }

  @override
  Widget build(BuildContext context) {
    final tablet = isTablet(context);
    final landscape = isLandscape(context);

    return ListenableBuilder(
      listenable: widget.services.store,
      builder: (context, _) {
        // Phones held sideways go straight to the immersive layout.
        if (_fullscreen || (landscape && !tablet)) return _immersive();
        if (tablet && landscape) return _sideBySide();
        return _portrait();
      },
    );
  }

  AppBar _appBar() => AppBar(
        title: Row(
          children: [
            StatusDot(color: _dotColor()),
            const SizedBox(width: 8),
            Expanded(child: Text(_camera.name, overflow: TextOverflow.ellipsis)),
          ],
        ),
        actions: [
          IconButton(
            tooltip: _muted ? 'Nyalakan suara' : 'Matikan suara',
            onPressed: () => setState(() => _muted = !_muted),
            icon: Icon(_muted ? Icons.volume_off : Icons.volume_up),
          ),
          IconButton(
            tooltip: 'Layar penuh',
            onPressed: _toggleFullscreen,
            icon: const Icon(Icons.fullscreen),
          ),
        ],
      );

  Color _dotColor() => switch (_state) {
        LiveState.playing => NvrColors.online,
        LiveState.offline || LiveState.pageMissing => NvrColors.offline,
        LiveState.error => NvrColors.warning,
        _ => statusColor(_camera.status),
      };

  Widget _video() => Container(
        color: Colors.black,
        width: double.infinity,
        child: AspectRatio(aspectRatio: 16 / 9, child: _player()),
      );

  Widget _portrait() {
    return Scaffold(
      appBar: _appBar(),
      body: Column(
        children: [
          _video(),
          Expanded(child: _panel()),
        ],
      ),
    );
  }

  Widget _sideBySide() {
    return Scaffold(
      appBar: _appBar(),
      body: Row(
        children: [
          Expanded(
            child: ColoredBox(color: Colors.black, child: Center(child: _video())),
          ),
          const VerticalDivider(width: 1),
          SizedBox(width: 340, child: _panel()),
        ],
      ),
    );
  }

  /// Controls below/beside the video: actions, PTZ pad, details.
  Widget _panel() {
    final camera = _camera;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        if (_state == LiveState.pageMissing && !_useHls) _hlsHint(),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            _action(
              icon: _muted ? Icons.volume_off : Icons.volume_up,
              label: _muted ? 'Suara mati' : 'Suara nyala',
              onTap: () => setState(() => _muted = !_muted),
            ),
            _action(icon: Icons.video_library_outlined, label: 'Rekaman', onTap: _openRecordings),
            _action(
              icon: Icons.refresh,
              label: 'Muat ulang',
              onTap: () => _useHls ? setState(() => _useHls = false) : _playerKey.currentState?.reload(),
            ),
            _action(icon: Icons.fullscreen, label: 'Layar penuh', onTap: _toggleFullscreen),
          ],
        ),
        const SizedBox(height: 20),
        if (_ptz) ...[
          const Text('Kontrol PTZ', style: TextStyle(fontWeight: FontWeight.w600)),
          const SizedBox(height: 4),
          const Text(
            'Tahan tombol arah untuk menggerakkan kamera, lepas untuk berhenti.',
            style: TextStyle(color: NvrColors.textSecondary, fontSize: 12),
          ),
          const SizedBox(height: 12),
          Center(child: PtzControls(api: widget.services.api, cameraId: camera.id)),
          const SizedBox(height: 20),
        ],
        _row('Status', camera.statusLabel),
        _row('Merek', [camera.brand, camera.model].whereType<String>().join(' ').trim().ifEmpty('-')),
        _row('Rekaman', camera.isRecording ? 'Aktif (segmen 5 menit)' : 'Nonaktif'),
        _row('PTZ', _ptz ? 'Didukung' : 'Tidak'),
        _row('Mode', _useHls ? 'HLS (cadangan, ada jeda)' : 'Live latensi rendah'),
      ],
    );
  }

  Widget _hlsHint() => Container(
        margin: const EdgeInsets.only(bottom: 16),
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: NvrColors.warning.withValues(alpha: 0.12),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: NvrColors.warning.withValues(alpha: 0.4)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Server belum punya player latensi rendah. Pakai HLS sebagai cadangan '
              '(ada jeda beberapa detik)?',
              style: TextStyle(color: NvrColors.warning, fontSize: 13),
            ),
            const SizedBox(height: 8),
            OutlinedButton(onPressed: () => setState(() => _useHls = true), child: const Text('Pakai HLS')),
          ],
        ),
      );

  Widget _action({required IconData icon, required String label, required VoidCallback onTap}) {
    return OutlinedButton.icon(
      onPressed: onTap,
      icon: Icon(icon, size: 18),
      label: Text(label),
    );
  }

  Widget _row(String label, String value) => Padding(
        padding: const EdgeInsets.only(bottom: 10),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(
              width: 90,
              child: Text(label, style: const TextStyle(color: NvrColors.textSecondary, fontSize: 13)),
            ),
            Expanded(child: Text(value, style: const TextStyle(fontSize: 13))),
          ],
        ),
      );

  /// Video edge to edge, controls floating on top; tap the video to hide them.
  Widget _immersive() {
    return Scaffold(
      backgroundColor: Colors.black,
      body: Stack(
        fit: StackFit.expand,
        children: [
          Center(
            child: AspectRatio(
              aspectRatio: 16 / 9,
              // Input goes to our tap detector, not the WebView.
              child: IgnorePointer(child: _player()),
            ),
          ),
          Positioned.fill(
            child: GestureDetector(
              behavior: HitTestBehavior.translucent,
              onTap: () => setState(() => _showOverlay = !_showOverlay),
            ),
          ),
          if (_showOverlay) ...[
            Positioned(
              top: 0,
              left: 0,
              right: 0,
              child: Container(
                decoration: const BoxDecoration(
                  gradient: LinearGradient(
                    colors: [Colors.black87, Colors.transparent],
                    begin: Alignment.topCenter,
                    end: Alignment.bottomCenter,
                  ),
                ),
                child: SafeArea(
                  bottom: false,
                  child: Row(
                    children: [
                      IconButton(
                        tooltip: 'Kembali',
                        onPressed: () => _fullscreen ? _toggleFullscreen() : Navigator.of(context).maybePop(),
                        icon: const Icon(Icons.arrow_back, color: Colors.white),
                      ),
                      StatusDot(color: _dotColor()),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          _camera.name,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(color: Colors.white, fontWeight: FontWeight.w600),
                        ),
                      ),
                      IconButton(
                        tooltip: _muted ? 'Nyalakan suara' : 'Matikan suara',
                        onPressed: () => setState(() => _muted = !_muted),
                        icon: Icon(_muted ? Icons.volume_off : Icons.volume_up, color: Colors.white),
                      ),
                      if (_ptz)
                        IconButton(
                          tooltip: 'PTZ',
                          onPressed: () => setState(() => _showPtzOverlay = !_showPtzOverlay),
                          icon: Icon(Icons.control_camera,
                              color: _showPtzOverlay ? NvrColors.accent : Colors.white),
                        ),
                      IconButton(
                        tooltip: 'Rekaman',
                        onPressed: _openRecordings,
                        icon: const Icon(Icons.video_library_outlined, color: Colors.white),
                      ),
                      if (_fullscreen)
                        IconButton(
                          tooltip: 'Keluar layar penuh',
                          onPressed: _toggleFullscreen,
                          icon: const Icon(Icons.fullscreen_exit, color: Colors.white),
                        )
                      else
                        IconButton(
                          tooltip: 'Layar penuh',
                          onPressed: _toggleFullscreen,
                          icon: const Icon(Icons.fullscreen, color: Colors.white),
                        ),
                    ],
                  ),
                ),
              ),
            ),
            if (_ptz && _showPtzOverlay)
              Positioned(
                right: 16,
                bottom: 16,
                child: SafeArea(
                  child: PtzControls(
                    api: widget.services.api,
                    cameraId: _camera.id,
                    compact: true,
                    translucent: true,
                  ),
                ),
              ),
          ],
        ],
      ),
    );
  }
}

extension on String {
  String ifEmpty(String fallback) => isEmpty ? fallback : this;
}
