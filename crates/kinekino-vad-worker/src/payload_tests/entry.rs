use super::*;
use crate::entry::{WorkerLaunch, serve_request, unsupported_execution};

fn request_raw(descriptor: &[u8]) -> Vec<u8> {
    let context: serde_json::Value = serde_json::from_slice(descriptor).unwrap();
    support::bytes(
        &serde_json::json!({"schema_version":"1","job_id":"parent-label","attempt":"1",
        "input_sha256":"a".repeat(64),"source_blob_sha256":"b".repeat(64),
        "submission_request_sha256":null,"descriptor_sha256":support::pin("descriptor",descriptor).sha256(),
        "context":context}),
    )
}

#[test]
fn production_unsupported_uses_validated_wire_without_runtime_observation() {
    let (request, descriptor, _) = inference_fixture();
    let raw = request_raw(&descriptor);
    let mut output = Vec::new();
    serve_request(
        raw.as_slice(),
        raw.len() as u64,
        &descriptor,
        request.expected().request(),
        &mut output,
        unsupported_execution,
    )
    .unwrap();
    let decoded = kinekino_vad::protocol::application::decode_worker_outcome(
        output.as_slice(),
        output.len() as u64,
        request.expected(),
    )
    .unwrap();
    assert!(matches!(
        decoded.result(),
        kinekino_vad::protocol::domain::WorkerResult::Failure {
            code: ErrorCode::Unsupported,
            ..
        }
    ));
    assert!(decoded.actual_runtime().is_none());
}

#[test]
fn test_only_engine_uses_same_entry_with_original_cancel_and_cutoff() {
    let (request, descriptor, source) = inference_fixture();
    let raw = request_raw(&descriptor);
    let control = Control::new();
    let cutoff = control.deadline;
    let engine = CooperativeEngine {
        original: &control,
        calls: Cell::new(0),
    };
    let mut output = Vec::new();
    serve_request(
        raw.as_slice(),
        raw.len() as u64,
        &descriptor,
        request.expected().request(),
        &mut output,
        |decoded| {
            crate::run::run_request(
                decoded,
                &Exact(decoded.expected().profile().clone()),
                &source,
                &engine,
                &control,
            )
        },
    )
    .unwrap();
    let decoded = kinekino_vad::protocol::application::decode_worker_outcome(
        output.as_slice(),
        output.len() as u64,
        request.expected(),
    )
    .unwrap();
    assert!(matches!(
        decoded.result(),
        kinekino_vad::protocol::domain::WorkerResult::Partial {
            processed_samples: 512,
            ..
        }
    ));
    assert_eq!(control.deadline, cutoff);
    assert!(control.cancel.get());
    assert_eq!(engine.calls.get(), 1);
}

#[test]
fn extra_stdin_bytes_are_rejected_before_dispatch_or_output() {
    let (request, descriptor, _) = inference_fixture();
    let mut raw = request_raw(&descriptor);
    let declared = raw.len() as u64;
    raw.push(b' ');
    let called = Cell::new(false);
    let mut output = Vec::new();
    let error = serve_request(
        raw.as_slice(),
        declared,
        &descriptor,
        request.expected().request(),
        &mut output,
        |_| {
            called.set(true);
            unsupported_execution(&request)
        },
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert!(!called.get());
    assert!(output.is_empty());
}

#[test]
fn other_original_request_pin_never_dispatches_or_writes() {
    let (_, descriptor, _) = inference_fixture();
    let raw = request_raw(&descriptor);
    let called = Cell::new(false);
    let mut output = Vec::new();
    let error = serve_request(
        raw.as_slice(),
        raw.len() as u64,
        &descriptor,
        &support::pin("other", &raw),
        &mut output,
        |_| {
            called.set(true);
            unreachable!()
        },
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert!(!called.get());
    assert!(output.is_empty());
}

struct FlushFailure(Vec<u8>);
impl std::io::Write for FlushFailure {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::other("fixture flush"))
    }
}
#[test]
fn stdout_flush_failure_is_error_not_terminal_or_cleanup_ack() {
    let (request, descriptor, _) = inference_fixture();
    let raw = request_raw(&descriptor);
    let error = serve_request(
        raw.as_slice(),
        raw.len() as u64,
        &descriptor,
        request.expected().request(),
        FlushFailure(Vec::new()),
        unsupported_execution,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::EngineFailed);
}

fn arguments() -> Vec<std::ffi::OsString> {
    let root = std::env::temp_dir().join("worker-entry-shape-only");
    vec![
        "--vad-request-v1".into(),
        root.join("stdin.raw").into_os_string(),
        "--cancel-file".into(),
        root.join("cancel.signal").into_os_string(),
        "--release-manifest-id".into(),
        "manifest".into(),
        "--release-manifest-sha256".into(),
        "a".repeat(64).into(),
        "--release-manifest-bytes".into(),
        "16".into(),
        "--worker-control-v1".into(),
        root.join("worker-control.v1.json").into_os_string(),
        "--worker-control-sha256".into(),
        "b".repeat(64).into(),
        "--worker-control-bytes".into(),
        "128".into(),
    ]
}
#[test]
fn existing_parent_argument_shape_is_not_an_admission_or_control() {
    let launch = WorkerLaunch::parse(arguments()).unwrap();
    assert_eq!(launch.request.parent(), launch.cancel_signal.parent());
    assert_eq!(launch.manifest.bytes(), 16);
}
#[test]
fn extra_flags_or_other_cancel_directory_are_rejected() {
    let mut extra = arguments();
    extra.push("--reset-budget".into());
    assert_eq!(
        WorkerLaunch::parse(extra).err().unwrap().code,
        ErrorCode::InvalidArgument
    );
    let mut other = arguments();
    other[3] = std::env::temp_dir()
        .join("other-worker-root/cancel.signal")
        .into_os_string();
    assert_eq!(
        WorkerLaunch::parse(other).err().unwrap().code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn guarded_original_files_and_inherited_reader_share_production_io() {
    use crate::staged_files::WorkerRequestFiles;
    use std::fs;
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "worker-entry-files-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&root).unwrap();
    let (request, descriptor, _) = inference_fixture();
    let raw = request_raw(&descriptor);
    fs::write(root.join("stdin.raw"), &raw).unwrap();
    fs::write(root.join("descriptor.json"), &descriptor).unwrap();
    let files = WorkerRequestFiles::open(&root.join("stdin.raw")).unwrap();
    assert_eq!(files.original_request(), request.expected().request());
    crate::infrastructure::os_policy::guarded_mutation(fs::write(
        root.join("stdin.raw"),
        b"replace",
    ));
    let mut output = Vec::new();
    serve_request(
        raw.as_slice(),
        files.declared_bytes(),
        files.descriptor_raw(),
        files.original_request(),
        &mut output,
        unsupported_execution,
    )
    .unwrap();
    assert!(!output.is_empty());
    drop(files);
    fs::remove_dir_all(root).unwrap();
}
