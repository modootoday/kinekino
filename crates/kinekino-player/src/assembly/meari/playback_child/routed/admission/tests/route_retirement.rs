use super::*;
use crate::infrastructure::mpv::{NativeEpisodeEngine, RetirementProgress, TrustedMpvExecutable};
use sha2::{Digest, Sha256};

struct Artifacts;
impl crate::application::catalog::CatalogArtifacts for Artifacts {
    fn save(
        &self,
        _: &[u8],
        _: &crate::application::catalog::CaptureMetadata,
    ) -> Result<crate::application::catalog::RawArtifact, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "inert retirement fixture has no media",
        ))
    }
}
fn settle(router: &ParentEngineRouter, route: &ParentRouteHandle) {
    let started = router.retire(route, Arc::new(|| Ok(()))).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if matches!(
            router.observe_retirement(&started.handle).unwrap().progress,
            RetirementProgress::Complete(_)
        ) {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}
#[test]
fn original_checkpoint_retirement_keeps_shared_slot_reusable_but_new_parent_loss_closes_forever() {
    let directory = std::env::temp_dir().join(format!(
        "route-admission-retirement-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("mpv.exe");
    let bytes = b"Inert original native engine fixture; never executed";
    std::fs::write(&path, bytes).unwrap();
    let engine = || {
        Arc::new(
            NativeEpisodeEngine::new(
                TrustedMpvExecutable::open(&path, &format!("{:x}", Sha256::digest(bytes))).unwrap(),
                Some(1),
                Arc::new(Artifacts),
            )
            .unwrap(),
        )
    };
    let mut dispatcher = DispatcherHost::start(NonZeroUsize::new(2).unwrap()).unwrap();
    let slot = Arc::new(TransitionSlot::default());
    let router = Arc::new(
        ParentEngineRouter::new_with_transitions(dispatcher.handle(), slot.clone())
            .with_transition_jobs(Arc::new(crate::application::jobs::UnavailableJobs)),
    );
    let old = router.prepare(engine()).unwrap();
    router.activate(&old).unwrap();
    let admission = Rc::new(OriginalAdmission::new(router.clone()));
    admission.install(old.clone()).unwrap();
    let checkpoint = super::super::super::ready::RoutedMediaCheckpoint {
        state: std::rc::Weak::new(),
        admission: admission.clone(),
    };
    let (original_router, pause) = checkpoint.retire_original_route().unwrap();
    assert!(Arc::ptr_eq(&original_router, &router));
    admission.revoke().unwrap();
    assert!(!slot.admission_closed());
    assert!(admission.require_live().is_err());
    settle(&router, &old);
    assert!(router.poll_route_transitions(&pause).unwrap());
    let fresh = router.prepare(engine()).unwrap();
    router.activate(&fresh).unwrap();
    let fresh_admission = OriginalAdmission::new(router.clone());
    fresh_admission.install(fresh.clone()).unwrap();
    router.resume_route_transitions(&pause, &fresh).unwrap();
    admission.revoke().unwrap();
    assert!(
        !slot.admission_closed(),
        "late old-route release must not close the fresh parent slot"
    );
    assert_eq!(
        slot.begin(|| Err(ApiError::new(
            ErrorCode::NeedsInput,
            "fresh preparation reached"
        )))
        .unwrap_err()
        .message,
        "fresh preparation reached"
    );
    fresh_admission.revoke().unwrap();
    assert!(slot.admission_closed());
    assert_eq!(
        slot.begin(|| panic!("new parent loss must close permanently"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    settle(&router, &fresh);
    drop(fresh_admission);
    drop(checkpoint);
    drop(admission);
    drop(original_router);
    drop(old);
    drop(fresh);
    drop(router);
    dispatcher.request_shutdown();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match dispatcher.poll_shutdown() {
            DispatcherShutdown::Joined {
                worker_panicked: false,
                faults: 0,
            } => break,
            DispatcherShutdown::Pending => {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            other => panic!("original dispatcher cleanup failed: {other:?}"),
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}
