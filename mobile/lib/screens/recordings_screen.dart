import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:video_player/video_player.dart';

import '../models/camera.dart';
import '../models/recording.dart';
import '../services/api_client.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/timeline_bar.dart';
import 'camera_live_screen.dart';
import 'home_shell.dart';

/// Browse and play recorded MP4 segments: camera + day, a 24 h timeline, the
/// segment list, and a player that runs on into the next segment by itself.
class RecordingsScreen extends StatefulWidget {
  const RecordingsScreen({
    super.key,
    required this.services,
    required this.active,
    this.initialCamera,
    this.standalone = false,
  });

  final AppServices services;

  /// False while another tab is showing; playback pauses.
  final bool active;
  final Camera? initialCamera;

  /// Pushed as its own route (from a camera) rather than living in a tab.
  final bool standalone;

  @override
  State<RecordingsScreen> createState() => RecordingsScreenState();
}

class RecordingsScreenState extends State<RecordingsScreen> {
  static const _speeds = [1.0, 2.0, 4.0];

  Camera? _camera;
  List<RecordingDay> _days = const [];
  DateTime? _day;
  List<RecordingFile> _files = const [];
  bool _loadingDays = false;
  bool _loadingFiles = false;
  String? _error;

  VideoPlayerController? _controller;
  int? _index;
  bool _opening = false;
  bool _advancing = false;
  String? _playError;
  Duration _resumeAt = Duration.zero;
  double _speed = 1.0;
  bool _fullscreen = false;
  int _openGeneration = 0;
  int? _liveOfferedFor;
  final _listScroll = ScrollController();

  @override
  void initState() {
    super.initState();
    widget.services.store.addListener(_onStore);
    final initial = widget.initialCamera;
    if (initial != null) {
      _selectCamera(initial);
    } else {
      _onStore();
    }
  }

