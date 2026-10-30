use super::*;
use crate::{application::api::PlayerSdk, infrastructure::local_host::TransportOptions};
fn host() -> ReviewTransportHost {
    start_transport(
        PlayerSdk::default(),
        super::super::transport::tests::sessions(),
        TransportOptions::default(),
    )
    .unwrap()
}
fn completed(outcome: ReviewStdioOutcome) -> ReviewStdioCompletion {
    match outcome {
        ReviewStdioOutcome::Complete(done) => done,
        ReviewStdioOutcome::Incomplete { .. } => panic!("empty host cleanup incomplete"),
    }
}
struct NeverRead;
impl Read for NeverRead {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        panic!("ready failure must skip input")
    }
}
struct FailWriter {
    flush: bool,
    bytes: Vec<u8>,
}
impl Write for FailWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.flush {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        } else {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::ErrorKind::BrokenPipe.into())
    }
}
struct FailRead;
impl Read for FailRead {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::ErrorKind::Other.into())
    }
}
#[test]
fn clean_eof_requires_original_worker_joins_and_empty_session_completion() {
    let mut out = Vec::new();
    let done = completed(serve_stdio(
        host(),
        &serde_json::json!({"ready":true}),
        &mut &b""[..],
        &mut out,
    ));
    assert!(done.succeeded());
    assert!(done.shutdown.transport.is_quiescent());
    assert!(done.shutdown.sessions.complete);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&out).unwrap(),
        serde_json::json!({"ready":true})
    );
}
#[test]
fn ready_write_and_flush_failure_skip_input_and_keep_first_error_after_cleanup() {
    for flush in [false, true] {
        let mut out = FailWriter {
            flush,
            bytes: Vec::new(),
        };
        let done = completed(serve_stdio(
            host(),
            &serde_json::json!({"ready":true}),
            &mut NeverRead,
            &mut out,
        ));
        assert!(!done.succeeded());
        assert!(done.shutdown.transport.is_quiescent());
        assert!(done.shutdown.sessions.complete);
        assert_eq!(
            done.first_error().unwrap().details["phase"],
            if flush { "ready_flush" } else { "ready_write" }
        );
        assert!(done.shutdown.first_error.is_none());
        assert_eq!(done.first_error().unwrap().details["io_kind"], "BrokenPipe");
    }
}
#[test]
fn input_failure_is_not_replaced_by_successful_physical_cleanup() {
    let done = completed(serve_stdio(
        host(),
        &serde_json::json!({"ready":true}),
        &mut FailRead,
        &mut Vec::new(),
    ));
    assert!(!done.succeeded());
    assert!(done.shutdown.transport.is_quiescent());
    assert!(done.shutdown.sessions.complete);
    assert_eq!(done.first_error().unwrap().details["phase"], "input_read");
}
