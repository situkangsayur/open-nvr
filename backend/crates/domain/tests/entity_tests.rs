use open_nvr_domain::entities::*;
use open_nvr_domain::value_objects::*;

#[test]
fn test_camera_creation() {
    let cam = Camera::new("Test".into(), ProtocolType::Rtsp, "rtsp://192.168.1.1/s".into());
    assert_eq!(cam.name, "Test");
    assert_eq!(cam.status, CameraStatus::Offline);
    assert_eq!(cam.recording_mode, RecordingMode::Continuous);
    assert!(!cam.ptz_capable);
    assert!(!cam.audio_capable);
}

#[test]
fn test_camera_wifi_timeouts() {
    let mut cam = Camera::new("Wifi".into(), ProtocolType::Rtsp, "rtsp://10.0.0.1/s".into());
    cam.connection_type = ConnectionType::Wifi;
    assert_eq!(cam.frame_timeout_secs(), 15);
    assert_eq!(cam.max_backoff_secs(), 90);

    cam.connection_type = ConnectionType::Ethernet;
    assert_eq!(cam.frame_timeout_secs(), 10);
    assert_eq!(cam.max_backoff_secs(), 60);
}

#[test]
fn test_recording_lifecycle() {
    let mut rec = Recording::new_continuous(uuid::Uuid::new_v4(), true);
    assert_eq!(rec.status, RecordingStatus::Recording);
    assert!(rec.end_time.is_none());
    assert!(rec.duration_secs().is_none());

    rec.complete();
    assert_eq!(rec.status, RecordingStatus::Completed);
    assert!(rec.end_time.is_some());
    assert!(rec.duration_secs().unwrap() >= 0);
}

#[test]
fn test_detection_zone_contains_point() {
    let zone = DetectionZone::new(
        uuid::Uuid::new_v4(),
        "Test Zone".into(),
        vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ],
    );
    assert!(zone.contains_point(0.5, 0.5));
    assert!(!zone.contains_point(1.5, 0.5));
    assert!(!zone.contains_point(-0.1, 0.5));
}

#[test]
fn test_stream_url_validation() {
    assert!(StreamUrl::new("rtsp://192.168.1.1/stream").is_ok());
    assert!(StreamUrl::new("http://camera.local/mjpeg").is_ok());
    assert!(StreamUrl::new("").is_err());
    assert!(StreamUrl::new("ftp://invalid").is_err());
}

#[test]
fn test_time_range_validation() {
    use chrono::{Duration, Utc};
    let now = Utc::now();
    let later = now + Duration::hours(1);

    assert!(TimeRange::new(now, later).is_ok());
    assert!(TimeRange::new(later, now).is_err());

    let range = TimeRange::new(now, later).unwrap();
    assert_eq!(range.duration_secs(), 3600);
    assert!(range.contains(now + Duration::minutes(30)));
    assert!(!range.contains(now - Duration::minutes(1)));
}

#[test]
fn test_camera_group_creation() {
    let group = CameraGroup::new("Outdoor".into(), Some("All outdoor cameras".into()));
    assert_eq!(group.name, "Outdoor");
    assert_eq!(group.description, Some("All outdoor cameras".into()));
}

#[test]
fn test_audit_log_builder() {
    let log = AuditLog::new("camera.create".into(), "camera".into(), Some("abc-123".into()))
        .with_user("user1".into(), Some("user@test.com".into()))
        .with_details(serde_json::json!({"key": "value"}));

    assert_eq!(log.action, "camera.create");
    assert_eq!(log.user_id, Some("user1".into()));
    assert_eq!(log.user_email, Some("user@test.com".into()));
}

#[test]
fn test_network_event_resolve() {
    let mut event = NetworkEvent::new(NetworkEventType::UnauthorizedAccess, Severity::Critical);
    assert!(!event.resolved);

    event.resolve("admin".into());
    assert!(event.resolved);
    assert!(event.resolved_at.is_some());
    assert_eq!(event.resolved_by, Some("admin".into()));
}

#[test]
fn test_retention_policy() {
    let policy = RetentionPolicy::new_global("Default".into(), 30);
    assert_eq!(policy.retention_days, 30);
    assert!(policy.camera_id.is_none());
    assert!(policy.enabled);
}

#[test]
fn test_grid_layout_creation() {
    let layout = GridLayout::new("user1".into(), "My Layout".into(), LayoutType::Grid);
    assert_eq!(layout.user_id, "user1");
    assert_eq!(layout.layout_type, LayoutType::Grid);
    assert!(layout.camera_positions.is_empty());
    assert!(!layout.is_default);
}

#[test]
fn test_recording_segment_storage_key() {
    let camera_id = uuid::Uuid::new_v4();
    let time = chrono::Utc::now();
    let key = RecordingSegment::storage_key_for(camera_id, time, 5);
    assert!(key.starts_with("recordings/"));
    assert!(key.ends_with("_5.mp4"));
    assert!(key.contains(&camera_id.to_string()));
}