  @override
  void didUpdateWidget(covariant RecordingsScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.active && !widget.active) _controller?.pause();
  }

  @override
  void dispose() {
    widget.services.store.removeListener(_onStore);
    _controller?.removeListener(_onTick);
    _controller?.dispose();
    _listScroll.dispose();
    if (_fullscreen) _restoreSystemUi();
    super.dispose();
  }

  /// Pick a camera automatically once the list is in, if none is chosen yet.
  void _onStore() {
    if (_camera != null) return;
    final cams = widget.services.store.cameras;
    if (cams.isNotEmpty) _selectCamera(cams.first);
  }

  ApiClient get _api => widget.services.api;

  // ------------------------------------------------------------ selection

  Future<void> _selectCamera(Camera camera) async {
    await _stopPlayback();
    setState(() {
      _camera = camera;
      _days = const [];
      _files = const [];
      _day = null;
      _error = null;
      _loadingDays = true;
    });
    try {
      final days = await _api.recordingDays(camera.id);
      if (!mounted || _camera?.id != camera.id) return;
      final now = DateTime.now();
      final today = DateTime(now.year, now.month, now.day);
      final pick = days.any((d) => d.date == today) ? today : (days.isNotEmpty ? days.first.date : null);
      setState(() {
        _days = days;
        _loadingDays = false;
      });
      if (pick != null) {
        await _selectDay(pick);
      } else {
        setState(() => _day = today);
      }
    } on ApiException catch (e) {
      if (!mounted) return;
      setState(() {
        _loadingDays = false;
        _error = e.message;
      });
    }
  }

  Future<void> _selectDay(DateTime day) async {
    final camera = _camera;
    if (camera == null) return;
    await _stopPlayback();
    setState(() {
      _day = day;
      _files = const [];
      _loadingFiles = true;
      _error = null;
    });
    try {
      final files = await _api.recordingFiles(camera.id, day);
      if (!mounted || _day != day || _camera?.id != camera.id) return;
      setState(() {
        _files = files;
        _loadingFiles = false;
      });
    } on ApiException catch (e) {
      if (!mounted) return;
      setState(() {
        _loadingFiles = false;
        _error = e.message;
      });
    }
  }

  Future<void> _pickDate() async {
    if (_days.isEmpty) return;
    final available = _days.map((d) => d.date).toSet();
    final picked = await showDatePicker(
      context: context,
      initialDate: _day != null && available.contains(_day) ? _day! : _days.first.date,
      firstDate: _days.last.date,
      lastDate: _days.first.date,
      selectableDayPredicate: (d) => available.contains(DateTime(d.year, d.month, d.day)),
      helpText: 'Pilih tanggal rekaman',
    );
    if (picked != null) await _selectDay(DateTime(picked.year, picked.month, picked.day));
  }

  /// Step to the previous/next day that has recordings.
  void _stepDay(int direction) {
    final day = _day;
    if (day == null || _days.isEmpty) return;
    // _days is newest first; "previous" means older.
    final i = _days.indexWhere((d) => d.date == day);
    final next = i < 0 ? 0 : i - direction;
    if (next >= 0 && next < _days.length) _selectDay(_days[next].date);
  }

  // ------------------------------------------------------------- playback

  Future<void> _stopPlayback() async {
    _openGeneration++;
    final old = _controller;
    _controller = null;
    _index = null;
    _playError = null;
    _opening = false;
    old?.removeListener(_onTick);
    await old?.dispose();
  }

  /// Index of the nearest playable (finished) segment from [from] in
  /// [direction] (+1 / -1), or null.
  int? _playable(int from, int direction) {
    for (var i = from; i >= 0 && i < _files.length; i += direction) {
      if (_files[i].complete) return i;
    }
    return null;
  }

  /// The segment being written right now cannot be played; live view is the
  /// way to see "now".
  void _offerLive() {
    final camera = _camera;
    if (camera == null) return;
    _controller?.pause();
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(
        SnackBar(
          content: const Text('Segmen ini masih direkam. Tonton live untuk melihat saat ini.'),
          action: SnackBarAction(
            label: 'LIVE',
            onPressed: () => Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => CameraLiveScreen(camera: camera, services: widget.services)),
            ),
          ),
        ),
      );
  }

  Future<void> _play(int index, {Duration at = Duration.zero}) async {
    if (index < 0 || index >= _files.length) return;
    final file = _files[index];
    if (!file.complete) {
      _offerLive();
      return;
    }
    final generation = ++_openGeneration;

    final old = _controller;
    old?.removeListener(_onTick);
    setState(() {
      _controller = null;
      _index = index;
      _opening = true;
      _playError = null;
      _resumeAt = at;
    });
    await old?.dispose();

    final token = await _api.auth.freshToken();
    if (!mounted || generation != _openGeneration) return;
    if (token == null) {
      setState(() {
        _opening = false;
        _playError = 'Sesi berakhir';
      });
      return;
    }

    final controller = VideoPlayerController.networkUrl(
      _api.recordingUri(file, token),
      httpHeaders: {'Authorization': 'Bearer $token'},
    );
    try {
      await controller.initialize();
      if (at > Duration.zero) await controller.seekTo(at);
      await controller.setPlaybackSpeed(_speed);
      if (!mounted || generation != _openGeneration) {
        await controller.dispose();
        return;
      }
      controller.addListener(_onTick);
      if (widget.active) await controller.play();
      setState(() {
        _controller = controller;
        _opening = false;
        _advancing = false;
      });
      _scrollToCurrent();
    } catch (e) {
      await controller.dispose();
      if (!mounted || generation != _openGeneration) return;
      setState(() {
        _opening = false;
        _playError = 'Gagal membuka ${file.filename}';
      });
    }
  }

  /// Runs on every player update: detects the end of a segment (to roll on
  /// into the next one) and playback errors.
  void _onTick() {
    final c = _controller;
    if (c == null) return;
    final v = c.value;
    if (v.hasError && _playError == null) {
      // Most often an expired token on a range request; reopening at the
      // same spot gets a fresh one.
      _resumeAt = v.position;
      setState(() => _playError = 'Pemutaran terputus');
      return;
    }
    if (!v.isInitialized || _advancing) return;
    final ended = v.duration > Duration.zero &&
        v.position >= v.duration - const Duration(milliseconds: 300) &&
        !v.isPlaying;
    if (ended) {
      final i = _index;
      final next = i == null ? null : _playable(i + 1, 1);
      if (next != null) {
        _advancing = true;
        _play(next);
      } else if (i != null && i + 1 < _files.length) {
        // Caught up with the segment still being written.
        if (_liveOfferedFor != i) {
          _liveOfferedFor = i;
          _offerLive();
        }
      }
    }
  }

  void _seekToTime(DateTime t) {
    if (_files.isEmpty) return;
    var index = _files.indexWhere((f) => f.contains(t));
    var offset = Duration.zero;
    if (index >= 0) {
      offset = t.difference(_files[index].start);
    } else {
      // In a gap: start at the next segment after it (or the last one).
      index = _files.indexWhere((f) => f.start.isAfter(t));
      if (index < 0) index = _files.length - 1;
    }
    if (index == _index && _controller != null) {
      _controller!.seekTo(offset);
    } else {
      _play(index, at: offset);
    }
  }

  void _cycleSpeed() {
    final next = _speeds[(_speeds.indexOf(_speed) + 1) % _speeds.length];
    setState(() => _speed = next);
    _controller?.setPlaybackSpeed(next);
  }

  void _togglePlay() {
    final c = _controller;
    if (c == null) {
      final first = _playable(_index ?? 0, 1);
      if (first != null) _play(first);
      return;
    }
    if (c.value.isPlaying) {
      c.pause();
    } else {
      if (c.value.position >= c.value.duration) c.seekTo(Duration.zero);
      c.play();
    }
  }

  void _skip(Duration by) {
    final c = _controller;
    if (c == null) return;
    final target = c.value.position + by;
    if (target < Duration.zero) {
      c.seekTo(Duration.zero);
    } else if (target >= c.value.duration) {
      final next = _index == null ? null : _playable(_index! + 1, 1);
      if (next != null) _play(next);
    } else {
      c.seekTo(target);
    }
  }

  void _scrollToCurrent() {
    final i = _index;
    if (i == null || !_listScroll.hasClients) return;
    const rowHeight = 56.0;
    final target = (i * rowHeight - 2 * rowHeight).clamp(0.0, _listScroll.position.maxScrollExtent);
    _listScroll.animateTo(target, duration: const Duration(milliseconds: 250), curve: Curves.easeOut);
  }

  void _toggleFullscreen() {
    setState(() => _fullscreen = !_fullscreen);
    if (_fullscreen) {
      if (!isTablet(context)) {
        SystemChrome.setPreferredOrientations([DeviceOrientation.landscapeLeft, DeviceOrientation.landscapeRight]);
      }
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
    } else {
      _restoreSystemUi();
    }
  }

  void _restoreSystemUi() {
    SystemChrome.setPreferredOrientations(DeviceOrientation.values);
    SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
  }

  // ---------------------------------------------------------------- build

  @override
  Widget build(BuildContext context) {
    if (_fullscreen) {
      return PopScope(
        canPop: false,
        onPopInvokedWithResult: (didPop, _) {
          if (!didPop) _toggleFullscreen();
        },
        child: Scaffold(
          backgroundColor: Colors.black,
          body: SafeArea(
            child: Column(
              children: [
                Expanded(child: _videoArea()),
                _controls(),
              ],
            ),
          ),
        ),
      );
    }

    final wide = isTablet(context) || MediaQuery.sizeOf(context).width >= 840;
    final landscapePhone = !wide && isLandscape(context);

    final Widget body;
    if (wide || landscapePhone) {
      body = Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          SizedBox(
            width: wide ? 380 : 300,
            child: Column(
              children: [
                _selectors(),
                _timeline(),
                const Divider(height: 1),
                Expanded(child: _list()),
              ],
            ),
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: Column(
              children: [
                Expanded(child: ColoredBox(color: Colors.black, child: _videoArea())),
                _controls(),
              ],
            ),
          ),
        ],
      );
    } else {
      body = Column(
        children: [
          AspectRatio(aspectRatio: 16 / 9, child: ColoredBox(color: Colors.black, child: _videoArea())),
          _controls(),
          _selectors(),
          _timeline(),
          const Divider(height: 1),
          Expanded(child: _list()),
        ],
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('Rekaman'),
        actions: [
          IconButton(
            tooltip: 'Muat ulang',
            onPressed: _camera == null
                ? null
                : () => _day != null ? _selectDay(_day!) : _selectCamera(_camera!),
            icon: const Icon(Icons.refresh),
          ),
          if (!widget.standalone) AccountMenu(auth: widget.services.auth, servers: widget.services.servers),
        ],
      ),
      body: body,
    );
  }

  Widget _selectors() {
    final cameras = widget.services.store.cameras;
    final camera = _camera;
    final dayIndex = _day == null ? -1 : _days.indexWhere((d) => d.date == _day);
    final dayInfo = dayIndex >= 0 ? _days[dayIndex] : null;

    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 10, 12, 4),
      child: Column(
        children: [
          DropdownButtonFormField<String>(
            initialValue: camera != null && cameras.any((c) => c.id == camera.id) ? camera.id : null,
            isExpanded: true,
            decoration: const InputDecoration(labelText: 'Kamera', prefixIcon: Icon(Icons.videocam_outlined)),
            hint: Text(camera?.name ?? 'Pilih kamera'),
            items: [
              for (final c in cameras)
                DropdownMenuItem(
                  value: c.id,
                  child: Row(
                    children: [
                      StatusDot(color: statusColor(c.status)),
                      const SizedBox(width: 8),
                      Expanded(child: Text(c.name, overflow: TextOverflow.ellipsis)),
                    ],
                  ),
                ),
            ],
            onChanged: (id) {
              final c = cameras.where((c) => c.id == id).firstOrNull;
              if (c != null && c.id != _camera?.id) _selectCamera(c);
            },
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              IconButton(
                tooltip: 'Hari sebelumnya',
                onPressed: dayIndex >= 0 && dayIndex + 1 < _days.length ? () => _stepDay(-1) : null,
                icon: const Icon(Icons.chevron_left),
              ),
              Expanded(
                child: OutlinedButton.icon(
                  onPressed: _days.isEmpty ? null : _pickDate,
                  icon: const Icon(Icons.calendar_today, size: 16),
                  label: Text(
                    _loadingDays
                        ? 'Memuat…'
                        : _day == null
                            ? 'Belum ada rekaman'
                            : formatDay(_day!),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ),
              IconButton(
                tooltip: 'Hari berikutnya',
                onPressed: dayIndex > 0 ? () => _stepDay(1) : null,
                icon: const Icon(Icons.chevron_right),
              ),
            ],
          ),
          if (dayInfo != null)
            Text(
              '${dayInfo.count} segmen · ${formatBytes(dayInfo.sizeBytes)}',
              style: const TextStyle(color: NvrColors.textSecondary, fontSize: 11),
            ),
        ],
      ),
    );
  }

  Widget _timeline() {
    final day = _day;
    if (day == null) return const SizedBox(height: 8);
    final c = _controller;
    final i = _index;
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 6, 12, 6),
      child: c == null || i == null
          ? TimelineBar(day: day, files: _files, onSeek: _seekToTime)
          : ValueListenableBuilder<VideoPlayerValue>(
              valueListenable: c,
              builder: (context, v, _) => TimelineBar(
                day: day,
                files: _files,
                playhead: _files[i].start.add(v.position),
                onSeek: _seekToTime,
              ),
            ),
    );
  }

  Widget _list() {
    if (_error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.error_outline, color: NvrColors.warning, size: 36),
              const SizedBox(height: 8),
              Text(_error!, textAlign: TextAlign.center),
              const SizedBox(height: 12),
              OutlinedButton(
                onPressed: _camera == null ? null : () => _selectCamera(_camera!),
                child: const Text('Coba lagi'),
              ),
            ],
          ),
        ),
      );
    }
    if (_loadingDays || _loadingFiles) return const Center(child: CircularProgressIndicator());
    if (_camera == null) {
      return const Center(child: Text('Pilih kamera', style: TextStyle(color: NvrColors.textSecondary)));
    }
    if (_files.isEmpty) {
      return const Center(
        child: Text('Tidak ada rekaman di hari ini', style: TextStyle(color: NvrColors.textSecondary)),
      );
    }

    return ListView.builder(
      controller: _listScroll,
      itemCount: _files.length,
      itemExtent: 56,
      itemBuilder: (context, i) {
        final f = _files[i];
        final selected = i == _index;
        final gapBefore = i > 0 && f.start.difference(_files[i - 1].end).inSeconds > 60;
        if (!f.complete) {
          return ListTile(
            dense: true,
            leading: const Icon(Icons.fiber_manual_record, color: NvrColors.offline),
            title: Text(
              '${formatHms(f.start)} – sekarang',
              style: const TextStyle(color: NvrColors.textSecondary),
            ),
            subtitle: const Text(
              'Sedang merekam · ketuk untuk tonton live',
              style: TextStyle(fontSize: 11, color: NvrColors.textSecondary),
            ),
            onTap: _offerLive,
          );
        }
        return ListTile(
          selected: selected,
          selectedTileColor: NvrColors.primary.withValues(alpha: 0.35),
          selectedColor: NvrColors.textPrimary,
          dense: true,
          leading: Icon(
            selected ? Icons.play_circle_fill : Icons.play_circle_outline,
            color: selected ? NvrColors.accent : NvrColors.textSecondary,
          ),
          title: Text('${formatHms(f.start)} – ${formatHms(f.end)}'),
          subtitle: Text(
            '${formatDuration(f.duration)} · ${formatBytes(f.sizeBytes)}${gapBefore ? ' · ada jeda sebelumnya' : ''}',
            style: TextStyle(fontSize: 11, color: gapBefore ? NvrColors.warning : NvrColors.textSecondary),
          ),
          onTap: () => _play(i),
        );
      },
    );
  }

  Widget _videoArea() {
    final c = _controller;
    if (_playError != null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, color: NvrColors.warning, size: 32),
            const SizedBox(height: 8),
            Text(_playError!, style: const TextStyle(color: NvrColors.textSecondary)),
            const SizedBox(height: 10),
            OutlinedButton(
              onPressed: _index == null ? null : () => _play(_index!, at: _resumeAt),
              child: const Text('Coba lagi'),
            ),
          ],
        ),
      );
    }
    if (_opening) return const Center(child: CircularProgressIndicator());
    if (c == null || !c.value.isInitialized) {
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.movie_outlined, color: NvrColors.textSecondary, size: 40),
            SizedBox(height: 8),
            Text('Pilih segmen atau ketuk timeline', style: TextStyle(color: NvrColors.textSecondary)),
          ],
        ),
      );
    }
    return GestureDetector(
      onTap: _togglePlay,
      onDoubleTap: _toggleFullscreen,
      child: Center(
        child: AspectRatio(aspectRatio: c.value.aspectRatio, child: VideoPlayer(c)),
      ),
    );
  }

  Widget _controls() {
    final c = _controller;
    final i = _index;
    final prev = i == null ? null : _playable(i - 1, -1);
    final next = i == null ? null : _playable(i + 1, 1);

    Widget buttons(bool playing) => Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            IconButton(
              tooltip: 'Segmen sebelumnya',
              onPressed: prev != null ? () => _play(prev) : null,
              icon: const Icon(Icons.skip_previous),
            ),
            IconButton(
              tooltip: 'Mundur 10 detik',
              onPressed: c == null ? null : () => _skip(const Duration(seconds: -10)),
              icon: const Icon(Icons.replay_10),
            ),
            IconButton.filled(
              tooltip: playing ? 'Jeda' : 'Putar',
              onPressed: _files.isEmpty ? null : _togglePlay,
              icon: Icon(playing ? Icons.pause : Icons.play_arrow),
            ),
            IconButton(
              tooltip: 'Maju 10 detik',
              onPressed: c == null ? null : () => _skip(const Duration(seconds: 10)),
              icon: const Icon(Icons.forward_10),
            ),
            IconButton(
              tooltip: 'Segmen berikutnya',
              onPressed: next != null ? () => _play(next) : null,
              icon: const Icon(Icons.skip_next),
            ),
            TextButton(
              onPressed: _cycleSpeed,
              child: Text('${_speed.toStringAsFixed(0)}x', style: const TextStyle(fontWeight: FontWeight.bold)),
            ),
            IconButton(
              tooltip: _fullscreen ? 'Keluar layar penuh' : 'Layar penuh',
              onPressed: c == null ? null : _toggleFullscreen,
              icon: Icon(_fullscreen ? Icons.fullscreen_exit : Icons.fullscreen),
            ),
          ],
        );

    if (c == null || i == null) {
      return Container(
        color: NvrColors.surface,
        child: FittedBox(fit: BoxFit.scaleDown, child: buttons(false)),
      );
    }

    final file = _files[i];
    return Container(
      color: NvrColors.surface,
      padding: const EdgeInsets.only(top: 4),
      child: ValueListenableBuilder<VideoPlayerValue>(
        valueListenable: c,
        builder: (context, v, _) {
          return Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              _SeekBar(
                position: v.position,
                duration: v.duration,
                buffered: v.buffered,
                onSeek: (d) => c.seekTo(d),
                startLabel: formatHms(file.start.add(v.position)),
                endLabel: formatHms(file.end),
              ),
              FittedBox(fit: BoxFit.scaleDown, child: buttons(v.isPlaying)),
            ],
          );
        },
      ),
    );
  }
}

