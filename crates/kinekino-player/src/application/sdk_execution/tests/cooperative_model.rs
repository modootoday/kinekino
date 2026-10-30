use super::*;
use crate::application::{
    api::PlayerSdk,
    model_catalog::{ModelCatalogPort, ModelFileObservation, ModelVerification, VerifyModel},
    review_source::MutationControl,
};
use crate::domain::model_catalog::{ModelCatalog, ModelManifest};

struct Gate {
    entered: mpsc::Sender<()>,
    release: (Mutex<bool>, Condvar),
    snapshots: AtomicUsize,
    reads: AtomicUsize,
}
#[derive(Clone)]
struct Port(Arc<Gate>);
impl ModelCatalogPort for Port {
    fn snapshot(&self) -> Result<(ModelCatalog, String), ApiError> {
        self.0.snapshots.fetch_add(1, Ordering::SeqCst);
        let catalog = serde_json::from_value(serde_json::json!({"schema_version":"model-catalog/1","models":[{
            "schema_version":"model-manifest/1","model_id":"technical-only","task":"text","repo_id":"fixture/model","revision":"a".repeat(40),
            "files":[{"path":"weights.onnx","bytes":"4","sha256":"a".repeat(64)},{"path":"config.json","bytes":"8","sha256":"b".repeat(64)},{"path":"tokenizer.json","bytes":"5","sha256":"c".repeat(64)}],
            "format":"onnx","quantization":"fp32","source_model":null,"conversion":null,"tokenizer_files":["tokenizer.json"],"config_files":["config.json"],"license":{"identifier":"fixture-only","reference":"https://example.invalid/LICENSE"},"supported_languages":["ko"],"runtime":"fixture-runtime","min_runtime_version":"1.0","declared_measured_memory":null,"evaluation_id":null
        }]})).unwrap();
        Ok((catalog, "b".repeat(64)))
    }
    fn inspect_files(&self, _: &ModelManifest) -> Result<Vec<ModelFileObservation>, ApiError> {
        panic!("verify must not use size-only inspection")
    }
    fn verification_available(&self) -> bool {
        true
    }
    fn verify_files(
        &self,
        _: &VerifyModel,
        control: &dyn MutationControl,
    ) -> Result<ModelVerification, ApiError> {
        self.0.reads.fetch_add(1, Ordering::SeqCst);
        control.checkpoint()?;
        self.0.entered.send(()).unwrap();
        let (released, timeout) = self
            .0
            .release
            .1
            .wait_timeout_while(
                self.0.release.0.lock().unwrap(),
                Duration::from_secs(3),
                |released| !*released,
            )
            .unwrap();
        if timeout.timed_out() && !*released {
            return Err(internal("Model fixture release expired"));
        }
        drop(released);
        control.checkpoint()?;
        Err(internal(
            "Fixture expected cancellation, no model admission",
        ))
    }
}
struct Release(Arc<Gate>);
impl Drop for Release {
    fn drop(&mut self) {
        *self.0.release.0.lock().unwrap() = true;
        self.0.release.1.notify_all();
    }
}
fn request() -> Request {
    let mut request = Request::new("models.verify");
    request.payload = serde_json::json!({"model_id":"technical-only","revision":"a".repeat(40),"catalog_sha256":"b".repeat(64),"maximum_bytes":"17"});
    request
}
fn gate() -> (Arc<Gate>, mpsc::Receiver<()>) {
    let (entered, receiver) = mpsc::channel();
    (
        Arc::new(Gate {
            entered,
            release: (Mutex::new(false), Condvar::new()),
            snapshots: AtomicUsize::new(0),
            reads: AtomicUsize::new(0),
        }),
        receiver,
    )
}

#[test]
fn running_model_verification_observes_original_operation_cancel_and_keeps_next_request_live() {
    let (gate, entered) = gate();
    let release = Release(gate.clone());
    let client = Arc::new(PlayerSdk::default().with_model_catalog(Port(gate.clone())));
    let owner = ExecutionOwner::start(
        client,
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: false,
        }),
    )
    .unwrap();
    let handle = owner.handle();
    let id = handle.submit(request()).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!(gate.reads.load(Ordering::SeqCst), 1);
    assert_eq!(handle.cancel(&id).unwrap(), CancelDisposition::Requested);
    let next = handle.submit(Request::new("version")).unwrap();
    drop(release);
    let cancelled = terminal(&handle, &id);
    assert!(cancelled.cancellation_requested);
    assert!(!cancelled.queued_cancelled);
    assert_eq!(
        cancelled.response.error.as_ref().unwrap().code,
        ErrorCode::Cancelled
    );
    assert_eq!(gate.snapshots.load(Ordering::SeqCst), 1);
    assert!(terminal(&handle, &next).response.ok);
    handle.acknowledge(&id).unwrap();
    handle.acknowledge(&next).unwrap();
    close(&handle);
}

#[test]
fn model_control_cancelled_before_work_never_opens_catalog_or_files() {
    struct Cancelled;
    impl MutationControl for Cancelled {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Err(ApiError::new(ErrorCode::Cancelled, "test cancellation"))
        }
    }
    let (gate, _) = gate();
    let client = PlayerSdk::default().with_model_catalog(Port(gate.clone()));
    let response = client.execute_controlled(request(), &Cancelled);
    assert_eq!(response.error.unwrap().code, ErrorCode::Cancelled);
    assert_eq!(gate.snapshots.load(Ordering::SeqCst), 0);
    assert_eq!(gate.reads.load(Ordering::SeqCst), 0);
}
