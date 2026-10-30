use super::{frame_capture::is_absolute_local_path, identifiers::valid_sha256};
use serde::{Deserialize, Serialize};

/// Source pixels, top-left origin, with an exclusive right/bottom edge. No scaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PixelCrop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl PixelCrop {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.width == 0
            || self.height == 0
            || self.x.checked_add(self.width).is_none()
            || self.y.checked_add(self.height).is_none()
        {
            return Err("Crop must have positive dimensions and checked pixel edges");
        }
        Ok(())
    }
    pub fn validate_bounds(&self, width: u32, height: u32) -> Result<(), &'static str> {
        self.validate()?;
        if self
            .x
            .checked_add(self.width)
            .is_none_or(|edge| edge > width)
            || self
                .y
                .checked_add(self.height)
                .is_none_or(|edge| edge > height)
        {
            return Err("Crop exceeds the decoded source pixel bounds");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportSnapshot {
    pub source_path: String,
    pub expected_source_sha256: String,
    pub output_path: String,
    pub crop: PixelCrop,
}
impl ExportSnapshot {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.crop.validate()?;
        if !valid_sha256(&self.expected_source_sha256) {
            return Err("Expected source SHA256 must be canonical lowercase hexadecimal");
        }
        for path in [&self.source_path, &self.output_path] {
            if !is_absolute_local_path(path) || !path.to_ascii_lowercase().ends_with(".png") {
                return Err("Source and output must be absolute local PNG paths");
            }
        }
        if self.source_path == self.output_path {
            return Err("Output must be a distinct new PNG path");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crop_checks_positive_dimensions_overflow_and_exclusive_edges() {
        let crop = PixelCrop {
            x: 2,
            y: 1,
            width: 2,
            height: 3,
        };
        assert!(crop.validate_bounds(4, 4).is_ok());
        assert!(crop.validate_bounds(3, 4).is_err());
        assert!(PixelCrop { width: 0, ..crop }.validate().is_err());
        assert!(
            PixelCrop {
                x: u32::MAX,
                ..crop
            }
            .validate()
            .is_err()
        );
        assert!(
            PixelCrop {
                y: u32::MAX,
                ..crop
            }
            .validate()
            .is_err()
        );
    }
    #[test]
    fn request_requires_fixed_identity_and_distinct_absolute_png_paths() {
        let root = std::env::temp_dir();
        let mut request = ExportSnapshot {
            source_path: root.join("source.png").to_string_lossy().into(),
            expected_source_sha256: "a".repeat(64),
            output_path: root.join("crop.png").to_string_lossy().into(),
            crop: PixelCrop {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        };
        assert!(request.validate().is_ok());
        request.expected_source_sha256 = "A".repeat(64);
        assert!(request.validate().is_err());
        request.expected_source_sha256 = "a".repeat(64);
        request.output_path = request.source_path.clone();
        assert!(request.validate().is_err());
        request.output_path = root.join("crop.png").to_string_lossy().into();
        for path in [
            "relative.png",
            "https://host/source.png",
            r"\\host\share\source.png",
        ] {
            request.source_path = path.into();
            assert!(request.validate().is_err());
        }
    }
}
