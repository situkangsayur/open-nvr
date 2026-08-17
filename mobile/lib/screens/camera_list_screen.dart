import 'package:flutter/material.dart';

import '../models/camera.dart';
import '../services/api_client.dart';
import '../services/auth_service.dart';
import '../services/server_config.dart';
import '../theme.dart';
import 'camera_live_screen.dart';

class CameraListScreen extends StatefulWidget {
  const CameraListScreen({
    super.key,
    required this.api,
    required this.auth,
    required this.servers,
  });

  final ApiClient api;
  final AuthService auth;
  final ServerConfigService servers;

  @override
  State<CameraListScreen> createState() => _CameraListScreenState();
}

class _CameraListScreenState extends State<CameraListScreen> {
  List<Camera> _cameras = const [];
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final cameras = await widget.api.listCameras();
      if (!mounted) return;
      setState(() {
        _cameras = cameras;
        _loading = false;
      });
    } on ApiException catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.message;
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final online = _cameras.where((c) => c.isOnline).length;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Kamera'),
        actions: [
          IconButton(
            tooltip: 'Muat ulang',
            onPressed: _loading ? null : _load,
            icon: const Icon(Icons.refresh),
          ),
          PopupMenuButton<String>(
            onSelected: (value) {
              if (value == 'logout') widget.auth.logout();
            },
            itemBuilder: (context) => [
              PopupMenuItem(
                enabled: false,
                child: Text(
                  '${widget.auth.username ?? 'user'} · ${widget.servers.current.name}',
                  style: const TextStyle(fontSize: 12, color: NvrColors.textSecondary),
                ),
              ),
              const PopupMenuDivider(),
              const PopupMenuItem(value: 'logout', child: Text('Keluar')),
            ],
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: _load,
        child: _buildBody(online),
      ),
    );
  }

  Widget _buildBody(int online) {
    if (_loading && _cameras.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null && _cameras.isEmpty) {
      return _ErrorState(message: _error!, onRetry: _load, server: widget.servers.current);
    }

    if (_cameras.isEmpty) {
      return const Center(
        child: Text('Belum ada kamera terdaftar', style: TextStyle(color: NvrColors.textSecondary)),
      );
    }

    return ListView(
      padding: const EdgeInsets.all(12),
      children: [
        _SummaryBar(total: _cameras.length, online: online, server: widget.servers.current),
        const SizedBox(height: 12),
        ..._cameras.map(
          (camera) => Padding(
            padding: const EdgeInsets.only(bottom: 10),
            child: _CameraCard(
              camera: camera,
              api: widget.api,
              onTap: camera.isOnline
                  ? () => Navigator.of(context).push(
                        MaterialPageRoute(
                          builder: (_) => CameraLiveScreen(camera: camera, api: widget.api),
                        ),
                      )
                  : null,
            ),
          ),
        ),
      ],
    );
  }
}

class _SummaryBar extends StatelessWidget {
  const _SummaryBar({required this.total, required this.online, required this.server});
  final int total;
  final int online;
  final ServerProfile server;

  @override
  Widget build(BuildContext context) {
    final offline = total - online;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: NvrColors.surface,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: NvrColors.border),
      ),
      child: Row(
        children: [
          _stat('$online', 'online', NvrColors.online),
          const SizedBox(width: 20),
          _stat('$offline', 'offline', offline > 0 ? NvrColors.offline : NvrColors.textSecondary),
          const Spacer(),
          Flexible(
            child: Text(
              '${server.name}\n${server.host}',
              textAlign: TextAlign.end,
              style: const TextStyle(color: NvrColors.textSecondary, fontSize: 11, height: 1.3),
            ),
          ),
        ],
      ),
    );
  }

  Widget _stat(String value, String label, Color color) => Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Text(value, style: TextStyle(color: color, fontSize: 22, fontWeight: FontWeight.bold)),
          const SizedBox(width: 4),
          Padding(
            padding: const EdgeInsets.only(bottom: 3),
            child: Text(label, style: const TextStyle(color: NvrColors.textSecondary, fontSize: 12)),
          ),
        ],
      );
}

