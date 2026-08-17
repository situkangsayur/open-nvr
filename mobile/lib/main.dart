import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'screens/camera_list_screen.dart';
import 'screens/login_screen.dart';
import 'services/api_client.dart';
import 'services/auth_service.dart';
import 'services/server_config.dart';
import 'theme.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: NvrColors.primaryDark,
      statusBarIconBrightness: Brightness.light,
    ),
  );

  final servers = ServerConfigService();
  await servers.load();

  final auth = AuthService(servers);
  await auth.restore();

  runApp(OpenNvrApp(servers: servers, auth: auth));
}

class OpenNvrApp extends StatelessWidget {
  const OpenNvrApp({super.key, required this.servers, required this.auth});

  final ServerConfigService servers;
  final AuthService auth;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Open-NVR',
      debugShowCheckedModeBanner: false,
      theme: buildNvrTheme(),
      home: _Gate(servers: servers, auth: auth),
    );
  }
}

/// Shows login or the camera list depending on auth state, and rebuilds when
/// either service changes — so a logout or an expired session lands the user
/// back on the login screen without any explicit navigation.
class _Gate extends StatelessWidget {
  const _Gate({required this.servers, required this.auth});

  final ServerConfigService servers;
  final AuthService auth;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: Listenable.merge([auth, servers]),
      builder: (context, _) {
        if (!auth.isAuthenticated) {
          return LoginScreen(servers: servers, auth: auth);
        }
        return CameraListScreen(
          api: ApiClient(servers, auth),
          auth: auth,
          servers: servers,
        );
      },
    );
  }
}
