use super::*;
use crate::{application::source_tests::admitted_fixture, infrastructure::PinnedRequestEncoder};
use kinekino_media_contracts::api_values::DecimalU64;
use std::sync::atomic::{AtomicUsize, Ordering};
struct Transport {
    calls: Arc<AtomicUsize>,
}
impl WorkerTransport for Transport {
    fn launch_read(&mut self, _: &[u8], _: &dyn RunControl) -> Result<WorkerRead, ApiError> {
        panic!("No launch in lease fixture")
    }
    fn poll_close(&mut self, _: Instant) -> Result<OwnerObservation, ApiError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            Err(ApiError::new(
                ErrorCode::EngineFailed,
                "original poll error",
            ))
        } else {
            Ok(OwnerObservation::Pending)
        }
    }
}
struct AttemptPublication(Arc<AtomicUsize>);
impl Drop for AttemptPublication {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
impl OutputPublication for AttemptPublication {
    fn publish(&self, _: &WorkerOutcomeCandidate, _: Instant) -> Result<(), ApiError> {
        panic!("Unused attempt destination must not publish")
    }
    fn verify(
        &self,
        _: &WorkerOutcomeCandidate,
        _: Instant,
    ) -> Result<VerifiedJobOutput, ApiError> {
        panic!("Unused attempt destination must not verify")
    }
}
fn input(admission: &PreparedVadAdmission) -> PreparedJobInput {
    PreparedJobInput {
        operation: "audio.vad.candidate".into(),
        parameters: serde_json::json!({}),
        source: admission.descriptor().source().clone(),
        execution_timeout_ms: DecimalU64(1000),
    }
}
fn resources(drops: Arc<AtomicUsize>, calls: Arc<AtomicUsize>) -> PreparedResources {
    PreparedResources {
        encoder: Arc::new(PinnedRequestEncoder),
        transport: Box::new(Transport { calls }),
        publication: Arc::new(AttemptPublication(drops)),
    }
}
#[test]
fn exact_prepared_source_is_required_before_admission_can_be_attached() {
    let (admission, _) = admitted_fixture();
    let mut source = input(&admission);
    validate_source(&source, &admission).unwrap();
    source.source.bytes.0 += 1;
    assert!(validate_source(&source, &admission).is_err());
    source = input(&admission);
    source.source.sha256 = "b".repeat(64);
    assert!(validate_source(&source, &admission).is_err());
}
#[test]
fn publication_keeps_original_source_and_attempt_leases_after_transport_release() {
    let (admission, lease) = admitted_fixture();
    let admission = Arc::new(admission);
    let drops = Arc::new(AtomicUsize::new(0));
    let attached = attach(
        resources(drops.clone(), Arc::new(AtomicUsize::new(0))),
        admission.clone(),
    );
    drop(admission);
    let PreparedResources {
        encoder,
        transport,
        publication,
    } = attached;
    drop(encoder);
    drop(transport);
    assert_eq!(Arc::strong_count(&lease), 4);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(publication);
    assert_eq!(Arc::strong_count(&lease), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
#[test]
fn wrapper_delegates_original_poll_error_and_later_pending_without_fabricating_ack() {
    let (admission, _) = admitted_fixture();
    let calls = Arc::new(AtomicUsize::new(0));
    let mut attached = attach(
        resources(Arc::new(AtomicUsize::new(0)), calls.clone()),
        Arc::new(admission),
    );
    let error = attached.transport.poll_close(Instant::now()).unwrap_err();
    assert_eq!(error.message, "original poll error");
    assert_eq!(
        attached.transport.poll_close(Instant::now()).unwrap(),
        OwnerObservation::Pending
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
