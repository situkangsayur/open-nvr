import 'package:flutter/foundation.dart';

@immutable
class Camera {
  const Camera({
    required this.id,
    required this.name,
    required this.status,
    this.brand,
    this.model,
    this.protocolType,
    this.streamUrl,
    this.ptzCapable = false,
    this.audioCapable = false,
    this.recordingMode,
  });

  final String id;
  final String name;
  final String status;
  final String? brand;
  final String? model;
  final String? protocolType;
  final String? streamUrl;
  final bool ptzCapable;
  final bool audioCapable;
  final String? recordingMode;

  bool get isOnline => status.toLowerCase() == 'online';
  bool get isRecording => recordingMode != null && recordingMode != 'disabled';

  /// Human label for the status chip.
  String get statusLabel => switch (status.toLowerCase()) {
        'online' => 'Online',
        'offline' => 'Offline',
        'connecting' => 'Menyambung',
        'error' => 'Error',
        _ => status,
      };

  factory Camera.fromJson(Map<String, dynamic> json) => Camera(
        id: json['id'] as String,
        name: json['name'] as String? ?? 'Tanpa nama',
        status: json['status'] as String? ?? 'unknown',
        brand: json['brand'] as String?,
        model: json['model'] as String?,
        protocolType: json['protocol_type'] as String?,
        streamUrl: json['stream_url'] as String?,
        ptzCapable: json['ptz_capable'] as bool? ?? false,
        audioCapable: json['audio_capable'] as bool? ?? false,
        recordingMode: json['recording_mode'] as String?,
      );
}
