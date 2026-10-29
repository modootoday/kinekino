//! `VideoSurface` port: pulls decoded frames as RGBA8 into a caller-owned buffer that the UI
//! uploads as a Meari texture. Adapters: libmpv software/GL render API (Unix, Windows).
use crate::port::error::{ErrorCode, PortError, PortResult};

/// Tightly packed RGBA8, row-major, `stride == width * 4`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RgbaFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl RgbaFrame {
    pub fn new(width: u32, height: u32) -> PortResult<Self> {
        if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
            return Err(PortError::new(
                ErrorCode::InvalidInput,
                "Frame size out of range",
            ));
        }
        Ok(Self {
            width,
            height,
            pixels: vec![0; width as usize * height as usize * 4],
        })
    }
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y as usize * self.width as usize + x as usize) * 4;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }
}

pub trait VideoSurface {
    /// True when a new frame is ready since the last `render` (cheap; UI polls per tick).
    fn frame_ready(&mut self) -> PortResult<bool>;
    /// Render the current frame scaled (aspect-preserving, black letterbox) into `target`.
    fn render(&mut self, target: &mut RgbaFrame) -> PortResult<()>;
}

/// Test double: a solid colour that changes on `push`.
#[derive(Debug, Default)]
pub struct FakeSurface {
    pub color: [u8; 4],
    ready: bool,
    pub renders: u64,
}

impl FakeSurface {
    pub fn push(&mut self, color: [u8; 4]) {
        self.color = color;
        self.ready = true;
    }
}

impl VideoSurface for FakeSurface {
    fn frame_ready(&mut self) -> PortResult<bool> {
        Ok(self.ready)
    }
    fn render(&mut self, target: &mut RgbaFrame) -> PortResult<()> {
        check_target(target)?;
        for px in target.pixels.as_chunks_mut::<4>().0 {
            *px = self.color;
        }
        self.ready = false;
        self.renders += 1;
        Ok(())
    }
}

/// Validate a caller buffer; adapters call this before writing.
pub fn check_target(target: &RgbaFrame) -> PortResult<()> {
    if target.width == 0
        || target.height == 0
        || target.pixels.len() != target.width as usize * target.height as usize * 4
    {
        return Err(PortError::new(
            ErrorCode::InvalidInput,
            "Frame buffer does not match its size",
        ));
    }
    Ok(())
}

/// Shared contract: after media is showing, a render fills an opaque frame and clears
/// readiness; a malformed buffer is `InvalidInput`.
pub fn check_surface(surface: &mut dyn VideoSurface) -> PortResult<()> {
    let fail = |m: &str| PortError::new(ErrorCode::Internal, format!("contract: {m}"));
    let mut bad = RgbaFrame {
        width: 4,
        height: 4,
        pixels: vec![0; 3],
    };
    if surface.render(&mut bad).map_err(|e| e.code) != Err(ErrorCode::InvalidInput) {
        return Err(fail("malformed buffer must be InvalidInput"));
    }
    let mut frame = RgbaFrame::new(64, 36)?;
    surface.render(&mut frame)?;
    if frame.pixels.as_chunks::<4>().0.iter().any(|p| p[3] != 255) {
        return Err(fail("rendered frame must be opaque"));
    }
    if surface.frame_ready()? {
        return Err(fail("render must consume readiness"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_surface_meets_contract() {
        let mut s = FakeSurface::default();
        s.push([10, 20, 30, 255]);
        assert!(s.frame_ready().unwrap());
        check_surface(&mut s).unwrap();
        let mut f = RgbaFrame::new(2, 2).unwrap();
        s.render(&mut f).unwrap();
        assert_eq!(f.pixel(1, 1), [10, 20, 30, 255]);
        assert!(RgbaFrame::new(0, 1).is_err());
    }
}
