//! Actual22 cases reuse the existing original owner and common response decoder.
use super::configured_payload::{ConfiguredPayload, Fault};
use super::*;
use kinekino_vad::protocol::domain::WorkerResult;
use std::cell::RefCell;

struct CancelAfterReady {
    end: Instant,
    root: PathBuf,
    ready: RefCell<Option<File>>,
}
impl RunControl for CancelAfterReady {
    fn deadline(&self) -> Instant {
        self.end
    }
    fn cancellation_requested(&self) -> bool {
        if self.ready.borrow().is_some() {
            return true;
        }
        match crate::fixture_data::ordinary_paths::file(
            &self.root.join(fixture_barrier::ENGINE_READY),
        ) {
            Ok(mut guard) => {
                let mut bytes = Vec::new();
                Read::by_ref(&mut guard)
                    .take(7)
                    .read_to_end(&mut bytes)
                    .unwrap();
                assert_eq!(bytes, b"ready\n");
                *self.ready.borrow_mut() = Some(guard);
                true
            }
            Err(error) if error.code == ErrorCode::NotFound => false,
            Err(error) => panic!("Original configured ready IO failed: {error:?}"),
        }
    }
}
pub(super) fn run(case: &mut Case) {
    let fault = match case.kind {
        CaseKind::ConfiguredRequestMismatch => Fault::Request,
        CaseKind::ConfiguredBytesMismatch => Fault::Configuration,
        CaseKind::ConfiguredManifestMismatch => Fault::Manifest,
        CaseKind::ConfiguredWorkerMismatch => Fault::Worker,
        _ => Fault::None,
    };
    let payload = Arc::new(ConfiguredPayload::new(case.data.clone(), fault).unwrap());
    if matches!(
        case.kind,
        CaseKind::ConfiguredComplete | CaseKind::ConfiguredCancel
    ) {
        return native(case, payload);
    }
    case.spawn_prepared(
        "complete",
        false,
        false,
        false,
        payload.as_ref(),
        &payload.arguments(),
    );
    case.wait_exit();
    assert_eq!(case.owner.as_ref().unwrap().receipt().child_exit, Some(0));
    let decoded = case.decoded();
    let expected = if matches!(case.kind, CaseKind::ConfiguredWorkerMismatch) {
        ErrorCode::RevisionConflict
    } else {
        ErrorCode::SourceExpired
    };
    assert!(matches!(
        decoded.result(),
        WorkerResult::Failure { code, .. } if *code == expected
    ));
    assert!(!case.root.join(fixture_barrier::ENGINE_READY).exists());
}
fn native(case: &mut Case, payload: Arc<ConfiguredPayload>) {
    let cancelling = matches!(case.kind, CaseKind::ConfiguredCancel);
    scenarios::prepare_native(
        case,
        if cancelling {
            "wait-cancel"
        } else {
            "complete"
        },
        payload,
    );
    let root = case.root.join(format!("vad-{}-0", std::process::id()));
    let cancel = CancelAfterReady {
        end: case.operation_end,
        root,
        ready: RefCell::new(None),
    };
    let fixed = Original {
        end: case.operation_end,
    };
    let control: &dyn RunControl = if cancelling { &cancel } else { &fixed };
    let result = case
        .native
        .as_mut()
        .unwrap()
        .launch_read(&case.data.request, control);
    let ready = cancel.ready.into_inner();
    let ready_observed = ready.is_some();
    if let Some(guard) = ready {
        case.guards.push(guard);
    }
    let mut read = result.unwrap();
    let mut bytes = Vec::new();
    Read::by_ref(&mut read.stdout)
        .take(kinekino_vad::protocol::application::MAX_OUTCOME_BYTES + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    fs::write(case.root.join("configured-stdout.bin"), &bytes).unwrap();
    let decoded = kinekino_vad::protocol::application::decode_worker_outcome(
        bytes.as_slice(),
        read.declared_bytes,
        &case.data.expected,
    )
    .unwrap();
    if cancelling {
        assert!(ready_observed);
        assert!(case.native.as_ref().unwrap().cancellation_signal_sent());
        assert!(
            matches!(decoded.result(), WorkerResult::Partial { processed_samples: 512, scores } if scores == &[0.2])
        );
    } else {
        assert!(!case.native.as_ref().unwrap().cancellation_signal_sent());
        assert!(
            matches!(decoded.result(), WorkerResult::Complete { scores } if scores == &[0.2,0.7])
        );
    }
}
macro_rules! actual {
    ($name:ident, $kind:ident) => {
        #[test]
        #[ignore = "Separate allocated configured22 gate and freshly pinned dedicated child BIN required"]
        fn $name() { exercise(CaseKind::$kind); }
    };
}
actual!(
    configured22_native_complete_preserves_original_owner,
    ConfiguredComplete
);
actual!(
    configured22_native_signal_cancel_returns_genuine_partial,
    ConfiguredCancel
);
actual!(
    configured22_wrong_request_binding_is_rejected_before_engine,
    ConfiguredRequestMismatch
);
actual!(
    configured22_changed_configuration_bytes_are_rejected_before_engine,
    ConfiguredBytesMismatch
);
actual!(
    configured22_changed_full_manifest_bytes_are_rejected_before_engine,
    ConfiguredManifestMismatch
);
actual!(
    configured22_other_worker_identity_is_rejected_before_engine,
    ConfiguredWorkerMismatch
);
