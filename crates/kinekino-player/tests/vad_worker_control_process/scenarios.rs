use super::*;
use kinekino_vad::protocol::domain::WorkerResult;

struct Executable(Arc<FixtureData>);
impl ExecutableGuard for Executable {
    fn path(&self) -> &Path {
        &self.0.executable
    }
    fn open_guarded(&self) -> Result<File, ApiError> {
        self.0.open_executable()
    }
}
struct Root {
    path: PathBuf,
    _guards: Vec<File>,
}
impl EvidenceRootGuard for Root {
    fn path(&self) -> &Path {
        &self.path
    }
}

pub(super) fn run(case: &mut Case) {
    if matches!(
        case.kind,
        CaseKind::ConfiguredComplete
            | CaseKind::ConfiguredCancel
            | CaseKind::ConfiguredRequestMismatch
            | CaseKind::ConfiguredBytesMismatch
            | CaseKind::ConfiguredManifestMismatch
            | CaseKind::ConfiguredWorkerMismatch
    ) {
        return super::configured::run(case);
    }
    if matches!(case.kind, CaseKind::Complete) {
        return native_complete(case);
    }
    let startup = matches!(
        case.kind,
        CaseKind::BeforeEngineCancel | CaseKind::DelayedStart
    );
    let wait = matches!(
        case.kind,
        CaseKind::RunningCancel | CaseKind::RunningExpiry | CaseKind::SignalFault
    );
    case.spawn(
        if wait { "wait-cancel" } else { "complete" },
        startup,
        matches!(case.kind, CaseKind::FlushFailure),
        matches!(case.kind, CaseKind::BindingMismatch),
    );
    match case.kind {
        CaseKind::RunningCancel => {
            case.wait_ready(fixture_barrier::ENGINE_READY);
            case.guards
                .push(worker_control::publish_cancellation(&case.root).unwrap());
        }
        CaseKind::BeforeEngineCancel => {
            case.wait_ready(fixture_barrier::STARTUP_READY);
            case.guards
                .push(worker_control::publish_cancellation(&case.root).unwrap());
            case.release(fixture_barrier::STARTUP_RELEASE);
        }
        CaseKind::DelayedStart => {
            case.wait_ready(fixture_barrier::STARTUP_READY);
            while Instant::now() < case.operation_end {
                std::thread::sleep(Duration::from_millis(1));
            }
            // Do not renew the original operation. Only its original cleanup
            // interval remains while the first child checkpoint rejects expiry.
            assert!(Instant::now() < case.end);
            case.release(fixture_barrier::STARTUP_RELEASE);
        }
        CaseKind::RunningExpiry => case.wait_ready(fixture_barrier::ENGINE_READY),
        CaseKind::SignalFault => {
            case.wait_ready(fixture_barrier::ENGINE_READY);
            // Exact-role local IO fault, no ACL/global OS settings or parent
            // descriptor mutation. The child must not turn this into Cancelled.
            fs::create_dir(case.root.join("cancel.signal")).unwrap();
        }
        _ => {}
    }
    case.wait_exit();
    if matches!(case.kind, CaseKind::FlushFailure) {
        assert!(fs::read(case.root.join("stdout.bin")).unwrap().is_empty());
        assert!(!fs::read(case.root.join("stderr.bin")).unwrap().is_empty());
        assert_ne!(case.owner.as_ref().unwrap().receipt().child_exit, Some(0));
        return;
    }
    assert_eq!(case.owner.as_ref().unwrap().receipt().child_exit, Some(0));
    let decoded = case.decoded();
    match (case.kind, decoded.result()) {
        (
            CaseKind::RunningCancel,
            WorkerResult::Partial {
                processed_samples,
                scores,
            },
        ) => {
            assert_eq!(*processed_samples, 512);
            assert_eq!(scores, &[0.2]);
        }
        (CaseKind::BeforeEngineCancel, WorkerResult::Failure { code, .. }) => {
            assert_eq!(*code, ErrorCode::Cancelled)
        }
        (CaseKind::DelayedStart | CaseKind::RunningExpiry, WorkerResult::Failure { code, .. }) => {
            assert_eq!(*code, ErrorCode::Timeout)
        }
        (CaseKind::SignalFault, WorkerResult::Failure { code, .. }) => {
            assert_eq!(*code, ErrorCode::PermissionDenied)
        }
        (CaseKind::BindingMismatch, WorkerResult::Failure { code, .. }) => {
            assert_eq!(*code, ErrorCode::SourceExpired)
        }
        _ => panic!("Original worker outcome differs: {:?}", decoded.result()),
    }
    if matches!(
        case.kind,
        CaseKind::BeforeEngineCancel | CaseKind::DelayedStart | CaseKind::BindingMismatch
    ) {
        assert!(!case.root.join(fixture_barrier::ENGINE_READY).exists());
    }
}

pub(super) fn prepare_native(
    case: &mut Case,
    mode: &str,
    payload: Arc<dyn WorkerPayloadPreparation>,
) {
    let (_, guards) = crate::fixture_data::ordinary_paths::root(&case.root).unwrap();
    let root = Arc::new(Root {
        path: case.root.clone(),
        _guards: guards,
    });
    let factory = NativeTransportFactory::new(
        Arc::new(Executable(case.data.clone())),
        case.data.profile.runtime().clone(),
        case.root.clone(),
        root,
        case.deadline,
        case.environment(mode),
    )
    .with_payload(case.data.manifest_pin.clone(), payload)
    .unwrap();
    case.native = Some(factory.prepare().unwrap());
}

fn native_complete(case: &mut Case) {
    prepare_native(
        case,
        "complete",
        Arc::new(FixturePayload::new(case.data.clone())),
    );
    let mut read = case
        .native
        .as_mut()
        .unwrap()
        .launch_read(
            &case.data.request,
            &Original {
                end: case.operation_end,
            },
        )
        .unwrap();
    let mut raw = Vec::new();
    read.stdout.read_to_end(&mut raw).unwrap();
    fs::write(case.root.join("native-stdout.bin"), &raw).unwrap();
    let decoded = kinekino_vad::protocol::application::decode_worker_outcome(
        raw.as_slice(),
        read.declared_bytes,
        &case.data.expected,
    )
    .unwrap();
    assert!(matches!(decoded.result(), WorkerResult::Complete { scores } if scores == &[0.2,0.7]));
    assert!(!case.native.as_ref().unwrap().cancellation_signal_sent());
}
