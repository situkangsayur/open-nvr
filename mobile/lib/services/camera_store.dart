import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../models/camera.dart';
import 'api_client.dart';

/// The camera list, shared by the live grid, the camera list and recordings,
/// so the three tabs never disagree about what exists or what is online.
///
/// Polls every 30 s while the app is in the foreground so status dots follow
/// cameras going on- and offline.
class CameraStore extends ChangeNotifier {
  CameraStore(this.api);

  final ApiClient api;

  List<Camera> _cameras = const [];
  bool _loading = false;
  bool _loadedOnce = false;
  String? _error;
  Timer? _poll;

  List<Camera> get cameras => _cameras;
  bool get loading => _loading;
  bool get loadedOnce => _loadedOnce;
  String? get error => _error;
  int get onlineCount => _cameras.where((c) => c.isOnline).length;

  Camera? byId(String id) {
    for (final c in _cameras) {
      if (c.id == id) return c;
    }
    return null;
  }

  Future<void> refresh({bool quiet = false}) async {
    if (_loading) return;
    _loading = true;
    if (!quiet) {
      _error = null;
      notifyListeners();
    }
    try {
      _cameras = await api.listCameras();
      _error = null;
      _loadedOnce = true;
    } on ApiException catch (e) {
      // A failed background poll keeps the last good list on screen.
      if (!quiet || _cameras.isEmpty) _error = e.message;
    } finally {
      _loading = false;
      notifyListeners();
    }
  }

  void startPolling() {
    _poll?.cancel();
    _poll = Timer.periodic(const Duration(seconds: 30), (_) => refresh(quiet: true));
  }

  void stopPolling() {
    _poll?.cancel();
    _poll = null;
  }

  @override
  void dispose() {
    stopPolling();
    super.dispose();
  }
}

/// Small UI preferences that should survive a restart.
class ViewPrefs {
  static const _gridKey = 'nvr_grid_layout';

  static const gridChoices = [1, 4, 9, 16];

  static Future<int?> loadGrid() async {
    final prefs = await SharedPreferences.getInstance();
    final v = prefs.getInt(_gridKey);
    return gridChoices.contains(v) ? v : null;
  }

  static Future<void> saveGrid(int tiles) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setInt(_gridKey, tiles);
  }
}
