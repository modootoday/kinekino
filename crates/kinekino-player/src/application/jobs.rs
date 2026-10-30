//! Job context's public application/domain contracts; request dispatch remains in api.
pub use kinekino_jobs::application::*;
pub use kinekino_jobs::domain::{
    CancelDisposition, JobFailure, JobId, JobRecord, JobState, VerifiedJobOutput,
};
