use super::*;
use crate::infrastructure::{
    browser_provider::test_host::{BrowserFixture, host_fixture},
    catalog::CatalogFiles,
    immutable_storage_paths as paths,
};
use kinekino_jobs::infrastructure::RetainingWorkerHost;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    profile: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn partial() -> (
    Fixture,
    super::super::ProviderStartFailure<RetainingWorkerHost>,
    BrowserFixture,
) {
    let root = std::env::temp_dir().join(format!(
        "provider-startup-carrier-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&root).unwrap();
    let profile = root.join("profile");
    let catalog = root.join("catalog");
    fs::create_dir(&profile).unwrap();
    fs::create_dir(&catalog).unwrap();
    let (_, guards) = paths::root(&profile).unwrap();
    let lock = paths::lock(&profile.join("provider-owner.lock")).unwrap();
    let files = Arc::new(CatalogFiles::open(&catalog).unwrap());
    let (browser, control) = host_fixture();
    let owner = ProviderJobsHost::<RetainingWorkerHost> {
        jobs: None,
        browser,
        files,
        _profile_guards: guards,
        _profile_lock: lock,
        jobs_never_started: true,
        download_admission_panic: None,
    };
    let mut error = ApiError::new(ErrorCode::PermissionDenied, "Original worker startup error");
    error.request_id = "startup-issuer-request".into();
    (
        Fixture { root, profile },
        super::super::ProviderStartFailure {
            error,
            owner: Some(Box::new(owner)),
        },
        control,
    )
}

#[test]
fn download_admission_panic_payload_stays_with_original_pending_host() {
    use std::sync::atomic::AtomicBool;
    struct OriginalPayload(Arc<AtomicBool>);
    impl Drop for OriginalPayload {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    let (fixture, mut failure, browser) = partial();
    let dropped = Arc::new(AtomicBool::new(false));
    struct PanickingFactory {
        dropped: Arc<AtomicBool>,
        calls: AtomicU64,
    }
    impl crate::infrastructure::download::DownloadHostFactory for PanickingFactory {
        fn admit(
            &self,
            _: Arc<dyn crate::application::browser_provider::BrowserProviderPort>,
            _: Arc<dyn crate::application::browser_provider::BrowserProviderOwner>,
        ) -> Result<Arc<dyn crate::application::download::DownloadPreparation>, ApiError> {
            self.calls.fetch_add(1, Ordering::AcqRel);
            std::panic::panic_any(OriginalPayload(self.dropped.clone()))
        }
    }
    let factory = PanickingFactory {
        dropped: dropped.clone(),
        calls: AtomicU64::new(0),
    };
    let admission = failure.owner.as_mut().unwrap().admit_download(&factory);
    match admission {
        Err(error) => assert_eq!(error.code, ErrorCode::Internal),
        Ok(_) => panic!("Panicking admission returned preparation"),
    }
    assert!(
        failure
            .owner
            .as_mut()
            .unwrap()
            .admit_download(&factory)
            .is_err()
    );
    assert_eq!(factory.calls.load(Ordering::Acquire), 1);
    let retained = failure
        .download_admission_panic()
        .unwrap()
        .downcast_ref::<OriginalPayload>()
        .unwrap();
    assert!(Arc::ptr_eq(&retained.0, &dropped));
    let deadline = Instant::now() + Duration::from_secs(30);
    let (first, owner) = failure.into_shutdown(deadline);
    assert_eq!(first.code, ErrorCode::PermissionDenied);
    assert_eq!(first.request_id, "startup-issuer-request");
    let owner = pending(owner.unwrap().poll(deadline));
    assert!(browser.shutdown_requested());
    assert!(!dropped.load(Ordering::Acquire));
    let retained = owner
        .host
        .download_admission_panic
        .as_deref()
        .unwrap()
        .downcast_ref::<OriginalPayload>()
        .unwrap();
    assert!(Arc::ptr_eq(&retained.0, &dropped));
    assert!(paths::lock(&fixture.profile.join("provider-owner.lock")).is_err());
    browser.report_exit(None);
    match owner.poll(deadline) {
        ProviderShutdownOutcome::Complete(result) => {
            assert!(result.receipt.is_some());
            assert!(result.errors.first().is_none());
        }
        ProviderShutdownOutcome::Pending(owner) => {
            Box::leak(owner);
            panic!("Original browser close remains unobserved");
        }
    }
    assert!(dropped.load(Ordering::Acquire));
}
fn pending<W: JobHostOwner>(outcome: ProviderShutdownOutcome<W>) -> Box<PendingProviderHost<W>> {
    match outcome {
        ProviderShutdownOutcome::Pending(owner) => owner,
        _ => panic!("Original provider still pending"),
    }
}

#[test]
fn partial_startup_keeps_original_profile_lock_until_original_browser_reports_exit() {
    let (fixture, failure, browser) = partial();
    let deadline = Instant::now() + Duration::from_secs(30);
    let (error, owner) = failure.into_shutdown(deadline);
    assert_eq!(error.request_id, "startup-issuer-request");
    let owner = pending(owner.unwrap().poll(deadline));
    assert!(browser.shutdown_requested());
    assert!(paths::lock(&fixture.profile.join("provider-owner.lock")).is_err());
    browser.report_exit(None);
    let ProviderShutdownOutcome::Complete(result) = owner.poll(deadline) else {
        panic!("Original synthetic browser exit required")
    };
    let receipt = result.receipt.unwrap();
    assert!(receipt.browser_worker_exited);
    assert_eq!(receipt.jobs.completed_total, 0);
    assert!(paths::lock(&fixture.profile.join("provider-owner.lock")).is_ok());
    // Browser facts are controlled test data; no COM/window/process was started.
}

#[test]
fn original_browser_failure_fields_survive_pending_cleanup_without_replacement() {
    let (_fixture, failure, browser) = partial();
    let mut original = ApiError::new(
        ErrorCode::PermissionDenied,
        "Original synthetic browser failure",
    );
    original.request_id = "original-browser-diagnostic".into();
    original.details = serde_json::json!({"original":true});
    browser.report_exit(Some(original));
    let deadline = Instant::now() + Duration::from_secs(30);
    let (_, owner) = failure.into_shutdown(deadline);
    let ProviderShutdownOutcome::Complete(result) = owner.unwrap().poll(deadline) else {
        panic!("Observed original exit")
    };
    let error = result.errors.first().unwrap();
    assert_eq!(error.code, ErrorCode::PermissionDenied);
    assert_eq!(error.request_id, "original-browser-diagnostic");
    assert_eq!(error.details["original"], true);
    assert!(result.receipt.unwrap().browser_worker_exited);
}

#[test]
fn missing_worker_without_parent_no_start_fact_never_becomes_empty_cleanup_receipt() {
    let (fixture, mut failure, browser) = partial();
    failure.owner.as_mut().unwrap().jobs_never_started = false;
    browser.report_exit(None);
    let deadline = Instant::now() + Duration::from_secs(30);
    let (_, owner) = failure.into_shutdown(deadline);
    let owner = pending(owner.unwrap().poll(deadline));
    assert!(owner.jobs_receipt.is_none());
    assert_eq!(owner.errors().first().unwrap().code, ErrorCode::Internal);
    assert!(paths::lock(&fixture.profile.join("provider-owner.lock")).is_err());
    drop(owner); // Explicit fixture disposal, never a product cleanup observation.
}
