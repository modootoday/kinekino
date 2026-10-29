//! Untrusted source-worker framing, never browser admission or physical-close proof.
#[path = "application/codec.rs"]
pub mod codec;
pub mod domain;
#[cfg(feature = "metadata")]
pub mod http_metadata;
#[cfg(feature = "http-worker")]
pub mod http_response;
#[cfg(feature = "http-worker")]
pub mod http_send;
#[cfg(all(windows, feature = "parent-host"))]
pub mod parent_host;
#[cfg(all(windows, feature = "parent-host"))]
pub mod parent_launch;
#[cfg(feature = "parent-stream")]
pub mod parent_stream;
#[cfg(all(windows, feature = "parent-host"))]
pub mod parent_transfer;
pub mod plugin_host;
#[cfg(feature = "request")]
pub mod request;
pub mod transport;
#[cfg(feature = "worker-buffer")]
pub mod worker_buffer;
#[cfg(all(windows, feature = "worker-ipc"))]
pub mod worker_ipc;
#[cfg(all(windows, feature = "worker-ipc"))]
pub mod worker_loop;
#[cfg(feature = "http-worker")]
pub mod worker_session;
