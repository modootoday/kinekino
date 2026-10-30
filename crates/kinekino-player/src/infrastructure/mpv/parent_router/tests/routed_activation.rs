//! Inert Rust fixtures only; no checked window, media process or OS readiness proof.
#[path = "routed_activation/transition_retirement.rs"]
mod transition_retirement;
use super::*;
use crate::{
    application::skip_segments::host::{PreparedRoutedSkipOwner, SkipHostApi, SkipHostRegistry},
    assembly::native_playback::routed::{self, ReadyRoutedNative, RoutedSetupError},
    infrastructure::{
        catalog::CatalogFiles, mpv::TrustedMpvExecutable, subtitle_files::SubtitleFiles,
    },
};

struct Fixture {
    engine: Arc<NativeEpisodeEngine>,
    files: Arc<CatalogFiles>,
}
impl Fixture {
    fn new() -> Self {
        use sha2::{Digest, Sha256};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "routed-activation-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        std::fs::create_dir(&directory).unwrap();
        eprintln!("routed_activation_fixture={}", directory.display());
        let executable = directory.join("mpv.exe");
        let bytes = b"inert routed activation fixture; never executed";
        std::fs::write(&executable, bytes).unwrap();
        let trusted =
            TrustedMpvExecutable::open(&executable, &format!("{:x}", Sha256::digest(bytes)))
                .unwrap();
        let engine = NativeEpisodeEngine::new_with_scope(
            trusted,
            Some(1),
            Arc::new(UnusedArtifacts),
            crate::infrastructure::mpv::guarded_command::HostScope::testing(),
        )
        .unwrap();
        std::fs::create_dir(directory.join("catalog")).unwrap();
        Self {
            engine: Arc::new(engine),
            files: Arc::new(CatalogFiles::open(&directory.join("catalog")).unwrap()),
        }
    }
}

struct Ready {
    router: Arc<ParentEngineRouter>,
    route: ParentRouteHandle,
    deadline: Instant,
    refuse: bool,
}
impl routed::ready_seal::Sealed for Ready {}
impl ReadyRoutedNative for Ready {
    type Error = &'static str;
    fn with_ready_route<T>(
        &self,
        visit: impl FnOnce(&Arc<ParentEngineRouter>, &ParentRouteHandle) -> T,
    ) -> Result<T, Self::Error> {
        if self.refuse {
            Err("synthetic checked transaction refusal")
        } else {
            Ok(visit(&self.router, &self.route))
        }
    }
}
fn ready(router: &Arc<ParentEngineRouter>, route: &ParentRouteHandle) -> Ready {
    Ready {
        router: router.clone(),
        route: route.clone(),
        deadline: Instant::now() + LIMIT,
        refuse: false,
    }
}

