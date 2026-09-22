import 'dart:async';

import 'package:flutter/material.dart';

import '../services/api_client.dart';
import '../services/auth_service.dart';
import '../services/camera_store.dart';
import '../services/server_config.dart';
import '../theme.dart';
import '../widgets/common.dart';
import 'camera_list_screen.dart';
import 'live_grid_screen.dart';
import 'recordings_screen.dart';

/// Everything a screen needs to talk to the server, passed as one value.
class AppServices {
  AppServices({required this.api, required this.auth, required this.servers, required this.store});
  final ApiClient api;
  final AuthService auth;
  final ServerConfigService servers;
  final CameraStore store;
}

/// Top level after login: Live grid · Kamera · Rekaman.
///
/// Phones get a bottom navigation bar; tablets (and phones held sideways, where
/// a bottom bar eats scarce height) get a navigation rail on the left.
class HomeShell extends StatefulWidget {
  const HomeShell({super.key, required this.api, required this.auth, required this.servers});

  final ApiClient api;
  final AuthService auth;
  final ServerConfigService servers;

  @override
  State<HomeShell> createState() => _HomeShellState();
}

class _HomeShellState extends State<HomeShell> with WidgetsBindingObserver {
  late final AppServices _services;
  int _tab = 0;
  Timer? _keepAlive;

  // Rotating a phone swaps bottom bar for rail, which moves the tab stack in
  // the tree; the key carries it (and any open stream or player) across.
  final _tabsKey = GlobalKey();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _services = AppServices(
      api: widget.api,
      auth: widget.auth,
      servers: widget.servers,
      store: CameraStore(widget.api),
    );
    _services.store.refresh();
    _startForeground();
  }

  void _startForeground() {
    _services.store.startPolling();
    // Keycloak access tokens are short-lived (minutes). Refresh ahead of
    // expiry so live views and players always get a valid token; the players
    // are pushed the new one through the auth listener.
    _keepAlive?.cancel();
    _keepAlive = Timer.periodic(const Duration(seconds: 20), (_) => widget.auth.ensureFresh());
  }

  void _stopForeground() {
    _services.store.stopPolling();
    _keepAlive?.cancel();
    _keepAlive = null;
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      widget.auth.ensureFresh();
      _services.store.refresh(quiet: true);
      _startForeground();
    } else if (state == AppLifecycleState.paused) {
      _stopForeground();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _stopForeground();
    _services.store.dispose();
    super.dispose();
  }

  void _select(int index) => setState(() => _tab = index);

  static const _destinations = [
    (icon: Icons.grid_view_outlined, selected: Icons.grid_view, label: 'Live'),
    (icon: Icons.videocam_outlined, selected: Icons.videocam, label: 'Kamera'),
    (icon: Icons.video_library_outlined, selected: Icons.video_library, label: 'Rekaman'),
  ];

  @override
  Widget build(BuildContext context) {
    final useRail = isTablet(context) || isLandscape(context);

    final body = IndexedStack(
      key: _tabsKey,
      index: _tab,
      children: [
        LiveGridScreen(services: _services, active: _tab == 0),
        CameraListScreen(services: _services, active: _tab == 1),
        RecordingsScreen(services: _services, active: _tab == 2),
      ],
    );

    if (!useRail) {
      return Scaffold(
        body: body,
        bottomNavigationBar: NavigationBar(
          selectedIndex: _tab,
          onDestinationSelected: _select,
          backgroundColor: NvrColors.primaryDark,
          indicatorColor: NvrColors.primary,
          height: 64,
          destinations: [
            for (final d in _destinations)
              NavigationDestination(icon: Icon(d.icon), selectedIcon: Icon(d.selected), label: d.label),
          ],
        ),
      );
    }

    return Scaffold(
      body: Row(
        children: [
          SafeArea(
            right: false,
            child: NavigationRail(
              selectedIndex: _tab,
              onDestinationSelected: _select,
              backgroundColor: NvrColors.primaryDark,
              indicatorColor: NvrColors.primary,
              labelType: NavigationRailLabelType.all,
              leading: const Padding(
                padding: EdgeInsets.symmetric(vertical: 8),
                child: Icon(Icons.videocam, color: NvrColors.accent, size: 28),
              ),
              destinations: [
                for (final d in _destinations)
                  NavigationRailDestination(
                    icon: Icon(d.icon),
                    selectedIcon: Icon(d.selected),
                    label: Text(d.label),
                  ),
              ],
            ),
          ),
          const VerticalDivider(width: 1),
          Expanded(child: body),
        ],
      ),
    );
  }
}