class _CameraCard extends StatelessWidget {
  const _CameraCard({required this.camera, required this.api, this.onTap});
  final Camera camera;
  final ApiClient api;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: InkWell(
        onTap: onTap,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            AspectRatio(
              aspectRatio: 16 / 9,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  Container(color: Colors.black),
                  if (camera.isOnline)
                    _Thumbnail(camera: camera, api: api)
                  else
                    const Center(
                      child: Icon(Icons.videocam_off, color: NvrColors.textSecondary, size: 40),
                    ),
                  Positioned(
                    top: 8,
                    left: 8,
                    child: _StatusChip(camera: camera),
                  ),
                  if (camera.isOnline)
                    const Positioned(
                      right: 8,
                      bottom: 8,
                      child: Icon(Icons.play_circle_fill, color: Colors.white70, size: 34),
                    ),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              child: Row(
                children: [
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          camera.name,
                          style: const TextStyle(fontWeight: FontWeight.w600),
                          overflow: TextOverflow.ellipsis,
                        ),
                        Text(
                          [
                            camera.protocolType?.toUpperCase(),
                            if (camera.isRecording) 'merekam',
                            if (camera.ptzCapable) 'PTZ',
                          ].whereType<String>().join(' · '),
                          style: const TextStyle(color: NvrColors.textSecondary, fontSize: 11),
                        ),
                      ],
                    ),
                  ),
                  if (camera.isRecording && camera.isOnline)
                    const Icon(Icons.fiber_manual_record, color: NvrColors.offline, size: 14),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Snapshot thumbnail. A still frame costs far less than a second video
/// pipeline per card, which matters on a phone.
class _Thumbnail extends StatelessWidget {
  const _Thumbnail({required this.camera, required this.api});
  final Camera camera;
  final ApiClient api;

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<Map<String, String>>(
      future: api.imageHeaders(),
      builder: (context, snapshot) {
        if (!snapshot.hasData) {
          return const Center(child: CircularProgressIndicator(strokeWidth: 2));
        }
        return Image.network(
          api.snapshot(camera.id).toString(),
          headers: snapshot.data,
          fit: BoxFit.cover,
          gaplessPlayback: true,
          errorBuilder: (_, __, ___) => const Center(
            child: Icon(Icons.image_not_supported_outlined, color: NvrColors.textSecondary),
          ),
          loadingBuilder: (context, child, progress) =>
              progress == null ? child : const Center(child: CircularProgressIndicator(strokeWidth: 2)),
        );
      },
    );
  }
}

class _StatusChip extends StatelessWidget {
  const _StatusChip({required this.camera});
  final Camera camera;

  @override
  Widget build(BuildContext context) {
    final color = camera.isOnline ? NvrColors.online : NvrColors.offline;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.black54,
        borderRadius: BorderRadius.circular(20),
      ),
      child: Row(
        children: [
          Container(height: 7, width: 7, decoration: BoxDecoration(color: color, shape: BoxShape.circle)),
          const SizedBox(width: 6),
          Text(camera.statusLabel, style: const TextStyle(color: Colors.white, fontSize: 11)),
        ],
      ),
    );
  }
}

class _ErrorState extends StatelessWidget {
  const _ErrorState({required this.message, required this.onRetry, required this.server});
  final String message;
  final VoidCallback onRetry;
  final ServerProfile server;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(32),
      children: [
        const SizedBox(height: 40),
        const Icon(Icons.cloud_off, size: 56, color: NvrColors.textSecondary),
        const SizedBox(height: 16),
        Text(
          message,
          textAlign: TextAlign.center,
          style: const TextStyle(color: NvrColors.textPrimary, height: 1.4),
        ),
        const SizedBox(height: 8),
        Text(
          'Profil aktif: ${server.name} (${server.apiBase})',
          textAlign: TextAlign.center,
          style: const TextStyle(color: NvrColors.textSecondary, fontSize: 12),
        ),
        const SizedBox(height: 24),
        OutlinedButton.icon(
          onPressed: onRetry,
          icon: const Icon(Icons.refresh),
          label: const Text('Coba lagi'),
        ),
      ],
    );
  }
}
