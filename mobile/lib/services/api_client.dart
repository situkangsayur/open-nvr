import 'dart:convert';

import 'package:http/http.dart' as http;

import '../models/camera.dart';
import '../models/recording.dart';
import 'auth_service.dart';
import 'server_config.dart';

/// Raised for API failures that the UI should show verbatim.
class ApiException implements Exception {
  ApiException(this.message, {this.statusCode});
  final String message;
  final int? statusCode;
  @override
  String toString() => message;
}

/// PTZ moves understood by `POST /api/cameras/{id}/ptz`.
enum PtzAction {
  panLeft('pan_left'),
  panRight('pan_right'),
  tiltUp('tilt_up'),
  tiltDown('tilt_down'),
  zoomIn('zoom_in'),
  zoomOut('zoom_out'),
  stop('stop'),
  home('home');

  const PtzAction(this.wire);
  final String wire;
}

/// Thin client over the Open-NVR REST API.
///
/// Reads its base URL from [ServerConfigService] on every call, so switching
/// server profile takes effect immediately with no re-wiring.
class ApiClient {
  ApiClient(this._servers, this.auth);

  final ServerConfigService _servers;
  final AuthService auth;

  String get base => _servers.current.apiBase;

  Future<Map<String, String>> _headers({bool json = false}) async {
    await auth.ensureFresh();
    final token = auth.accessToken;
    return {
      'Accept': 'application/json',
      if (json) 'Content-Type': 'application/json',
      if (token != null) 'Authorization': 'Bearer $token',
    };
  }

  Future<dynamic> _send(String method, String path, {Object? body, Duration? timeout}) async {
    final http.Response response;
    final uri = Uri.parse('$base$path');
    try {
      final headers = await _headers(json: body != null);
      final request = switch (method) {
        'POST' => http.post(uri, headers: headers, body: body == null ? null : jsonEncode(body)),
        _ => http.get(uri, headers: headers),
      };
      response = await request.timeout(timeout ?? const Duration(seconds: 20));
    } catch (e) {
      throw ApiException('Tidak bisa menjangkau server di $base');
    }

    if (response.statusCode == 401) {
      // One more chance: the token may have lapsed between check and use.
      final renewed = await auth.ensureFresh();
      if (!renewed) {
        await auth.logout();
        throw ApiException('Sesi berakhir, silakan login ulang', statusCode: 401);
      }
      throw ApiException('Server menolak token (HTTP 401). Coba lagi atau login ulang.', statusCode: 401);
    }
    if (response.statusCode >= 400) {
      throw ApiException(_errorText(response), statusCode: response.statusCode);
    }
    if (response.body.isEmpty) return null;
    return jsonDecode(response.body);
  }

  String _errorText(http.Response response) {
    try {
      final body = jsonDecode(response.body);
      if (body is Map && body['error'] is String) return body['error'] as String;
      if (body is Map && body['message'] is String) return body['message'] as String;
    } catch (_) {}
    return 'Server menjawab HTTP ${response.statusCode}';
  }

  Future<dynamic> _get(String path) => _send('GET', path);

  // ---------------------------------------------------------------- cameras

  Future<List<Camera>> listCameras() async {
    final data = await _get('/api/cameras');
    if (data is! List) throw ApiException('Balasan daftar kamera tidak sesuai');
    return data.whereType<Map<String, dynamic>>().map(Camera.fromJson).toList()
      ..sort((a, b) => a.name.compareTo(b.name));
  }

  /// Single JPEG frame — cheap enough to use as a thumbnail.
  Uri snapshot(String cameraId) =>
      Uri.parse('$base/api/cameras/$cameraId/snapshot?t=${DateTime.now().millisecondsSinceEpoch}');

  Future<Map<String, String>> imageHeaders() => _headers();

  // -------------------------------------------------------------- live view

