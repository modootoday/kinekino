//! Bounded MSH1 incremental framing. No process/HTTP/replay authority.
mod receiver;
mod writer;
pub use receiver::FrameReceiver;
pub use writer::FrameWriter;
