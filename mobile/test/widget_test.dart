import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:open_nvr_mobile/models/camera.dart';
import 'package:open_nvr_mobile/services/server_config.dart';
import 'package:open_nvr_mobile/theme.dart';

void main() {
  group('ServerProfile', () {
    test('builds API and Keycloak base URLs from host and ports', () {
      const profile = ServerProfile(name: 'Rumah', host: '192.168.1.11');
      expect(profile.apiBase, 'http://192.168.1.11:8888');
      expect(profile.keycloakBase, 'http://192.168.1.11:8080');
    });

    test('honours the HTTPS flag on both endpoints', () {
      const profile = ServerProfile(
        name: 'Remote',
        host: 'nvr.example.com',
        apiPort: 443,
        keycloakPort: 8443,
        useHttps: true,
      );
      expect(profile.apiBase, 'https://nvr.example.com:443');
      expect(profile.keycloakBase, 'https://nvr.example.com:8443');
    });

    test('survives a JSON round trip', () {
      const original = ServerProfile(
        name: 'Remote (WireGuard)',
        host: '10.0.0.10',
        apiPort: 8888,
        keycloakPort: 8080,
      );
      expect(ServerProfile.fromJson(original.toJson()), original);
    });

    test('ships a home and a remote profile out of the box', () {
      final hosts = ServerConfigService.defaultProfiles.map((p) => p.host).toList();
      expect(hosts, contains('192.168.1.11'));
      expect(hosts, contains('10.0.0.10'));
    });
  });

  group('Camera', () {
    test('reads status and capabilities from the API payload', () {
      final camera = Camera.fromJson(const {
        'id': '31c116e2-c863-4b07-b43f-8fead4f4e310',
        'name': 'Device 192.168.1.14',
        'status': 'online',
        'protocol_type': 'rtsp',
        'stream_url': 'rtsp://192.168.1.14:554/live/ch00_1',
        'ptz_capable': false,
        'audio_capable': false,
        'recording_mode': 'continuous',
      });

      expect(camera.isOnline, isTrue);
      expect(camera.isRecording, isTrue);
      expect(camera.statusLabel, 'Online');
    });

    test('treats a disabled recording mode as not recording', () {
      final camera = Camera.fromJson(const {
        'id': 'a',
        'name': 'Cam',
        'status': 'offline',
        'recording_mode': 'disabled',
      });
      expect(camera.isOnline, isFalse);
      expect(camera.isRecording, isFalse);
      expect(camera.statusLabel, 'Offline');
    });

    test('falls back gracefully when optional fields are missing', () {
      final camera = Camera.fromJson(const {'id': 'a'});
      expect(camera.name, 'Tanpa nama');
      expect(camera.status, 'unknown');
      expect(camera.isRecording, isFalse);
    });
  });

  testWidgets('theme is dark with the sea-blue accent', (tester) async {
    final theme = buildNvrTheme();
    expect(theme.brightness, Brightness.dark);
    expect(theme.colorScheme.primary, NvrColors.accent);
    expect(theme.scaffoldBackgroundColor, NvrColors.background);

    await tester.pumpWidget(
      MaterialApp(theme: theme, home: const Scaffold(body: Text('Open-NVR'))),
    );
    expect(find.text('Open-NVR'), findsOneWidget);
  });
}
