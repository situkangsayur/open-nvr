import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:video_player/video_player.dart';

import '../models/camera.dart';
import '../services/api_client.dart';
import '../theme.dart';

/// Live view for one camera, played as HLS.
///
/// HLS rather than the WebSocket/MSE path the web UI uses: on Android this
/// hands decoding to ExoPlayer, which is hardware accelerated and much easier
/// on the battery than shuttling frames through Dart.
class CameraLiveScreen extends StatefulWidget {
  const CameraLiveScreen({super.key, required this.camera, required this.api});

  final Camera camera;
  final ApiClient api;

  @override
  State<CameraLiveScreen> createState() => _CameraLiveScreenState();
}

class _CameraLiveScreenState extends State<CameraLiveScreen> {
  VideoPlayerController? _controller;
  bool _loading = true;
  String? _error;
  bool _fullscreen = false;

  @override
  void initState() {
    super.initState();
    _start();
  }

  Future<void> _start() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    // Nudge the backend in case this camera's pipeline is not running yet.
    await widget.api.startHls(widget.camera.id);

    final headers = await widget.api.imageHeaders();
    final controller = VideoPlayerController.networkUrl(
      widget.api.hlsPlaylist(widget.camera.id),
      httpHeaders: headers,
      videoPlayerOptions: VideoPlayerOptions(mixWithOthers: true),
    );

    try {
      await controller.initialize();
      await controller.setLooping(false);
      await controller.play();
      if (!mounted) {
        await controller.dispose();
        return;
      }
      setState(() {
        _controller = controller;
        _loading = false;
      });
    } catch (e) {
      await controller.dispose();
      if (!mounted) return;
      setState(() {
        _loading = false;
        _error = 'Stream belum siap. Backend mungkin masih menyiapkan HLS — '
            'tunggu beberapa detik lalu coba lagi.';
      });
    }
  }

  Future<void> _retry() async {
    final old = _controller;
    setState(() => _controller = null);
    await old?.dispose();
    await _start();
  }

  void _toggleFullscreen() {
    setState(() => _fullscreen = !_fullscreen);
    if (_fullscreen) {
      SystemChrome.setPreferredOrientations([
        DeviceOrientation.landscapeLeft,
        DeviceOrientation.landscapeRight,
      ]);
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
    } else {
      SystemChrome.setPreferredOrientations(DeviceOrientation.values);
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
    }
  }

  @override
  void dispose() {
    _controller?.dispose();
    // Leave the device as we found it, whatever state the screen was in.
    SystemChrome.setPreferredOrientations(DeviceOrientation.values);
    SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_fullscreen) {
      return Scaffold(
        backgroundColor: Colors.black,
        body: GestureDetector(
          onTap: _toggleFullscreen,
          child: Center(child: _player()),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: Text(widget.camera.name),
        actions: [
          IconButton(
            tooltip: 'Muat ulang stream',
            onPressed: _loading ? null : _retry,
            icon: const Icon(Icons.refresh),
          ),
          if (_controller != null)
            IconButton(
              tooltip: 'Layar penuh',
              onPressed: _toggleFullscreen,
              icon: const Icon(Icons.fullscreen),
            ),
        ],
      ),
      body: Column(
        children: [
          Container(
            color: Colors.black,
            width: double.infinity,
            child: AspectRatio(
              aspectRatio: _controller?.value.aspectRatio ?? 16 / 9,
              child: _player(),
            ),
          ),
          Expanded(child: _details()),
        ],
      ),
    );
  }

  Widget _player() {
    if (_loading) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 12),
            Text('Menyambung ke kamera…', style: TextStyle(color: NvrColors.textSecondary, fontSize: 12)),
          ],
        ),
      );
    }

    if (_error != null) {
      return Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, color: NvrColors.warning, size: 36),
            const SizedBox(height: 10),
            Text(
              _error!,
              textAlign: TextAlign.center,
              style: const TextStyle(color: NvrColors.textSecondary, fontSize: 12, height: 1.4),
            ),
            const SizedBox(height: 14),
            OutlinedButton(onPressed: _retry, child: const Text('Coba lagi')),
          ],
        ),
      );
    }

    final controller = _controller;
    if (controller == null) return const SizedBox.shrink();

    return Stack(
      fit: StackFit.expand,
      children: [
        VideoPlayer(controller),
        const Positioned(
          top: 10,
          left: 10,
          child: _LiveBadge(),
        ),
      ],
    );
  }

  Widget _details() {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _row('Status', widget.camera.statusLabel),
        _row('Protokol', widget.camera.protocolType?.toUpperCase() ?? '-'),
        _row('Rekaman', widget.camera.isRecording ? 'Aktif (segmen 5 menit)' : 'Nonaktif'),
        _row('PTZ', widget.camera.ptzCapable ? 'Didukung' : 'Tidak'),
        if (widget.camera.streamUrl != null) _row('Stream', widget.camera.streamUrl!),
      ],
    );
  }

  Widget _row(String label, String value) => Padding(
        padding: const EdgeInsets.only(bottom: 12),
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
}

class _LiveBadge extends StatelessWidget {
  const _LiveBadge();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.black54,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Row(
        children: [
          Container(
            height: 7,
            width: 7,
            decoration: const BoxDecoration(color: NvrColors.offline, shape: BoxShape.circle),
          ),
          const SizedBox(width: 6),
          const Text(
            'LIVE',
            style: TextStyle(color: Colors.white, fontSize: 10, fontWeight: FontWeight.bold, letterSpacing: 1),
          ),
        ],
      ),
    );
  }
}
