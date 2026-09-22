import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../models/camera.dart';
import '../services/camera_store.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/live_player_view.dart';
import 'camera_live_screen.dart';
import 'home_shell.dart';

/// Multi-camera live view: 1 / 4 / 9 / 16 tiles, paged when there are more
/// cameras than tiles. Tap a tile for the full single-camera view.
class LiveGridScreen extends StatefulWidget {
  const LiveGridScreen({super.key, required this.services, required this.active});

  final AppServices services;

  /// False while another tab is showing: tiles are torn down so hidden video
  /// does not keep eating bandwidth and battery.
  final bool active;

  @override
  State<LiveGridScreen> createState() => _LiveGridScreenState();
}

class _LiveGridScreenState extends State<LiveGridScreen> {
  int? _tiles;
  int _page = 0;
  final _pager = PageController();

  /// True while the single-camera view is on top of us.
  bool _suspended = false;

  CameraStore get _store => widget.services.store;

  @override
  void initState() {
    super.initState();
    ViewPrefs.loadGrid().then((saved) {
      if (!mounted) return;
      setState(() => _tiles = saved ?? (isTablet(context) ? 9 : 4));
    });
  }

  @override
  void dispose() {
    _pager.dispose();
    super.dispose();
  }

  void _setTiles(int tiles) {
    setState(() {
      _tiles = tiles;
      _page = 0;
    });
    if (_pager.hasClients) _pager.jumpToPage(0);
    ViewPrefs.saveGrid(tiles);
  }

  Future<void> _open(Camera camera) async {
    setState(() => _suspended = true);
    await Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => CameraLiveScreen(camera: camera, services: widget.services)),
    );
    if (mounted) setState(() => _suspended = false);
  }

  void _goToPage(int page) {
    _pager.animateToPage(page, duration: const Duration(milliseconds: 250), curve: Curves.easeOut);
  }

  @override
  Widget build(BuildContext context) {
    final tiles = _tiles ?? (isTablet(context) ? 9 : 4);

    return ListenableBuilder(
      listenable: _store,
      builder: (context, _) {
        final cameras = _store.cameras;
        final pageCount = math.max(1, (cameras.length / tiles).ceil());
        final page = _page.clamp(0, pageCount - 1);

        return Scaffold(
          appBar: AppBar(
            title: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('Live'),
                if (cameras.isNotEmpty)
                  Text(
                    '${_store.onlineCount}/${cameras.length} online',
                    style: const TextStyle(fontSize: 11, color: NvrColors.textSecondary),
                  ),
              ],
            ),
            actions: [
              _LayoutPicker(current: tiles, onChanged: _setTiles),
              IconButton(
                tooltip: 'Muat ulang',
                onPressed: _store.loading ? null : () => _store.refresh(),
                icon: const Icon(Icons.refresh),
              ),
              AccountMenu(auth: widget.services.auth, servers: widget.services.servers),
            ],
          ),
          backgroundColor: Colors.black,
          body: _body(cameras, tiles, pageCount),
          bottomNavigationBar: pageCount > 1
              ? _PageBar(
                  page: page,
                  pageCount: pageCount,
                  onPrev: page > 0 ? () => _goToPage(page - 1) : null,
                  onNext: page < pageCount - 1 ? () => _goToPage(page + 1) : null,
                )
              : null,
        );
      },
    );
  }

  Widget _body(List<Camera> cameras, int tiles, int pageCount) {
    if ((!_store.loadedOnce && _store.loading) || _tiles == null) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_store.error != null && cameras.isEmpty) {
      return _Message(
        icon: Icons.cloud_off,
        text: _store.error!,
        detail: 'Profil aktif: ${widget.services.servers.current.name} '
            '(${widget.services.servers.current.apiBase})',
        onRetry: () => _store.refresh(),
      );
    }
    if (cameras.isEmpty) {
      return _Message(
        icon: Icons.videocam_off,
        text: 'Belum ada kamera terdaftar',
        onRetry: () => _store.refresh(),
      );
    }

    final live = widget.active && !_suspended;
    return PageView.builder(
      controller: _pager,
      itemCount: pageCount,
      onPageChanged: (p) => setState(() => _page = p),
      itemBuilder: (context, index) {
        final start = index * tiles;
        final slice = cameras.sublist(start, math.min(start + tiles, cameras.length));
        return _GridPage(
          cameras: slice,
          tiles: tiles,
          live: live,
          services: widget.services,
          onOpen: _open,
        );
      },
    );
  }
}

