-- Migration: 001_initial.sql
-- Description: Initial schema for Open-NVR Phase 1
-- Database: PostgreSQL 16
-- Created: 2026-03-22

BEGIN;

-- =============================================================================
-- Trigger function: auto-update updated_at on row change
-- =============================================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- Table: camera_groups
-- =============================================================================
CREATE TABLE camera_groups (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE camera_groups IS
    'Logical groupings of cameras for organisation and bulk operations.';

CREATE TRIGGER trg_camera_groups_updated_at
    BEFORE UPDATE ON camera_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- Table: cameras
-- =============================================================================
CREATE TABLE cameras (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                  VARCHAR(255) NOT NULL,
    brand                 VARCHAR(100),
    model                 VARCHAR(100),
    protocol_type         VARCHAR(50)  NOT NULL DEFAULT 'rtsp'
        CHECK (protocol_type IN ('rtsp', 'onvif', 'mjpeg', 'rtmp', 'hls', 'p2p')),
    stream_url            TEXT         NOT NULL,
    sub_stream_url        TEXT,
    onvif_url             TEXT,
    credentials_encrypted BYTEA,
    ptz_capable           BOOLEAN      NOT NULL DEFAULT false,
    audio_capable         BOOLEAN      NOT NULL DEFAULT false,
    group_id              UUID         REFERENCES camera_groups (id),
    status                VARCHAR(50)  NOT NULL DEFAULT 'offline'
        CHECK (status IN ('online', 'offline', 'error', 'connecting')),
    connection_type       VARCHAR(20)  NOT NULL DEFAULT 'ethernet'
        CHECK (connection_type IN ('ethernet', 'wifi')),
    recording_mode        VARCHAR(20)  NOT NULL DEFAULT 'continuous'
        CHECK (recording_mode IN ('continuous', 'motion', 'disabled')),
    config                JSONB        NOT NULL DEFAULT '{}',
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE cameras IS
    'Registered IP cameras and their connection/configuration details.';

CREATE TRIGGER trg_cameras_updated_at
    BEFORE UPDATE ON cameras
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- Table: detection_zones
-- =============================================================================
CREATE TABLE detection_zones (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id       UUID         NOT NULL REFERENCES cameras (id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    polygon         JSONB        NOT NULL,   -- array of {x, y} points
    detection_types JSONB        NOT NULL DEFAULT '["motion"]',
    sensitivity     REAL         NOT NULL DEFAULT 0.5
        CHECK (sensitivity >= 0 AND sensitivity <= 1),
    enabled         BOOLEAN      NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE detection_zones IS
    'User-defined polygonal zones within a camera view used for targeted detection.';

CREATE TRIGGER trg_detection_zones_updated_at
    BEFORE UPDATE ON detection_zones
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- Table: recordings
-- =============================================================================
CREATE TABLE recordings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id       UUID         NOT NULL REFERENCES cameras (id) ON DELETE CASCADE,
    start_time      TIMESTAMPTZ  NOT NULL,
    end_time        TIMESTAMPTZ,
    recording_type  VARCHAR(20)  NOT NULL DEFAULT 'continuous'
        CHECK (recording_type IN ('continuous', 'motion')),
    has_audio       BOOLEAN      NOT NULL DEFAULT false,
    total_size      BIGINT       NOT NULL DEFAULT 0,
    status          VARCHAR(20)  NOT NULL DEFAULT 'recording'
        CHECK (status IN ('recording', 'completed', 'error')),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE recordings IS
    'Top-level recording sessions linked to a camera, spanning one or more segments.';

CREATE INDEX idx_recordings_camera_start
    ON recordings (camera_id, start_time);

-- =============================================================================
-- Table: recording_segments
-- =============================================================================
CREATE TABLE recording_segments (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recording_id     UUID         NOT NULL REFERENCES recordings (id) ON DELETE CASCADE,
    sequence_number  INT          NOT NULL,
    storage_key      TEXT         NOT NULL UNIQUE,
    start_time       TIMESTAMPTZ  NOT NULL,
    end_time         TIMESTAMPTZ,
    duration_ms      INT,
    size_bytes       BIGINT       NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE recording_segments IS
    'Individual file segments that compose a recording, stored by storage_key.';

CREATE INDEX idx_recording_segments_recording_seq
    ON recording_segments (recording_id, sequence_number);

-- =============================================================================
-- Table: detection_events
-- =============================================================================
CREATE TABLE detection_events (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id     UUID         NOT NULL REFERENCES cameras (id) ON DELETE CASCADE,
    zone_id       UUID         REFERENCES detection_zones (id) ON DELETE SET NULL,
    event_type    VARCHAR(50)  NOT NULL
        CHECK (event_type IN ('motion', 'human', 'animal', 'vehicle', 'unknown')),
    confidence    REAL
        CHECK (confidence >= 0 AND confidence <= 1),
    bounding_box  JSONB,       -- {x, y, w, h}
    thumbnail_key TEXT,
    metadata      JSONB        NOT NULL DEFAULT '{}',
    occurred_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE detection_events IS
    'Detected events (motion, object recognition) captured within camera views.';

CREATE INDEX idx_detection_events_camera_occurred
    ON detection_events (camera_id, occurred_at);

CREATE INDEX idx_detection_events_type_occurred
    ON detection_events (event_type, occurred_at);

-- =============================================================================
-- Table: grid_layouts
-- =============================================================================
CREATE TABLE grid_layouts (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          VARCHAR(255) NOT NULL,  -- Keycloak user ID
    name             VARCHAR(255) NOT NULL,
    layout_type      VARCHAR(50)  NOT NULL DEFAULT 'grid'
        CHECK (layout_type IN ('grid', 'single', 'l_shape', 'custom')),
    camera_positions JSONB        NOT NULL DEFAULT '[]',
    is_default       BOOLEAN      NOT NULL DEFAULT false,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT now(),

    UNIQUE (user_id, name)
);

COMMENT ON TABLE grid_layouts IS
    'Saved multi-camera grid/layout configurations per user for the live-view UI.';

CREATE TRIGGER trg_grid_layouts_updated_at
    BEFORE UPDATE ON grid_layouts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- Table: retention_policies
-- =============================================================================
CREATE TABLE retention_policies (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name              VARCHAR(255) NOT NULL,
    camera_id         UUID         REFERENCES cameras (id),  -- NULL = global policy
    retention_days    INT          NOT NULL DEFAULT 30,
    max_storage_bytes BIGINT,
    recording_type    VARCHAR(20)
        CHECK (recording_type IN ('all', 'continuous', 'motion')),
    enabled           BOOLEAN      NOT NULL DEFAULT true,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE retention_policies IS
    'Configurable data-retention rules per camera or globally, controlling automatic cleanup.';

CREATE TRIGGER trg_retention_policies_updated_at
    BEFORE UPDATE ON retention_policies
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- Table: audit_logs  (security / audit trail)
-- =============================================================================
CREATE TABLE audit_logs (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       VARCHAR(255),
    user_email    VARCHAR(255),
    action        VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id   VARCHAR(255),
    details       JSONB        NOT NULL DEFAULT '{}',
    ip_address    INET,
    user_agent    TEXT,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE audit_logs IS
    'Immutable audit trail capturing every user and system action for security compliance.';

CREATE INDEX idx_audit_logs_user_created
    ON audit_logs (user_id, created_at);

CREATE INDEX idx_audit_logs_resource
    ON audit_logs (resource_type, resource_id);

CREATE INDEX idx_audit_logs_action_created
    ON audit_logs (action, created_at);

-- =============================================================================
-- Table: network_events  (security / network monitoring)
-- =============================================================================
CREATE TABLE network_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type      VARCHAR(100) NOT NULL
        CHECK (event_type IN (
            'unauthorized_access',
            'port_scan',
            'brute_force',
            'unknown_device',
            'protocol_violation',
            'connection_anomaly'
        )),
    source_ip       INET,
    source_mac      VARCHAR(17),
    target_resource VARCHAR(255),
    severity        VARCHAR(20)  NOT NULL DEFAULT 'info'
        CHECK (severity IN ('info', 'warning', 'critical')),
    details         JSONB        NOT NULL DEFAULT '{}',
    resolved        BOOLEAN      NOT NULL DEFAULT false,
    resolved_at     TIMESTAMPTZ,
    resolved_by     VARCHAR(255),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE network_events IS
    'Security events detected on the local network such as scans, brute-force attempts, and anomalies.';

CREATE INDEX idx_network_events_type_created
    ON network_events (event_type, created_at);

CREATE INDEX idx_network_events_severity_created
    ON network_events (severity, created_at);

CREATE INDEX idx_network_events_source_ip_created
    ON network_events (source_ip, created_at);

COMMIT;