#[test]
fn atomic_capture_requires_original_active_route() {
    let mut dispatcher = host(8);
    let fixture = Fixture::new();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(fixture.engine.clone()).unwrap();
    assert_eq!(
        router
            .capture_active_native_export_issuer(&route)
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    router.activate(&route).unwrap();
    let scope = fixture.engine.guard_scope().upgrade().unwrap();
    seed_wire_scope(&scope, 71);
    assert!(router.capture_active_native_export_issuer(&route).is_ok());
    let foreign = ParentEngineRouter::new(dispatcher.handle());
    assert_eq!(
        foreign
            .capture_active_native_export_issuer(&route)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    router.revoke_route_admission(&route).unwrap();
    assert!(router.capture_active_native_export_issuer(&route).is_err());
    complete(&router, &route);
    finish(&mut dispatcher);
}

#[test]
fn failed_registry_returns_original_prepared_issuer_and_sessions() {
    let mut dispatcher = host(8);
    let fixture = Fixture::new();
    let router = Arc::new(ParentEngineRouter::new(dispatcher.handle()));
    let route = router.prepare(fixture.engine.clone()).unwrap();
    router.activate(&route).unwrap();
    seed_wire_scope(&fixture.engine.guard_scope().upgrade().unwrap(), 72);
    let issuer = router.capture_active_native_export_issuer(&route).unwrap();
    let sessions = Arc::new(EpisodePlaybackSessions::new(
        fixture.files.clone(),
        router.clone(),
    ));
    let prepared = PreparedRoutedSkipOwner::new(sessions.clone(), issuer.clone());
    let failure = match SkipHostRegistry::new_routed_export_host(String::new(), prepared, 1) {
        Err(failure) => failure,
        Ok(_) => panic!("empty generation must fail"),
    };
    assert_eq!(failure.prepared.issuer().scope(), issuer.scope());
    assert!(Arc::ptr_eq(&failure.prepared.sessions(), &sessions));
    let registry =
        SkipHostRegistry::new_routed_export_host("routed-valid".into(), failure.prepared, 1)
            .unwrap_or_else(|_| panic!("original prepared owner remains usable"));
    assert!(!registry.capability().seek_submission);
    assert_eq!(
        registry.export_capability().issuer_scope,
        Some(issuer.scope())
    );
    complete(&router, &route);
    finish(&mut dispatcher);
}

#[test]
fn ready_failure_preserves_owner_deadline_and_never_builds_base() {
    let mut dispatcher = host(8);
    let fixture = Fixture::new();
    let router = Arc::new(ParentEngineRouter::new(dispatcher.handle()));
    let route = router.prepare(fixture.engine.clone()).unwrap();
    let mut original = ready(&router, &route);
    original.refuse = true;
    let deadline = original.deadline;
    let invoked = std::cell::Cell::new(false);
    let failure = match routed::assemble_routed(original, fixture.files.clone(), || {
        invoked.set(true);
        PlayerSdk::new(SubtitleFiles)
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("transaction refusal must fail"),
    };
    assert!(matches!(
        failure.error,
        RoutedSetupError::Ready("synthetic checked transaction refusal")
    ));
    assert!(!invoked.get());
    assert_eq!(failure.ready.deadline, deadline);
    assert!(Arc::ptr_eq(&failure.ready.router, &router));
    assert!(failure.ready.route.same(&route));
    complete(&router, &route);
    finish(&mut dispatcher);
}

#[test]
fn review_owner_reads_original_terminal_row_after_route_replacement() {
    let mut dispatcher = host(8);
    let first = Fixture::new();
    let second = Fixture::new();
    let router = Arc::new(ParentEngineRouter::new(dispatcher.handle()));
    let route = router.prepare(first.engine.clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = first.engine.guard_scope().upgrade().unwrap();
    seed_wire_scope(&scope, 73);
    let original_issuer = router.capture_active_native_export_issuer(&route).unwrap();
    let assembly =
        match routed::assemble_routed(ready(&router, &route), first.files.clone(), || {
            PlayerSdk::new(SubtitleFiles)
        }) {
            Ok(value) => value,
            Err(_) => panic!("active synthetic route should assemble"),
        };
    let (guard, link, mut receiver) =
        crate::infrastructure::mpv::guarded_command::standalone_export_guard_for_test(&scope);
    let input = wire_input(guard);
    let origin = input.export.origin().clone();
    let output = router.submit_exported_guarded_seek_v1(input).unwrap();
    assert!(receiver.try_recv().is_ok());
    let reference = output.reference();
    let capabilities = assembly.client.execute(Request::new("capabilities"));
    assert!(capabilities.ok);
    let capabilities = capabilities.data.unwrap();
    assert_eq!(capabilities["skip_exports"]["available"], true);
    assert_eq!(
        capabilities["skip_sessions"]["seek_submission_available"],
        false
    );
    assert_eq!(assembly.read_owner.list(&origin).unwrap().items().len(), 1);
    complete(&router, &route);
    let next = router.prepare(second.engine.clone()).unwrap();
    router.activate(&next).unwrap();
    assert_eq!(
        router
            .observe_native_guard_for_export(&original_issuer)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict,
    );
    let routed::RoutedNativeAssembly {
        ready,
        client,
        read_owner,
    } = assembly;
    drop(ready);
    drop(client);
    drop(output);
    drop(link);
    drop(receiver);
    drop(scope);
    drop(route);
    drop(first);
    assert!(read_owner.peek(reference, &origin).is_ok());
    assert!(read_owner.consume(reference, &origin).is_ok());
    assert!(read_owner.list(&origin).unwrap().items().is_empty());
    assert!(read_owner.consume(reference, &origin).is_err());
    drop(read_owner);
    assert_eq!(
        original_issuer.peek(reference, &origin).unwrap_err().code,
        ErrorCode::SourceExpired,
    );
    complete(&router, &next);
    finish(&mut dispatcher);
}