class _GridPage extends StatelessWidget {
  const _GridPage({
    required this.cameras,
    required this.tiles,
    required this.live,
    required this.services,
    required this.onOpen,
  });

  final List<Camera> cameras;
  final int tiles;
  final bool live;
  final AppServices services;
  final ValueChanged<Camera> onOpen;

  @override
  Widget build(BuildContext context) {
    final cols = math.sqrt(tiles).round();
    final rows = cols;
    const gap = 2.0;

    return LayoutBuilder(
      builder: (context, box) {
        // Largest 16:9 cells that fit the grid into the available space.
        var cellW = (box.maxWidth - gap * (cols - 1)) / cols;
        var cellH = cellW * 9 / 16;
        if (cellH * rows + gap * (rows - 1) > box.maxHeight) {
          cellH = (box.maxHeight - gap * (rows - 1)) / rows;
          cellW = cellH * 16 / 9;
        }
        // Whole pixels, so rounding can never push a row past the edge.
        cellW = cellW.floorToDouble();
        cellH = cellH.floorToDouble();

        return Center(
          child: SizedBox(
            width: cellW * cols + gap * (cols - 1),
            height: cellH * rows + gap * (rows - 1),
            child: Column(
              children: [
                for (var r = 0; r < rows; r++) ...[
                  if (r > 0) const SizedBox(height: gap),
                  Row(
                    children: [
                      for (var c = 0; c < cols; c++) ...[
                        if (c > 0) const SizedBox(width: gap),
                        SizedBox(
                          width: cellW,
                          height: cellH,
                          child: _cell(r * cols + c),
                        ),
                      ],
                    ],
                  ),
                ],
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _cell(int index) {
    if (index >= cameras.length) {
      return const ColoredBox(color: NvrColors.background);
    }
    final camera = cameras[index];
    return _Tile(
      key: ValueKey(camera.id),
      camera: camera,
      live: live,
      // Sub stream whenever several tiles share the screen.
      mainQuality: tiles < 4,
      compact: tiles >= 9,
      services: services,
      onTap: () => onOpen(camera),
    );
  }
}

class _Tile extends StatefulWidget {
  const _Tile({
    super.key,
    required this.camera,
    required this.live,
    required this.mainQuality,
    required this.compact,
    required this.services,
    required this.onTap,
  });

  final Camera camera;
  final bool live;
  final bool mainQuality;
  final bool compact;
  final AppServices services;
  final VoidCallback onTap;

  @override
  State<_Tile> createState() => _TileState();
}

class _TileState extends State<_Tile> {
  LiveState? _state;

  @override
  Widget build(BuildContext context) {
    final camera = widget.camera;
    final offline = camera.status.toLowerCase() == 'offline';

    final Color dot;
    if (!widget.live || offline) {
      dot = statusColor(camera.status);
    } else if (_state == LiveState.playing) {
      dot = NvrColors.online;
    } else if (_state == LiveState.offline || _state == LiveState.pageMissing) {
      dot = NvrColors.offline;
    } else {
      dot = statusColor(camera.status);
    }

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: widget.onTap,
      child: Stack(
        fit: StackFit.expand,
        children: [
          if (offline || !widget.live)
            _Placeholder(offline: offline, compact: widget.compact)
          else
            // The WebView would eat the tap; the tile's detector owns input.
            IgnorePointer(
              child: LivePlayerView(
                key: ValueKey('${camera.id}-${widget.mainQuality}'),
                camera: camera,
                api: widget.services.api,
                mainQuality: widget.mainQuality,
                compact: widget.compact,
                // Tiles fill their cell; the full view shows the whole frame.
                cover: true,
                onStateChanged: (s) => setState(() => _state = s),
              ),
            ),
          Positioned(
            left: 4,
            top: 4,
            right: 4,
            child: Align(
              alignment: Alignment.topLeft,
              child: CameraLabel(camera: camera, color: dot, small: widget.compact),
            ),
          ),
        ],
      ),
    );
  }
}

class _Placeholder extends StatelessWidget {
  const _Placeholder({required this.offline, required this.compact});
  final bool offline;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: NvrColors.surface,
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              offline ? Icons.videocam_off : Icons.pause_circle_outline,
              color: NvrColors.textSecondary,
              size: compact ? 20 : 32,
            ),
            if (!compact && offline) ...[
              const SizedBox(height: 4),
              const Text('Offline', style: TextStyle(color: NvrColors.textSecondary, fontSize: 12)),
            ],
          ],
        ),
      ),
    );
  }
}