  /// Low-latency player page (MSE over WebSocket), served by the backend
  /// outside /api. Parameters ride in the hash so they never reach server
  /// logs; the `_` query just forces a real reload when only the hash changed.
  Uri livePlayer({
    required String cameraId,
    required String token,
    required bool mainQuality,
    required bool muted,
    bool cover = false,
  }) {
    final hash = 'cam=${Uri.encodeComponent(cameraId)}'
        '&token=${Uri.encodeComponent(token)}'
        '&quality=${mainQuality ? 'main' : 'sub'}'
        '&muted=${muted ? 1 : 0}'
        '${cover ? '&fit=cover' : ''}';
    return Uri.parse('$base/live-player.html?_=${DateTime.now().millisecondsSinceEpoch}#$hash');
  }

  /// Ask the backend to spin up the HLS pipeline for a camera (fallback path).
  Future<void> startHls(String cameraId) async {
    try {
      await http
          .post(Uri.parse('$base/api/hls/$cameraId/start'), headers: await _headers())
          .timeout(const Duration(seconds: 15));
    } catch (_) {
      // Ignored: playback will surface a real problem.
    }
  }

  Uri hlsPlaylist(String cameraId) => Uri.parse('$base/api/hls/$cameraId/stream.m3u8');

  // -------------------------------------------------------------------- PTZ

  Future<bool> ptzSupported(String cameraId) async {
    try {
      final data = await _get('/api/cameras/$cameraId/ptz');
      return data is Map && data['supported'] == true;
    } on ApiException {
      return false;
    }
  }

  Future<void> ptz(String cameraId, PtzAction action, {double speed = 0.5}) async {
    await _send(
      'POST',
      '/api/cameras/$cameraId/ptz',
      body: {'action': action.wire, 'speed': speed.clamp(0.0, 1.0)},
      timeout: const Duration(seconds: 8),
    );
  }

  // ------------------------------------------------------------- recordings

  Future<List<RecordingDay>> recordingDays(String cameraId) async {
    final offset = DateTime.now().timeZoneOffset.inMinutes;
    final data = await _get('/api/recordings/files/$cameraId/days?tz_offset_minutes=$offset');
    final days = data is Map ? data['days'] : null;
    if (days is! List) throw ApiException('Balasan daftar hari rekaman tidak sesuai');
    return days.whereType<Map<String, dynamic>>().map(RecordingDay.fromJson).toList()
      ..sort((a, b) => b.date.compareTo(a.date));
  }

  /// Segments that overlap the local calendar day [day].
  Future<List<RecordingFile>> recordingFiles(String cameraId, DateTime day) async {
    final from = DateTime(day.year, day.month, day.day);
    final to = DateTime(day.year, day.month, day.day + 1);
    final query = Uri(queryParameters: {'from': _rfc3339(from), 'to': _rfc3339(to)}).query;
    final data = await _get('/api/recordings/files/$cameraId?$query');
    final files = data is Map ? data['files'] : null;
    if (files is! List) throw ApiException('Balasan daftar rekaman tidak sesuai');
    return files.whereType<Map<String, dynamic>>().map(RecordingFile.fromJson).toList()
      ..sort((a, b) => a.start.compareTo(b.start));
  }

  /// Playable URL for ExoPlayer: the token rides in the query string because
  /// the player's range requests cannot be trusted to keep custom headers.
  Uri recordingUri(RecordingFile file, String token, {bool download = false}) {
    final sep = file.url.contains('?') ? '&' : '?';
    return Uri.parse('$base${file.url}${sep}access_token=${Uri.encodeQueryComponent(token)}'
        '${download ? '&download=1' : ''}');
  }

  static String _rfc3339(DateTime local) {
    final u = local.toUtc();
    String p(int n) => n.toString().padLeft(2, '0');
    return '${u.year.toString().padLeft(4, '0')}-${p(u.month)}-${p(u.day)}'
        'T${p(u.hour)}:${p(u.minute)}:${p(u.second)}Z';
  }
}
