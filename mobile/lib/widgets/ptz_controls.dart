import 'package:flutter/material.dart';

import '../services/api_client.dart';
import '../theme.dart';

/// Hold-to-move PTZ pad: pressing a direction sends the move, lifting the
/// finger (or the gesture being cancelled) sends `stop`.
class PtzControls extends StatefulWidget {
  const PtzControls({
    super.key,
    required this.api,
    required this.cameraId,
    this.compact = false,
    this.translucent = false,
  });

  final ApiClient api;
  final String cameraId;

  /// Smaller buttons, for the overlay in landscape fullscreen.
  final bool compact;

  /// Semi-transparent styling for use on top of video.
  final bool translucent;

  @override
  State<PtzControls> createState() => _PtzControlsState();
}

class _PtzControlsState extends State<PtzControls> {
  double _speed = 0.5;
  PtzAction? _active;

  Future<void> _queue = Future.value();

  /// Commands go out strictly in order: a quick tap fires move and stop
  /// back to back, and a stop overtaking its move would leave the camera
  /// turning until the next press.
  void _send(PtzAction action) {
    _queue = _queue.then((_) => _deliver(action));
  }

  Future<void> _deliver(PtzAction action) async {
    try {
      await widget.api.ptz(widget.cameraId, action, speed: _speed);
    } on ApiException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
        ..hideCurrentSnackBar()
        ..showSnackBar(SnackBar(content: Text('PTZ gagal: ${e.message}')));
    }
  }

  void _press(PtzAction action) {
    setState(() => _active = action);
    _send(action);
  }

  void _release() {
    if (_active == null) return;
    setState(() => _active = null);
    _send(PtzAction.stop);
  }

  @override
  void dispose() {
    // Never leave a camera spinning because the screen closed mid-press.
    if (_active != null) {
      final api = widget.api;
      final id = widget.cameraId;
      _queue.then((_) => api.ptz(id, PtzAction.stop)).catchError((_) {});
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final size = widget.compact ? 44.0 : 56.0;

    final pad = SizedBox(
      width: size * 3,
      height: size * 3,
      child: Stack(
        children: [
          Positioned(left: size, top: 0, child: _hold(PtzAction.tiltUp, Icons.keyboard_arrow_up, size)),
          Positioned(left: 0, top: size, child: _hold(PtzAction.panLeft, Icons.keyboard_arrow_left, size)),
          Positioned(left: size * 2, top: size, child: _hold(PtzAction.panRight, Icons.keyboard_arrow_right, size)),
          Positioned(left: size, top: size * 2, child: _hold(PtzAction.tiltDown, Icons.keyboard_arrow_down, size)),
          Positioned(
            left: size,
            top: size,
            child: _button(
              size,
              icon: Icons.home_outlined,
              tooltip: 'Posisi awal',
              onTap: () => _send(PtzAction.home),
            ),
          ),
        ],
      ),
    );

    final zoom = Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        _hold(PtzAction.zoomIn, Icons.zoom_in, size),
        SizedBox(height: size * 0.5),
        _hold(PtzAction.zoomOut, Icons.zoom_out, size),
      ],
    );

    final controls = Row(
      mainAxisSize: MainAxisSize.min,
      children: [pad, SizedBox(width: size * 0.6), zoom],
    );

    if (widget.compact) return controls;

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        controls,
        const SizedBox(height: 8),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('Kecepatan', style: TextStyle(color: NvrColors.textSecondary, fontSize: 12)),
            SizedBox(
              width: 180,
              child: Slider(
                value: _speed,
                min: 0.1,
                max: 1.0,
                divisions: 9,
                label: '${(_speed * 100).round()}%',
                onChanged: (v) => setState(() => _speed = v),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _hold(PtzAction action, IconData icon, double size) {
    final active = _active == action;
    return Listener(
      onPointerDown: (_) => _press(action),
      onPointerUp: (_) => _release(),
      onPointerCancel: (_) => _release(),
      child: _face(size, icon, active),
    );
  }

  Widget _button(double size, {required IconData icon, required String tooltip, required VoidCallback onTap}) {
    return Tooltip(
      message: tooltip,
      child: GestureDetector(onTap: onTap, child: _face(size, icon, false)),
    );
  }

  Widget _face(double size, IconData icon, bool active) {
    final base = widget.translucent ? Colors.black45 : NvrColors.surfaceRaised;
    return SizedBox(
      width: size,
      height: size,
      child: Padding(
        padding: const EdgeInsets.all(2),
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: active ? NvrColors.accent.withValues(alpha: 0.5) : base,
            shape: BoxShape.circle,
            border: Border.all(color: active ? NvrColors.accent : NvrColors.border),
          ),
          child: Icon(icon, color: Colors.white, size: size * 0.5),
        ),
      ),
    );
  }
}
