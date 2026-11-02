//! The engine port the controller drives; libmpv implements it here.
use kinekino_core::playback::VideoSurface;
use kinekino_core::port::error::PortResult;

/// What a playback engine offers beyond frames (libmpv in production, fakes in tests).
pub trait Engine: VideoSurface + 'static {
    /// Starts opening `source` paused at `start_secs` and returns without waiting for it.
    fn begin_load(&mut self, source: &str, start_secs: f64) -> PortResult<()>;
    /// Whether the open started by `begin_load` has its first frame: Ok(false) while still
    /// opening, an error when it failed. Called on the UI thread, so it must not block.
    fn load_ready(&mut self) -> PortResult<bool>;
    fn command(&mut self, args: &[&str]) -> PortResult<()>;
    fn property(&mut self, name: &str) -> Option<f64>;
    /// String properties (`sub-text`); engines without subtitles report none.
    fn text(&mut self, _name: &str) -> Option<String> {
        None
    }
}

impl Engine for kinekino_libmpv::LibMpvSurface {
    fn begin_load(&mut self, source: &str, start_secs: f64) -> PortResult<()> {
        self.begin_open(source, start_secs)
    }
    fn load_ready(&mut self) -> PortResult<bool> {
        self.open_ready()
    }
    fn command(&mut self, args: &[&str]) -> PortResult<()> {
        kinekino_libmpv::LibMpvSurface::command(self, args)
    }
    fn property(&mut self, name: &str) -> Option<f64> {
        self.property_f64(name)
    }
    fn text(&mut self, name: &str) -> Option<String> {
        self.property_string(name)
    }
}
