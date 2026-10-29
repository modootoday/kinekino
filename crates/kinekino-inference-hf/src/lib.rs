//! Hugging Face cloud adapter for `InferenceBackend` (P1b). HTTP goes through
//! [`port::HttpTransport`]; this crate ships only [`loopback::LoopbackHttp`], which refuses
//! non-loopback hosts, so no real HF call is possible until the P7 gate adds a TLS transport.
mod backend;
pub mod loopback;
pub mod port;
mod wire;

pub use backend::{HF_CAPS, HF_ROUTER, HfCloudBackend, HfConfig, register_hf_cloud};
