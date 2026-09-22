import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:open_nvr_mobile/models/camera.dart';
import 'package:open_nvr_mobile/models/recording.dart';
import 'package:open_nvr_mobile/services/server_config.dart';
import 'package:open_nvr_mobile/theme.dart';

void main() {
  group('ServerProfile', () {
    test('builds API and Keycloak base URLs from host and ports', () {
      const profile = ServerProfile(name: 'Rumah', host: '192.168.1.10');
      expect(profile.apiBase, 'http://192.168.1.10:8888');
      expect(profile.keycloakBase, 'http://192.168.1.10:8080');
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
      final profiles = ServerConfigService.defaultProfiles;
      expect(profiles, hasLength(2));
      expect(profiles.map((p) => p.name), containsAll(['Rumah (LAN)', 'Remote (WireGuard)']));
      // The addresses themselves come from --dart-define, so assert only that
      // each profile ends up with a usable host.
      expect(profiles.every((p) => p.host.isNotEmpty), isTrue);
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

  group('Recordings', () {
    test('parses a segment and converts its times to local', () {
      final f = RecordingFile.fromJson(const {
        'filename': '20260922_101500.mp4',
        'start': '2026-09-22T10:15:00Z',
        'end': '2026-09-22T10:20:00Z',
        'duration_secs': 300,
        'size_bytes': 1024,
        'url': '/api/recordings/files/cam/20260922_101500.mp4',
      });
      expect(f.start.isUtc, isFalse);
      expect(f.start.toUtc(), DateTime.utc(2026, 9, 22, 10, 15));
      expect(f.duration, const Duration(minutes: 5));
      expect(f.complete, isTrue, reason: 'older servers omit the flag');
    });

    test('marks the segment still being written', () {
      final f = RecordingFile.fromJson(const {
        'start': '2026-09-22T10:15:00Z',
        'end': '2026-09-22T10:17:00Z',
        'url': '/x.mp4',
        'complete': false,
      });
      expect(f.complete, isFalse);
    });

    test('reads a day as local midnight', () {
      final d = RecordingDay.fromJson(const {'date': '2026-09-22', 'count': 3, 'size_bytes': 10});
      expect(d.date, DateTime(2026, 9, 22));
      expect(d.count, 3);
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
