use super::identity::ProcessIdentity;
pub use kinekino_core::platform::process::FamilyObservation;
#[derive(Clone, Debug)]
pub struct ProcessReceipt {
    pub identity: ProcessIdentity,
    pub child_exit: Option<u32>,
    pub family: FamilyObservation,
    pub forced: bool,
    pub stdin_eof_requested: bool,
}
#[derive(Clone, Copy, Debug)]
pub enum FailureStage {
    StdinEof,
    ChildObservation,
    FamilyObservation,
    RawOutput,
    Deadline,
    Termination,
    Receipt,
}
#[derive(Debug)]
pub struct FailureCause {
    pub stage: FailureStage,
    pub error: std::io::Error,
}
