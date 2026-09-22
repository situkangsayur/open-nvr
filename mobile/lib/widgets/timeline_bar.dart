import 'package:flutter/material.dart';

import '../models/recording.dart';
import '../theme.dart';

/// 24-hour bar for one local day: recorded segments as filled blocks, the
/// playhead as a line. Tap anywhere to jump to that time.
class TimelineBar extends StatelessWidget {
  const TimelineBar({
    super.key,
    required this.day,
    required this.files,
    this.playhead,
    this.onSeek,
    this.height = 44,
  });

  /// Local midnight of the day shown.
  final DateTime day;
  final List<RecordingFile> files;
  final DateTime? playhead;
  final ValueChanged<DateTime>? onSeek;
  final double height;

  DateTime _timeAt(double dx, double width) {
    final fraction = (dx / width).clamp(0.0, 1.0);
    final dayStart = DateTime(day.year, day.month, day.day);
    final dayEnd = DateTime(day.year, day.month, day.day + 1);
    final span = dayEnd.difference(dayStart).inMilliseconds;
    return dayStart.add(Duration(milliseconds: (span * fraction).round()));
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final width = box.maxWidth;
        void seek(Offset local) => onSeek?.call(_timeAt(local.dx, width));
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapUp: (d) => seek(d.localPosition),
          child: SizedBox(
            width: width,
            height: height + 16,
            child: CustomPaint(
              painter: _TimelinePainter(day: day, files: files, playhead: playhead, barHeight: height),
            ),
          ),
        );
      },
    );
  }
}

class _TimelinePainter extends CustomPainter {
  _TimelinePainter({required this.day, required this.files, required this.playhead, required this.barHeight});

  final DateTime day;
  final List<RecordingFile> files;
  final DateTime? playhead;
  final double barHeight;

  @override
  void paint(Canvas canvas, Size size) {
    final dayStart = DateTime(day.year, day.month, day.day);
    final dayEnd = DateTime(day.year, day.month, day.day + 1);
    final span = dayEnd.difference(dayStart).inMilliseconds.toDouble();
    double x(DateTime t) => (t.difference(dayStart).inMilliseconds / span).clamp(0.0, 1.0) * size.width;

    final bar = RRect.fromRectAndRadius(Rect.fromLTWH(0, 0, size.width, barHeight), const Radius.circular(6));
    canvas.drawRRect(bar, Paint()..color = NvrColors.surfaceRaised);

    canvas.save();
    canvas.clipRRect(bar);
    final seg = Paint()..color = NvrColors.accent.withValues(alpha: 0.75);
    for (final f in files) {
      final left = x(f.start);
      // At least a pixel wide, so a short clip is still visible.
      final right = (x(f.end)).clamp(left + 1, size.width);
      canvas.drawRect(Rect.fromLTRB(left, 0, right, barHeight), seg);
    }
    canvas.restore();

    // Hour ticks and labels every 3 hours.
    final tick = Paint()
      ..color = NvrColors.border
      ..strokeWidth = 1;
    for (var h = 0; h <= 24; h++) {
      final tx = size.width * h / 24;
      final major = h % 3 == 0;
      canvas.drawLine(Offset(tx, barHeight - (major ? 10 : 5)), Offset(tx, barHeight), tick);
      if (major && h < 24) {
        final tp = TextPainter(
          text: TextSpan(text: two(h), style: const TextStyle(color: NvrColors.textSecondary, fontSize: 10)),
          textDirection: TextDirection.ltr,
        )..layout();
        final lx = (tx - tp.width / 2).clamp(0.0, size.width - tp.width);
        tp.paint(canvas, Offset(lx, barHeight + 2));
      }
    }

    final p = playhead;
    if (p != null && !p.isBefore(dayStart) && p.isBefore(dayEnd)) {
      final px = x(p);
      canvas.drawLine(
        Offset(px, -2),
        Offset(px, barHeight + 2),
        Paint()
          ..color = Colors.white
          ..strokeWidth = 2,
      );
    }

    // "Now" marker on today's bar: nothing can be recorded to its right yet.
    final now = DateTime.now();
    if (now.isAfter(dayStart) && now.isBefore(dayEnd)) {
      final nx = x(now);
      canvas.drawLine(
        Offset(nx, 0),
        Offset(nx, barHeight),
        Paint()
          ..color = NvrColors.offline
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _TimelinePainter old) =>
      old.files != files || old.playhead != playhead || old.day != day;
}
