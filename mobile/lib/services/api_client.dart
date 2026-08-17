import 'dart:convert';

import 'package:http/http.dart' as http;

import '../models/camera.dart';
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

/// Thin client over the Open-NVR REST API.
///
/// Reads its base URL from [ServerConfigService] on every call, so switching
/// server profile takes effect immediately with no re-wiring.
class ApiClient {
  ApiClient(this._servers, this._auth);

  final ServerConfigService _servers;
  final AuthService _auth;

  String get _base => _servers.current.apiBase;

  Future<Map<String, String>> _headers() async {
    await _auth.ensureFresh();
    final token = _auth.accessToken;
    return {
      'Accept': 'application/json',
      if (token != null) 'Authorization': 'Bearer $token',
    };
  }

  Future<dynamic> _get(String path) async {
    final http.Response response;
    try {
      response = await http
          .get(Uri.parse('$_base$path'), headers: await _headers())
          .timeout(const Duration(seconds: 20));
    } catch (e) {
      throw ApiException('Tidak bisa menjangkau server di $_base');
    }

    if (response.statusCode == 401) {
      await _auth.logout();
      throw ApiException('Sesi berakhir, silakan login ulang', statusCode: 401);
    }
    if (response.statusCode >= 400) {
      throw ApiException('Server menjawab HTTP ${response.statusCode}', statusCode: response.statusCode);
    }
    return jsonDecode(response.body);
  }

  Future<List<Camera>> listCameras() async {
    final data = await _get('/api/cameras');
    if (data is! List) throw ApiException('Balasan daftar kamera tidak sesuai');
    return data.whereType<Map<String, dynamic>>().map(Camera.fromJson).toList()
      ..sort((a, b) => a.name.compareTo(b.name));
  }

  /// Ask the backend to spin up the HLS pipeline for a camera.
  ///
  /// Best-effort on purpose: the stream is usually already running, and a
  /// failure here should not stop us from trying to play the playlist.
  Future<void> startHls(String cameraId) async {
    try {
      await http
          .post(Uri.parse('$_base/api/hls/$cameraId/start'), headers: await _headers())
          .timeout(const Duration(seconds: 15));
    } catch (_) {
      // Ignored: playback below will surface a real problem.
    }
  }

  /// HLS playlist URL for the player. ExoPlayer fetches this directly, so the
  /// token has to ride along as a query parameter rather than a header.
  Uri hlsPlaylist(String cameraId) => Uri.parse('$_base/api/hls/$cameraId/stream.m3u8');

  /// Single JPEG frame — cheap enough to use as a live thumbnail.
  Uri snapshot(String cameraId) =>
      Uri.parse('$_base/api/cameras/$cameraId/snapshot?t=${DateTime.now().millisecondsSinceEpoch}');

  Future<Map<String, String>> imageHeaders() => _headers();
}
