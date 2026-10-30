//! The engine port the controller drives; libmpv implements it here.
use kinekino_core::playback::VideoSurface;
use kinekino_core::port::error::PortResult;

/// What a playback engine offers beyond frames (libmpv in production, fakes in tests).
pub trait Engine: VideoSurface + 'static {
    fn load(&mut self, source: &str, start_secs: f64) -> PortResult<()>;
    fn command(&mut self, args: &[&str]) -> PortResult<()>;
    fn property(&mut self, name: &str) -> Option<f64>;
}

impl Engine for kinekino_libmpv::LibMpvSurface {
    fn load(&mut self, source: &str, start_secs: f64) -> PortResult<()> {
        self.open(source, start_secs)
    }
    fn command(&mut self, args: &[&str]) -> PortResult<()> {
        kinekino_libmpv::LibMpvSurface::command(self, args)
    }
    fn property(&mut self, name: &str) -> Option<f64> {
        self.property_f64(name)
    }
}
