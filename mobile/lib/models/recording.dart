import 'package:flutter/foundation.dart';

/// One local calendar day that has recordings for a camera.
@immutable
class RecordingDay {
  const RecordingDay({required this.date, required this.count, required this.sizeBytes});

  /// Local midnight of that day.
  final DateTime date;
  final int count;
  final int sizeBytes;

  factory RecordingDay.fromJson(Map<String, dynamic> json) {
    final parts = (json['date'] as String).split('-').map(int.parse).toList();
    return RecordingDay(
      date: DateTime(parts[0], parts[1], parts[2]),
      count: (json['count'] as num?)?.toInt() ?? 0,
      sizeBytes: (json['size_bytes'] as num?)?.toInt() ?? 0,
    );
  }
}

/// One recorded MP4 segment (about five minutes of continuous recording).
@immutable
class RecordingFile {
  const RecordingFile({
    required this.filename,
    required this.start,
    required this.end,
    required this.durationSecs,
    required this.sizeBytes,
    required this.url,
    this.complete = true,
  });

  final String filename;

  /// Local time (converted from the UTC timestamps the API sends).
  final DateTime start;
  final DateTime end;
  final double durationSecs;
  final int sizeBytes;

  /// Path relative to the API base, e.g. `/api/recordings/files/<cam>/<file>.mp4`.
  final String url;

  /// False for the segment ffmpeg is still writing: its MP4 index is only
  /// written when the file is closed, so it cannot be played yet.
  final bool complete;

  Duration get duration => end.difference(start);

  bool contains(DateTime t) => !t.isBefore(start) && t.isBefore(end);

  factory RecordingFile.fromJson(Map<String, dynamic> json) {
    final start = DateTime.parse(json['start'] as String).toLocal();
    final durationSecs = (json['duration_secs'] as num?)?.toDouble() ?? 300;
    final endRaw = json['end'] as String?;
    final end = endRaw != null
        ? DateTime.parse(endRaw).toLocal()
        : start.add(Duration(milliseconds: (durationSecs * 1000).round()));
    return RecordingFile(
      filename: json['filename'] as String? ?? '',
      start: start,
      end: end,
      durationSecs: durationSecs,
      sizeBytes: (json['size_bytes'] as num?)?.toInt() ?? 0,
      url: json['url'] as String,
      complete: json['complete'] as bool? ?? true,
    );
  }
}

String formatBytes(int bytes) {
  if (bytes >= 1 << 30) return '${(bytes / (1 << 30)).toStringAsFixed(1)} GB';
  if (bytes >= 1 << 20) return '${(bytes / (1 << 20)).toStringAsFixed(0)} MB';
  if (bytes >= 1 << 10) return '${(bytes / (1 << 10)).toStringAsFixed(0)} KB';
  return '$bytes B';
}

String two(int n) => n.toString().padLeft(2, '0');

String formatHm(DateTime t) => '${two(t.hour)}:${two(t.minute)}';

String formatHms(DateTime t) => '${two(t.hour)}:${two(t.minute)}:${two(t.second)}';

String formatDuration(Duration d) {
  final h = d.inHours;
  final m = d.inMinutes.remainder(60);
  final s = d.inSeconds.remainder(60);
  return h > 0 ? '$h:${two(m)}:${two(s)}' : '${two(m)}:${two(s)}';
}

const _days = ['Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu', 'Minggu'];
const _months = ['Jan', 'Feb', 'Mar', 'Apr', 'Mei', 'Jun', 'Jul', 'Agu', 'Sep', 'Okt', 'Nov', 'Des'];

/// "Senin, 22 Sep 2026" — or "Hari ini" / "Kemarin" when that is clearer.
String formatDay(DateTime day, {bool relative = true}) {
  final today = DateTime.now();
  final d0 = DateTime(day.year, day.month, day.day);
  final t0 = DateTime(today.year, today.month, today.day);
  final diff = t0.difference(d0).inDays;
  final full = '${_days[day.weekday - 1]}, ${day.day} ${_months[day.month - 1]} ${day.year}';
  if (!relative) return full;
  if (diff == 0) return 'Hari ini · ${day.day} ${_months[day.month - 1]}';
  if (diff == 1) return 'Kemarin · ${day.day} ${_months[day.month - 1]}';
  return full;
}

String formatDayShort(DateTime day) => '${day.day} ${_months[day.month - 1]}';
