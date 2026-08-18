import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

/// One saved way of reaching an Open-NVR install.
///
/// The same server is reachable at different addresses depending on where the
/// phone is (home Wi-Fi vs. the WireGuard tunnel), so the app stores a list of
/// profiles and remembers which one was last used.
@immutable
class ServerProfile {
  const ServerProfile({
    required this.name,
    required this.host,
    this.apiPort = 8888,
    this.keycloakPort = 8080,
    this.useHttps = false,
  });

  final String name;
  final String host;
  final int apiPort;
  final int keycloakPort;
  final bool useHttps;

  String get scheme => useHttps ? 'https' : 'http';
  String get apiBase => '$scheme://$host:$apiPort';
  String get keycloakBase => '$scheme://$host:$keycloakPort';

  ServerProfile copyWith({
    String? name,
    String? host,
    int? apiPort,
    int? keycloakPort,
    bool? useHttps,
  }) =>
      ServerProfile(
        name: name ?? this.name,
        host: host ?? this.host,
        apiPort: apiPort ?? this.apiPort,
        keycloakPort: keycloakPort ?? this.keycloakPort,
        useHttps: useHttps ?? this.useHttps,
      );

  Map<String, dynamic> toJson() => {
        'name': name,
        'host': host,
        'apiPort': apiPort,
        'keycloakPort': keycloakPort,
        'useHttps': useHttps,
      };

  factory ServerProfile.fromJson(Map<String, dynamic> json) => ServerProfile(
        name: json['name'] as String? ?? 'Server',
        host: json['host'] as String? ?? '',
        apiPort: json['apiPort'] as int? ?? 8888,
        keycloakPort: json['keycloakPort'] as int? ?? 8080,
        useHttps: json['useHttps'] as bool? ?? false,
      );

  @override
  bool operator ==(Object other) =>
      other is ServerProfile &&
      other.name == name &&
      other.host == host &&
      other.apiPort == apiPort &&
      other.keycloakPort == keycloakPort &&
      other.useHttps == useHttps;

  @override
  int get hashCode => Object.hash(name, host, apiPort, keycloakPort, useHttps);
}

/// Result of probing a server's `/health` endpoint.
@immutable
class ConnectionResult {
  const ConnectionResult({required this.ok, required this.message, this.version});
  final bool ok;
  final String message;
  final String? version;
}

/// Holds the server profiles and the currently selected one.
class ServerConfigService extends ChangeNotifier {
  static const _profilesKey = 'nvr_server_profiles';
  static const _selectedKey = 'nvr_selected_profile';

  /// Seeded on first run so the login screen is not blank. Real addresses stay
  /// out of the repository: pass them at build time with
  /// `--dart-define=NVR_LAN_HOST=… --dart-define=NVR_VPN_HOST=…`, or just type
  /// the address on the login screen — it is remembered from then on.
  static const _lanHost = String.fromEnvironment('NVR_LAN_HOST', defaultValue: '192.168.1.10');
  static const _vpnHost = String.fromEnvironment('NVR_VPN_HOST', defaultValue: '10.0.0.10');

  static const defaultProfiles = <ServerProfile>[
    ServerProfile(name: 'Rumah (LAN)', host: _lanHost),
    ServerProfile(name: 'Remote (WireGuard)', host: _vpnHost),
  ];

  List<ServerProfile> _profiles = List.of(defaultProfiles);
  int _selectedIndex = 0;
  bool _loaded = false;

  List<ServerProfile> get profiles => List.unmodifiable(_profiles);
  bool get isLoaded => _loaded;
  int get selectedIndex => _selectedIndex;

  ServerProfile get current =>
      _profiles.isEmpty ? defaultProfiles.first : _profiles[_selectedIndex.clamp(0, _profiles.length - 1)];

  Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_profilesKey);

    if (raw != null) {
      try {
        final decoded = jsonDecode(raw) as List<dynamic>;
        final parsed = decoded
            .whereType<Map<String, dynamic>>()
            .map(ServerProfile.fromJson)
            .where((p) => p.host.isNotEmpty)
            .toList();
        if (parsed.isNotEmpty) _profiles = parsed;
      } catch (e) {
        debugPrint('Could not parse saved server profiles: $e');
      }
    }

    _selectedIndex = (prefs.getInt(_selectedKey) ?? 0).clamp(0, _profiles.length - 1);
    _loaded = true;
    notifyListeners();
  }

  Future<void> _persist() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(
      _profilesKey,
      jsonEncode(_profiles.map((p) => p.toJson()).toList()),
    );
    await prefs.setInt(_selectedKey, _selectedIndex);
  }

  Future<void> select(int index) async {
    if (index < 0 || index >= _profiles.length) return;
    _selectedIndex = index;
    await _persist();
    notifyListeners();
  }

  /// Replace the profile at [index], or append when [index] is out of range.
  Future<void> upsert(ServerProfile profile, {int? index}) async {
    if (index != null && index >= 0 && index < _profiles.length) {
      _profiles[index] = profile;
      _selectedIndex = index;
    } else {
      _profiles.add(profile);
      _selectedIndex = _profiles.length - 1;
    }
    await _persist();
    notifyListeners();
  }

  Future<void> remove(int index) async {
    if (_profiles.length <= 1 || index < 0 || index >= _profiles.length) return;
    _profiles.removeAt(index);
    _selectedIndex = _selectedIndex.clamp(0, _profiles.length - 1);
    await _persist();
    notifyListeners();
  }

  /// Probe `GET {apiBase}/health`.
  ///
  /// Checking the address before authenticating means a wrong host is reported
  /// as a wrong host, instead of showing up later as a puzzling login failure.
  Future<ConnectionResult> testConnection(ServerProfile profile) async {
    if (profile.host.trim().isEmpty) {
      return const ConnectionResult(ok: false, message: 'Alamat server masih kosong');
    }

    try {
      final response = await http
          .get(Uri.parse('${profile.apiBase}/health'))
          .timeout(const Duration(seconds: 8));

      if (response.statusCode != 200) {
        return ConnectionResult(ok: false, message: 'Server menjawab HTTP ${response.statusCode}');
      }

      final body = jsonDecode(response.body) as Map<String, dynamic>;
      if (body['status'] != 'ok') {
        return ConnectionResult(ok: false, message: 'Balasan tak terduga: ${response.body}');
      }
      return ConnectionResult(
        ok: true,
        message: 'Tersambung ke ${body['service']} v${body['version']}',
        version: body['version'] as String?,
      );
    } catch (e) {
      return ConnectionResult(
        ok: false,
        message: 'Tidak bisa menjangkau ${profile.apiBase}. '
            'Cek alamatnya, atau nyalakan WireGuard kalau sedang di luar rumah.',
      );
    }
  }
}
