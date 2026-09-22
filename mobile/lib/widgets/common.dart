import 'package:flutter/material.dart';

import '../models/camera.dart';
import '../services/auth_service.dart';
import '../services/server_config.dart';
import '../theme.dart';

/// Tablets (and big foldables) get the side-by-side layouts. Same APK.
bool isTablet(BuildContext context) => MediaQuery.sizeOf(context).shortestSide >= 600;

bool isLandscape(BuildContext context) {
  final size = MediaQuery.sizeOf(context);
  return size.width > size.height;
}

Color statusColor(String status) => switch (status.toLowerCase()) {
      'online' => NvrColors.online,
      'connecting' => NvrColors.warning,
      _ => NvrColors.offline,
    };

class StatusDot extends StatelessWidget {
  const StatusDot({super.key, required this.color, this.size = 8});
  final Color color;
  final double size;

  @override
  Widget build(BuildContext context) => Container(
        height: size,
        width: size,
        decoration: BoxDecoration(color: color, shape: BoxShape.circle),
      );
}

/// Camera name + status dot, drawn over video.
class CameraLabel extends StatelessWidget {
  const CameraLabel({super.key, required this.camera, this.color, this.small = false});
  final Camera camera;
  final Color? color;
  final bool small;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.symmetric(horizontal: small ? 6 : 8, vertical: small ? 2 : 4),
      decoration: BoxDecoration(color: Colors.black54, borderRadius: BorderRadius.circular(4)),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          StatusDot(color: color ?? statusColor(camera.status), size: small ? 6 : 8),
          SizedBox(width: small ? 4 : 6),
          Flexible(
            child: Text(
              camera.name,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(color: Colors.white, fontSize: small ? 10 : 12, fontWeight: FontWeight.w500),
            ),
          ),
        ],
      ),
    );
  }
}

/// User + server, and logout. Shown in every top-level screen's app bar.
class AccountMenu extends StatelessWidget {
  const AccountMenu({super.key, required this.auth, required this.servers});
  final AuthService auth;
  final ServerConfigService servers;

  @override
  Widget build(BuildContext context) {
    return PopupMenuButton<String>(
      tooltip: 'Akun',
      icon: const Icon(Icons.account_circle_outlined),
      onSelected: (value) {
        if (value == 'logout') auth.logout();
      },
      itemBuilder: (context) => [
        PopupMenuItem(
          enabled: false,
          child: Text(
            '${auth.username ?? 'user'} · ${servers.current.name}\n${servers.current.apiBase}',
            style: const TextStyle(fontSize: 12, color: NvrColors.textSecondary),
          ),
        ),
        const PopupMenuDivider(),
        const PopupMenuItem(value: 'logout', child: Text('Keluar')),
      ],
    );
  }
}
