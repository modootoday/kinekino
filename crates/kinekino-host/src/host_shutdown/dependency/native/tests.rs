//! Concrete deferred native host only: no playback.open or process launch.
use super::super::{DependencyObservation, finish_dependency, observe_dependency};
use super::*;
use kinekino_player::{
    application::{
        playback_session::EpisodePlaybackSessions, skip_segments::host::SkipHostRegistry,
    },
    infrastructure::{catalog::CatalogFiles, content_digest},
};
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::{
        Arc, Weak,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

struct NoMediaFixture {
    root: PathBuf,
    executable: PathBuf,
    original_registry: Weak<SkipHostRegistry>,
}
impl NoMediaFixture {
    fn create() -> (Self, NativePlaybackHost, SkipHostOwner) {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "native-no-media-carrier-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        eprintln!("native_no_media_fixture_root={}", root.display());
        let executable = root.join("mpv.exe");
        // This inert ordinary artifact is pinned, never executed or called a PE.
        fs::write(
            &executable,
            b"inert native constructor fixture; no media and no child",
        )
        .unwrap();
        let (_, pin) = content_digest::sha256(&mut File::open(&executable).unwrap(), 1024).unwrap();
        let catalog_root = root.join("catalog");
        fs::create_dir(&catalog_root).unwrap();
        let files = Arc::new(CatalogFiles::open(&catalog_root).unwrap());
        let host =
            NativePlaybackHost::start_trusted_standalone(&executable, &pin, files.clone()).unwrap();
        let sessions = Arc::new(EpisodePlaybackSessions::new(files, Arc::new(host.engine())));
        let registry = Arc::new(
            SkipHostRegistry::new_standalone_export_host(
                "native-no-media-fixture".into(),
                sessions,
                1,
            )
            .unwrap(),
        );
        let original_registry = Arc::downgrade(&registry);
        let owner = registry.owner();
        registry.begin_close();
        drop(registry);
        (
            Self {
                root,
                executable,
                original_registry,
            },
            host,
            owner,
        )
    }
    fn held(&self) {
        assert!(
            self.original_registry.upgrade().is_some(),
            "Original registry owner lost: {}",
            self.root.display()
        );
        let error = OpenOptions::new()
            .write(true)
            .open(&self.executable)
            .expect_err("Original native pin guard was released");
        assert_eq!(error.raw_os_error(), Some(32));
    }
    fn released(&self) {
        assert!(
            self.original_registry.upgrade().is_none(),
            "Original registry still retained: {}",
            self.root.display()
        );
        drop(
            OpenOptions::new()
                .write(true)
                .open(&self.executable)
                .unwrap(),
        );
    }
}

#[test]
fn native_no_media_completion_consumes_original_host_and_skip_owner() {
    let (fixture, host, skip_owner) = NoMediaFixture::create();
    let deadline = Instant::now() + Duration::from_secs(5);
    let owner = pending_native(host, skip_owner, deadline);
    fixture.held();
    let DependencyPoll::Complete(result) = owner.poll(deadline) else {
        panic!("Deferred no-media host should complete without a child");
    };
    assert!(
        result.receipt.is_null(),
        "No active engine means no child receipt"
    );
    assert!(result.errors.first().is_none());
    fixture.released();
}

#[test]
fn native_past_cutoff_keeps_original_pin_and_skip_owner_without_completion() {
    let (fixture, host, skip_owner) = NoMediaFixture::create();
    let past = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    let future = Instant::now() + Duration::from_secs(5);
    let original = pending_native(host, skip_owner, past);
    fixture.held();
    let DependencyPoll::Pending(original) = original.poll(future) else {
        panic!("Past cutoff must not return a no-media completion");
    };
    let first = serde_json::to_value(original.errors().first().unwrap()).unwrap();
    assert_eq!(original.errors().first().unwrap().code, ErrorCode::Timeout);
    fixture.held();
    let DependencyPoll::Pending(original) = original.poll(future + Duration::from_secs(5)) else {
        panic!("Later constraint must not refresh the original cutoff");
    };
    assert_eq!(
        serde_json::to_value(original.errors().first().unwrap()).unwrap(),
        first
    );
    fixture.held();
    // Explicit NO-MEDIA fixture disposal, not a receipt or active cleanup proof.
    // This disposal pattern is not valid for an active engine fixture.
    drop(original);
    fixture.released();
}

#[test]
fn native_no_media_completion_preserves_prior_transport_error() {
    let (fixture, host, skip_owner) = NoMediaFixture::create();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut first = ApiError::new(ErrorCode::PermissionDenied, "Original transport failure");
    first.request_id = "native-original-request".into();
    first.details = json!({"original":true,"phase":"ready_write"});
    let expected = serde_json::to_value(&first).unwrap();
    let original = pending_native(host, skip_owner, deadline);
    fixture.held();
    let DependencyObservation::Complete(result) =
        observe_dependency(original, Some(first), None, deadline)
    else {
        panic!("Deferred native close must consume the original carrier");
    };
    assert!(result.receipt.is_null());
    let error =
        finish_dependency(result).expect_err("Physical no-media close must not erase prior error");
    assert_eq!(serde_json::to_value(error).unwrap(), expected);
    fixture.released();
}
