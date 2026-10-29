mod descriptor;
mod modules;
mod payload;
mod scores;
mod worker_outcome;
mod worker_request;
mod worker_response;

pub use descriptor::decode_descriptor;
pub use worker_outcome::decode_worker_outcome;
pub use worker_request::{ValidatedWorkerRequest, decode_worker_request};
pub use worker_response::encode_worker_response;
pub const MAX_DESCRIPTOR_BYTES: u64 = 16 * 1024;
pub const MAX_OUTCOME_BYTES: u64 = 128 * 1024;
pub const MAX_SCORES: usize =
    crate::domain::MAX_SAMPLES.div_ceil(crate::domain::CHUNK_SAMPLES) as usize;
pub const MAX_MODULES: usize = 32;
