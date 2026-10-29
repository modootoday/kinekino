//! Browser engine port: the only surface the player/CLI may depend on.
mod capability;
mod error;
mod event;
mod kind;
mod network;

pub use capability::{Capability, CapabilitySet};
pub use error::{EngineError, EngineResult};
pub use event::BrowserEvent;
pub use kind::EngineKind;
pub use network::{InterceptedRequest, NetworkInterceptor, RequestDecision};

/// Navigation, script and event surface of one browser view.
///
/// Contract: every capability in `capabilities()` works; an undeclared API capability
/// returns `EngineError::Unsupported` (or `None` from the accessors below).
pub trait BrowserEngine {
    fn kind(&self) -> EngineKind;
    /// Declared capability table; defaults to the ADR 0002 table for `kind()`.
    fn capabilities(&self) -> CapabilitySet {
        self.kind().declared()
    }
    fn navigate(&mut self, url: &str) -> EngineResult<()>;
    /// Evaluates an expression in the current document and returns its JSON value.
    fn eval(&mut self, script: &str) -> EngineResult<serde_json::Value>;
    /// Registers a script that runs before page scripts on every new document.
    fn inject_script(&mut self, source: &str) -> EngineResult<()>;
    /// Installs the request filter; replaces a previous one.
    fn set_interceptor(&mut self, interceptor: Box<dyn NetworkInterceptor>) -> EngineResult<()>;
    /// Drains events observed since the last call.
    fn drain_events(&mut self) -> Vec<BrowserEvent>;
    /// Audio control when `AudioMute` is declared.
    fn audio(&mut self) -> Option<&mut dyn AudioControl> {
        None
    }
    /// Viewport capture when `FrameCapture` is declared.
    fn frame_capture(&mut self) -> Option<&mut dyn FrameCapture> {
        None
    }
}

/// Whole-view audio output control.
pub trait AudioControl {
    fn set_muted(&mut self, muted: bool) -> EngineResult<()>;
    fn is_muted(&mut self) -> EngineResult<bool>;
}

/// Pixel capture of the current viewport.
pub trait FrameCapture {
    fn capture_png(&mut self) -> EngineResult<Vec<u8>>;
}
