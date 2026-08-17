import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:http/http.dart' as http;

import 'server_config.dart';

/// Raised when Keycloak rejects a login, with a message already translated
/// into something worth showing the user.
class AuthException implements Exception {
  AuthException(this.message, {this.isNetworkError = false});
  final String message;
  final bool isNetworkError;
  @override
  String toString() => message;
}

/// Keycloak authentication via the direct-access (password) grant.
///
/// Tokens live in the Android Keystore rather than shared preferences, since
/// an access token is a bearer credential for the whole NVR.
class AuthService extends ChangeNotifier {
  AuthService(this._servers);

  final ServerConfigService _servers;
  static const _storage = FlutterSecureStorage();
  static const _accessKey = 'nvr_access_token';
  static const _refreshKey = 'nvr_refresh_token';
  static const _realm = 'opennvr';
  static const _clientId = 'opennvr-frontend';

  String? _accessToken;
  String? _refreshToken;
  Map<String, dynamic>? _claims;
  DateTime? _expiresAt;

  bool get isAuthenticated => _accessToken != null && !_isExpired;
  String? get accessToken => _accessToken;
  String? get username => _claims?['preferred_username'] as String?;

  /// Realm roles, used to hide controls the account cannot use anyway.
  List<String> get roles {
    final realmAccess = _claims?['realm_access'];
    if (realmAccess is Map && realmAccess['roles'] is List) {
      return (realmAccess['roles'] as List).whereType<String>().toList();
    }
    return const [];
  }

  bool get _isExpired => _expiresAt == null || DateTime.now().isAfter(_expiresAt!);

  Uri _tokenUri() =>
      Uri.parse('${_servers.current.keycloakBase}/realms/$_realm/protocol/openid-connect/token');

  /// Restore a session saved on a previous run, refreshing if it has expired.
  Future<void> restore() async {
    try {
      final access = await _storage.read(key: _accessKey);
      final refresh = await _storage.read(key: _refreshKey);
      if (access == null) return;

      _applyAccessToken(access, refresh);
      if (_isExpired) {
        if (refresh != null) {
          await _refresh(refresh);
        } else {
          await logout();
        }
      }
    } catch (e) {
      debugPrint('Could not restore session: $e');
      await logout();
    }
    notifyListeners();
  }

  Future<void> login(String username, String password) async {
    http.Response response;
    try {
      response = await http
          .post(
            _tokenUri(),
            headers: {'Content-Type': 'application/x-www-form-urlencoded'},
            body: {
              'grant_type': 'password',
              'client_id': _clientId,
              'username': username,
              'password': password,
              'scope': 'openid profile email',
            },
          )
          .timeout(const Duration(seconds: 15));
    } catch (e) {
      // Never reached a server: the address is wrong or the tunnel is down.
      // Say that, rather than letting it look like a bad password.
      throw AuthException(
        'Tidak bisa menjangkau server login di ${_servers.current.keycloakBase}. '
        'Cek pengaturan Server.',
        isNetworkError: true,
      );
    }

    if (response.statusCode != 200) {
      throw AuthException(_describeError(response));
    }

    final body = jsonDecode(response.body) as Map<String, dynamic>;
    final access = body['access_token'] as String?;
    if (access == null) throw AuthException('Server tidak mengirim access token');

    _applyAccessToken(access, body['refresh_token'] as String?);
    await _persist();
    notifyListeners();
  }

  /// Turn a Keycloak error body into a message that says what to do next.
  String _describeError(http.Response response) {
    String detail = '';
    try {
      final body = jsonDecode(response.body) as Map<String, dynamic>;
      detail = (body['error_description'] ?? body['error'] ?? '') as String;
    } catch (_) {
      detail = response.body;
    }

    if (detail.contains('Invalid user credentials') || detail.contains('invalid_grant')) {
      return 'Username atau password salah';
    }
    if (detail.contains('Account is not fully set up')) {
      return 'Akun belum selesai disiapkan. Selesaikan lewat web dulu.';
    }
    if (detail.contains('not allowed for direct access')) {
      return 'Login langsung belum diaktifkan untuk client ini';
    }
    if (detail.contains('Realm does not exist')) {
      return 'Realm "$_realm" tidak ada — periksa port Keycloak';
    }
    return detail.isEmpty ? 'Login gagal (HTTP ${response.statusCode})' : detail;
  }

  /// Exchange a refresh token for a fresh access token.
  /// Returns false when the session can no longer be renewed.
  Future<bool> _refresh(String refreshToken) async {
    try {
      final response = await http
          .post(
            _tokenUri(),
            headers: {'Content-Type': 'application/x-www-form-urlencoded'},
            body: {
              'grant_type': 'refresh_token',
              'client_id': _clientId,
              'refresh_token': refreshToken,
            },
          )
          .timeout(const Duration(seconds: 15));

      if (response.statusCode == 200) {
        final body = jsonDecode(response.body) as Map<String, dynamic>;
        final access = body['access_token'] as String?;
        if (access != null) {
          _applyAccessToken(access, body['refresh_token'] as String? ?? refreshToken);
          await _persist();
          notifyListeners();
          return true;
        }
      }
    } catch (e) {
      debugPrint('Token refresh failed: $e');
    }
    await logout();
    return false;
  }

  /// Renew the session if the access token is close to expiry.
  /// Callers use this before a request rather than relying on a timer, so a
  /// phone waking from sleep does not fire requests with a stale token.
  Future<bool> ensureFresh() async {
    if (_accessToken == null) return false;
    final expiry = _expiresAt;
    if (expiry != null && DateTime.now().isBefore(expiry.subtract(const Duration(seconds: 30)))) {
      return true;
    }
    final refresh = _refreshToken;
    if (refresh == null) return false;
    return _refresh(refresh);
  }

  void _applyAccessToken(String accessToken, String? refreshToken) {
    _accessToken = accessToken;
    _refreshToken = refreshToken;
    _claims = _decodeJwtPayload(accessToken);
    final exp = _claims?['exp'];
    _expiresAt = exp is int ? DateTime.fromMillisecondsSinceEpoch(exp * 1000) : null;
  }

  /// Decode the JWT payload for its claims. This is not a signature check —
  /// the backend validates the token; here we only need username, roles, exp.
  Map<String, dynamic>? _decodeJwtPayload(String token) {
    try {
      final parts = token.split('.');
      if (parts.length < 2) return null;
      final payload = utf8.decode(base64Url.decode(base64Url.normalize(parts[1])));
      return jsonDecode(payload) as Map<String, dynamic>;
    } catch (e) {
      debugPrint('Could not decode token payload: $e');
      return null;
    }
  }

  Future<void> _persist() async {
    if (_accessToken != null) {
      await _storage.write(key: _accessKey, value: _accessToken);
    }
    if (_refreshToken != null) {
      await _storage.write(key: _refreshKey, value: _refreshToken);
    }
  }

  Future<void> logout() async {
    _accessToken = null;
    _refreshToken = null;
    _claims = null;
    _expiresAt = null;
    await _storage.delete(key: _accessKey);
    await _storage.delete(key: _refreshKey);
    notifyListeners();
  }
}
