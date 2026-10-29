//! ai context: `InferenceBackend` port (P1b), settings (`settings.inference`), cost guard,
//! upload consent and the in-process `MockBackend`. Adapters (HF cloud, local packs) live in
//! their own crates and register into [`registry::inference_registry`].
pub mod budget;
pub mod consent;
pub mod executor;
pub mod mock;
pub mod port;
pub mod registry;
pub mod settings;

pub use budget::{CostGuard, GuardDecision, LedgerEntry};
pub use consent::{Consent, ConsentScope};
pub use port::{
    Chunk, InferenceBackend, InferenceCaps, InferenceError, InferenceKind, InferenceRequest,
    InferenceResponse, Provenance, Task,
};
pub use settings::{InferenceMode, InferenceSettings};
