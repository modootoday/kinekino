//! Purpose-specific bounded source transport. No HTTP policy, codec credit, or
//! physical family/receipt ACK is inferred from pipe completion or EOF.
pub const MAX_CONTROL_SEGMENT: usize = 8 * 1024;
pub const MAX_DATA_SEGMENT: usize = 64 * 1024;
pub use kinekino_core::platform::process::{
    CONTROL_ARGUMENT, DATA_ARGUMENT, END_ARGUMENT, OPERATION_ARGUMENT,
};

#[derive(Debug, PartialEq, Eq)]
pub enum IoPoll {
    Pending,
    Read(Vec<u8>),
    Written(usize),
    Eof,
    Cancelled,
}

pub use super::family::{SourceFamily, SourceLaunchFailure, spawn_source};
pub use super::source_channel::{
    ClosePoll, PreparedSourceIpc, SourceIpc, SourceLane, emergency_retained_operations,
    source_cutoffs, uptime_millis,
};

pub fn check_segment(length: usize, maximum: usize) -> std::io::Result<()> {
    if length == 0 || length > maximum || maximum > MAX_DATA_SEGMENT {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "source IPC segment bound",
        ))
    } else {
        Ok(())
    }
}
