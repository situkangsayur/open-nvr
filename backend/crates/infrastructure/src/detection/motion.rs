use open_nvr_domain::entities::{BoundingBox, Point};
use tracing::debug;

/// Simple motion detector using frame differencing.
/// Compares consecutive grayscale frames to detect motion regions.
pub struct MotionDetector {
    previous_frame: Option<Vec<u8>>,
    width: u32,
    height: u32,
    threshold: u8,
    min_area_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct MotionRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub intensity: f32,
}

impl MotionDetector {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            previous_frame: None,
            width,
            height,
            threshold: 25,
            min_area_ratio: 0.005, // 0.5% of frame must change
        }
    }

    pub fn with_threshold(mut self, threshold: u8) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_min_area_ratio(mut self, ratio: f32) -> Self {
        self.min_area_ratio = ratio;
        self
    }

    /// Convert raw frame data to grayscale (assumes RGB or RGBA input).
    /// If the frame is already grayscale (length == w*h), returns as-is.
    fn to_grayscale(&self, frame: &[u8]) -> Vec<u8> {
        let pixel_count = (self.width * self.height) as usize;

        if frame.len() == pixel_count {
            return frame.to_vec();
        }

        let channels = frame.len() / pixel_count;
        let mut gray = Vec::with_capacity(pixel_count);

        for i in 0..pixel_count {
            let offset = i * channels;
            if offset + 2 < frame.len() {
                let r = frame[offset] as u16;
                let g = frame[offset + 1] as u16;
                let b = frame[offset + 2] as u16;
                // Standard luminance conversion
                gray.push(((r * 77 + g * 150 + b * 29) >> 8) as u8);
            } else {
                gray.push(0);
            }
        }

        gray
    }

    /// Detect motion by comparing current frame with previous.
    /// Returns a list of motion regions if motion is detected.
    pub fn detect(&mut self, frame: &[u8]) -> Vec<MotionRegion> {
        let current = self.to_grayscale(frame);

        let regions = if let Some(ref previous) = self.previous_frame {
            self.compute_motion(previous, &current)
        } else {
            Vec::new()
        };

        self.previous_frame = Some(current);
        regions
    }

    fn compute_motion(&self, prev: &[u8], curr: &[u8]) -> Vec<MotionRegion> {
        let pixel_count = (self.width * self.height) as usize;
        if prev.len() != pixel_count || curr.len() != pixel_count {
            return Vec::new();
        }

        // Create diff mask
        let mut diff_mask = vec![false; pixel_count];
        let mut changed_pixels = 0u32;

        for i in 0..pixel_count {
            let diff = (prev[i] as i16 - curr[i] as i16).unsigned_abs() as u8;
            if diff > self.threshold {
                diff_mask[i] = true;
                changed_pixels += 1;
            }
        }

        let total_pixels = self.width * self.height;
        let change_ratio = changed_pixels as f32 / total_pixels as f32;

        if change_ratio < self.min_area_ratio {
            return Vec::new();
        }

        debug!(
            changed_pixels = changed_pixels,
            change_ratio = change_ratio,
            "Motion detected"
        );

        // Find bounding box of all changed pixels
        let mut min_x = self.width;
        let mut min_y = self.height;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                if diff_mask[idx] {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        if max_x <= min_x || max_y <= min_y {
            return Vec::new();
        }

        vec![MotionRegion {
            x: min_x,
            y: min_y,
            width: max_x - min_x + 1,
            height: max_y - min_y + 1,
            intensity: change_ratio,
        }]
    }

    /// Check if a motion region falls within a detection zone polygon.
    /// Uses normalized coordinates (0.0-1.0).
    pub fn region_in_zone(&self, region: &MotionRegion, polygon: &[Point]) -> bool {
        if polygon.len() < 3 {
            return true; // No valid zone = entire frame
        }

        // Check center of region
        let cx = (region.x as f64 + region.width as f64 / 2.0) / self.width as f64;
        let cy = (region.y as f64 + region.height as f64 / 2.0) / self.height as f64;

        point_in_polygon(cx, cy, polygon)
    }
}

/// Ray casting algorithm to check if point is inside polygon
fn point_in_polygon(px: f64, py: f64, polygon: &[Point]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    let mut j = n - 1;

    for i in 0..n {
        let pi = &polygon[i];
        let pj = &polygon[j];

        if ((pi.y > py) != (pj.y > py))
            && (px < (pj.x - pi.x) * (py - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }

    inside
}

impl MotionRegion {
    pub fn to_bounding_box(&self, frame_width: u32, frame_height: u32) -> BoundingBox {
        BoundingBox {
            x: self.x as f32 / frame_width as f32,
            y: self.y as f32 / frame_height as f32,
            width: self.width as f32 / frame_width as f32,
            height: self.height as f32 / frame_height as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_motion_identical_frames() {
        let mut detector = MotionDetector::new(4, 4);
        let frame = vec![128u8; 16];
        assert!(detector.detect(&frame).is_empty());
        assert!(detector.detect(&frame).is_empty());
    }

    #[test]
    fn test_motion_detected_different_frames() {
        let mut detector = MotionDetector::new(10, 10).with_min_area_ratio(0.01);
        let frame1 = vec![0u8; 100];
        let frame2 = vec![255u8; 100];
        detector.detect(&frame1);
        let regions = detector.detect(&frame2);
        assert!(!regions.is_empty());
        assert!(regions[0].intensity > 0.5);
    }

    #[test]
    fn test_below_threshold_no_motion() {
        let mut detector = MotionDetector::new(10, 10).with_threshold(50);
        let frame1 = vec![100u8; 100];
        let frame2 = vec![120u8; 100]; // diff = 20, below threshold of 50
        detector.detect(&frame1);
        let regions = detector.detect(&frame2);
        assert!(regions.is_empty());
    }

    #[test]
    fn test_point_in_polygon() {
        let polygon = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ];
        assert!(point_in_polygon(0.5, 0.5, &polygon));
        assert!(!point_in_polygon(1.5, 0.5, &polygon));
    }

    #[test]
    fn test_bounding_box_normalized() {
        let region = MotionRegion {
            x: 50,
            y: 100,
            width: 200,
            height: 150,
            intensity: 0.3,
        };
        let bbox = region.to_bounding_box(1920, 1080);
        assert!((bbox.x - 50.0 / 1920.0).abs() < 0.001);
        assert!((bbox.y - 100.0 / 1080.0).abs() < 0.001);
    }
}