class _LayoutPicker extends StatelessWidget {
  const _LayoutPicker({required this.current, required this.onChanged});
  final int current;
  final ValueChanged<int> onChanged;

  static IconData _icon(int tiles) => switch (tiles) {
        1 => Icons.crop_square,
        4 => Icons.grid_view,
        9 => Icons.apps,
        _ => Icons.view_comfy,
      };

  @override
  Widget build(BuildContext context) {
    return PopupMenuButton<int>(
      tooltip: 'Tata letak',
      icon: Icon(_icon(current)),
      initialValue: current,
      onSelected: onChanged,
      itemBuilder: (context) => [
        for (final n in ViewPrefs.gridChoices)
          PopupMenuItem(
            value: n,
            child: Row(
              children: [
                Icon(_icon(n), size: 20, color: n == current ? NvrColors.accent : null),
                const SizedBox(width: 12),
                Text(n == 1 ? '1 kamera' : '$n kamera'),
              ],
            ),
          ),
      ],
    );
  }
}

class _PageBar extends StatelessWidget {
  const _PageBar({required this.page, required this.pageCount, this.onPrev, this.onNext});
  final int page;
  final int pageCount;
  final VoidCallback? onPrev;
  final VoidCallback? onNext;

  @override
  Widget build(BuildContext context) {
    return Container(
      color: NvrColors.surface,
      child: SafeArea(
        top: false,
        child: SizedBox(
          height: 44,
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              IconButton(onPressed: onPrev, icon: const Icon(Icons.chevron_left), tooltip: 'Sebelumnya'),
              const SizedBox(width: 8),
              for (var i = 0; i < pageCount; i++)
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 3),
                  child: StatusDot(
                    color: i == page ? NvrColors.accent : NvrColors.border,
                    size: i == page ? 9 : 7,
                  ),
                ),
              const SizedBox(width: 8),
              Text('${page + 1}/$pageCount', style: const TextStyle(color: NvrColors.textSecondary, fontSize: 12)),
              IconButton(onPressed: onNext, icon: const Icon(Icons.chevron_right), tooltip: 'Berikutnya'),
            ],
          ),
        ),
      ),
    );
  }
}

class _Message extends StatelessWidget {
  const _Message({required this.icon, required this.text, this.detail, required this.onRetry});
  final IconData icon;
  final String text;
  final String? detail;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 56, color: NvrColors.textSecondary),
            const SizedBox(height: 16),
            Text(text, textAlign: TextAlign.center, style: const TextStyle(height: 1.4)),
            if (detail != null) ...[
              const SizedBox(height: 8),
              Text(
                detail!,
                textAlign: TextAlign.center,
                style: const TextStyle(color: NvrColors.textSecondary, fontSize: 12),
              ),
            ],
            const SizedBox(height: 24),
            OutlinedButton.icon(onPressed: onRetry, icon: const Icon(Icons.refresh), label: const Text('Coba lagi')),
          ],
        ),
      ),
    );
  }
}