class _SeekBar extends StatefulWidget {
  const _SeekBar({
    required this.position,
    required this.duration,
    required this.buffered,
    required this.onSeek,
    required this.startLabel,
    required this.endLabel,
  });

  final Duration position;
  final Duration duration;
  final List<DurationRange> buffered;
  final ValueChanged<Duration> onSeek;
  final String startLabel;
  final String endLabel;

  @override
  State<_SeekBar> createState() => _SeekBarState();
}

class _SeekBarState extends State<_SeekBar> {
  double? _drag;

  @override
  Widget build(BuildContext context) {
    final total = widget.duration.inMilliseconds.toDouble();
    final pos = widget.position.inMilliseconds.clamp(0, total).toDouble();
    final value = _drag ?? pos;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(
        children: [
          Text(widget.startLabel, style: const TextStyle(fontSize: 11, color: NvrColors.textSecondary)),
          Expanded(
            child: Slider(
              value: total <= 0 ? 0 : value.clamp(0, total),
              max: total <= 0 ? 1 : total,
              onChanged: total <= 0 ? null : (v) => setState(() => _drag = v),
              onChangeEnd: (v) {
                setState(() => _drag = null);
                widget.onSeek(Duration(milliseconds: v.round()));
              },
            ),
          ),
          Text(widget.endLabel, style: const TextStyle(fontSize: 11, color: NvrColors.textSecondary)),
        ],
      ),
    );
  }
}
