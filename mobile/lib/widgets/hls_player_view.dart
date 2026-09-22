import 'package:flutter/material.dart';
import 'package:video_player/video_player.dart';

import '../models/camera.dart';
import '../services/api_client.dart';
import '../theme.dart';

/// Fallback live view over HLS (several seconds of delay). Only used when the
/// server has no low-latency player page.
class HlsPlayerView extends StatefulWidget {
  const HlsPlayerView({super.key, required this.camera, required this.api, this.muted = true});

  final Camera camera;
  final ApiClient api;
  final bool muted;

  @override
  State<HlsPlayerView> createState() => _HlsPlayerViewState();
}

class _HlsPlayerViewState extends State<HlsPlayerView> {
  VideoPlayerController? _controller;
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _start();
  }

  @override
  void didUpdateWidget(covariant HlsPlayerView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.muted != widget.muted) _controller?.setVolume(widget.muted ? 0 : 1);
  }

  Future<void> _start() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    await widget.api.startHls(widget.camera.id);
    final token = await widget.api.auth.freshToken();
    final controller = VideoPlayerController.networkUrl(
      widget.api.hlsPlaylist(widget.camera.id),
      httpHeaders: {if (token != null) 'Authorization': 'Bearer $token'},
      videoPlayerOptions: VideoPlayerOptions(mixWithOthers: true),
    );
    try {
      await controller.initialize();
      await controller.setVolume(widget.muted ? 0 : 1);
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
        _error = 'Stream HLS belum siap. Tunggu beberapa detik lalu coba lagi.';
      });
    }
  }

  Future<void> _retry() async {
    final old = _controller;
    setState(() => _controller = null);
    await old?.dispose();
    await _start();
  }

  @override
  void dispose() {
    _controller?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) return const Center(child: CircularProgressIndicator());
    if (_error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(_error!, textAlign: TextAlign.center, style: const TextStyle(color: NvrColors.textSecondary)),
              const SizedBox(height: 10),
              OutlinedButton(onPressed: _retry, child: const Text('Coba lagi')),
            ],
          ),
        ),
      );
    }
    final controller = _controller!;
    return Center(
      child: AspectRatio(aspectRatio: controller.value.aspectRatio, child: VideoPlayer(controller)),
    );
  }
}
