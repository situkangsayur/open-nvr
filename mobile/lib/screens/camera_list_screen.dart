import 'package:flutter/material.dart';

import '../models/camera.dart';
import '../services/api_client.dart';
import '../services/camera_store.dart';
import '../services/server_config.dart';
import '../theme.dart';
import '../widgets/common.dart';
import 'camera_live_screen.dart';
import 'home_shell.dart';
import 'recordings_screen.dart';

/// All cameras as cards with a still thumbnail, status and quick links to
/// live view and recordings.
class CameraListScreen extends StatelessWidget {
  const CameraListScreen({super.key, required this.services, required this.active});

  final AppServices services;

  /// Thumbnails are only fetched while this tab is visible.
  final bool active;

  CameraStore get _store => services.store;

  void _openLive(BuildContext context, Camera camera) {
    Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => CameraLiveScreen(camera: camera, services: services)),
    );
  }

  void _openRecordings(BuildContext context, Camera camera) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => RecordingsScreen(
          services: services,
          active: true,
          initialCamera: camera,
          standalone: true,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: _store,
      builder: (context, _) => Scaffold(
        appBar: AppBar(
          title: const Text('Kamera'),
          actions: [
            IconButton(
              tooltip: 'Muat ulang',
              onPressed: _store.loading ? null : () => _store.refresh(),
              icon: const Icon(Icons.refresh),
            ),
            AccountMenu(auth: services.auth, servers: services.servers),
          ],
        ),
        body: RefreshIndicator(
          onRefresh: () => _store.refresh(),
          child: _buildBody(context),
        ),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    final cameras = _store.cameras;
    if (_store.loading && cameras.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_store.error != null && cameras.isEmpty) {
      return _ErrorState(message: _store.error!, onRetry: () => _store.refresh(), server: services.servers.current);
    }

    if (cameras.isEmpty) {
      return const Center(
        child: Text('Belum ada kamera terdaftar', style: TextStyle(color: NvrColors.textSecondary)),
      );
    }

    Widget card(Camera camera) => _CameraCard(
          camera: camera,
          api: services.api,
          showThumbnail: active,
          onTap: () => _openLive(context, camera),
          onRecordings: () => _openRecordings(context, camera),
        );

    return CustomScrollView(
      slivers: [
        SliverPadding(
          padding: const EdgeInsets.fromLTRB(12, 12, 12, 0),
          sliver: SliverToBoxAdapter(
            child: _SummaryBar(total: cameras.length, online: _store.onlineCount, server: services.servers.current),
          ),
        ),
        SliverPadding(
          padding: const EdgeInsets.all(12),
          sliver: SliverLayoutBuilder(
            builder: (context, constraints) {
              // One column on phones, two or three on tablets.
              final width = constraints.crossAxisExtent;
              final columns = (width / 460).ceil().clamp(1, 4);
              final cardWidth = (width - 10 * (columns - 1)) / columns;
              return SliverGrid(
                gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: columns,
                  mainAxisSpacing: 10,
                  crossAxisSpacing: 10,
                  // 16:9 thumbnail plus the text row underneath.
                  mainAxisExtent: cardWidth * 9 / 16 + 62,
                ),
                delegate: SliverChildBuilderDelegate(
                  (context, i) => card(cameras[i]),
                  childCount: cameras.length,
                ),
              );
            },
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
  const _CameraCard({
    required this.camera,
    required this.api,
    required this.showThumbnail,
    this.onTap,
    this.onRecordings,
  });
  final Camera camera;
  final ApiClient api;
  final bool showThumbnail;
  final VoidCallback? onTap;
  final VoidCallback? onRecordings;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: InkWell(
        onTap: onTap,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // The grid cell is sized for a 16:9 image; the image takes
            // whatever the text row leaves so nothing can overflow.
            Expanded(
              child: Stack(
                fit: StackFit.expand,
                children: [
                  Container(color: Colors.black),
                  if (camera.isOnline && showThumbnail)
                    _Thumbnail(camera: camera, api: api)
                  else if (camera.isOnline)
                    const SizedBox.shrink()
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
              padding: const EdgeInsets.only(left: 12, right: 4, top: 4, bottom: 4),
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
                  IconButton(
                    tooltip: 'Rekaman',
                    visualDensity: VisualDensity.compact,
                    onPressed: onRecordings,
                    icon: const Icon(Icons.video_library_outlined, size: 20),
                  ),
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
