-- Migration: 002_user_permissions.sql
-- Per-user camera access and feature permissions

CREATE TABLE IF NOT EXISTS user_camera_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    camera_id UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    can_view BOOLEAN DEFAULT true,
    can_ptz BOOLEAN DEFAULT false,
    can_playback BOOLEAN DEFAULT true,
    can_export BOOLEAN DEFAULT false,
    granted_by VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE(user_id, camera_id)
);

CREATE INDEX IF NOT EXISTS idx_user_camera_access_user ON user_camera_access(user_id);
CREATE INDEX IF NOT EXISTS idx_user_camera_access_camera ON user_camera_access(camera_id);

COMMENT ON TABLE user_camera_access IS 'Per-user camera access permissions. Admin role bypasses this.';
