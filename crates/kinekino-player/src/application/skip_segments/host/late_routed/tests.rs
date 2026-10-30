//! Synthetic issuer/session fixtures, never a native guard or production Ready implementation.
use super::*;

#[test]
fn exact_registry_rearm_keeps_same_sdk_and_rejects_consumed_or_old_generation() {
    let original = sessions();
    let (port, installer) = LateRoutedPort::prepare(original.clone());
    let first = installer
        .install("rearm-first".into(), prepared(original.clone()))
        .unwrap_or_else(|_| panic!("Original registry installation failed"));
    let old_scope = port.export_capability().issuer_scope;
    first.rearm.suspend().unwrap();
    first.rearm.suspend().unwrap();
    assert!(!port.export_capability().available);
    assert_eq!(
        port.execute(&Request::new("segments.inspect"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    let replacement = first.rearm.resume().unwrap();
    assert!(first.rearm.resume().is_err());
    let second = replacement
        .install("rearm-second".into(), prepared(original))
        .unwrap_or_else(|_| panic!("Same original sessions reactivation failed"));
    assert!(port.export_capability().available);
    assert!(!Arc::ptr_eq(&first.registry, &second.registry));
    // This inherited synthetic issuer deliberately has one fixed scope; real scope renewal
    // is issued by the original router during fresh native activation, not by this model.
    assert_eq!(port.export_capability().issuer_scope, old_scope);
    assert!(first.rearm.suspend().is_err());
    drop(first.owner);
    assert!(port.export_capability().available);
    assert_eq!(
        port.export_capability().issuer_scope,
        second.registry.export_capability().issuer_scope
    );
}

#[test]
fn closing_reactivated_pending_registry_never_revives_old_native_owner() {
    let original = sessions();
    let (port, installer) = LateRoutedPort::prepare(original.clone());
    let installed = installer
        .install("rearm-close".into(), prepared(original.clone()))
        .unwrap_or_else(|_| panic!("Original registry installation failed"));
    installed.rearm.suspend().unwrap();
    let pending = installed.rearm.resume().unwrap();
    pending.close_pending().unwrap();
    let failure = match pending.install("rearm-closed".into(), prepared(original.clone())) {
        Ok(_) => panic!("Closed original reactivation accepted"),
        Err(failure) => failure,
    };
    assert_eq!(failure.error.code, ErrorCode::Cancelled);
    assert!(Arc::ptr_eq(&failure.prepared.sessions(), &original));
    assert!(installed.rearm.resume().is_err());
    assert!(!port.export_capability().available);
}
use crate::application::{
    catalog::CaptureMetadata,
    episode_resolve::{EpisodeResolutionResults, StoredEpisodeResolution},
    jobs::VerifiedJobOutput,
    playback_control::ControlReceipt,
    playback_session::{v1, *},
    skip_segments::host::PreparedRoutedSkipOwner,
};
struct Results;
impl EpisodeResolutionResults for Results {
    fn publish(
        &self,
        _: &StoredEpisodeResolution,
        _: &CaptureMetadata,
    ) -> Result<VerifiedJobOutput, ApiError> {
        panic!("No resolution publication")
    }
    fn read(&self, _: &str) -> Result<StoredEpisodeResolution, ApiError> {
        panic!("No media open")
    }
}
struct Engine;
impl EpisodePlaybackEngine for Engine {
    fn open(
        &self,
        _: EpisodePlaybackSource,
        _: bool,
        _: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        panic!("No native open")
    }
    fn close(&self, _: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        panic!("No native close")
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        panic!("No native observation")
    }
    fn control(&self, _: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        panic!("No native control")
    }
}
#[derive(Debug)]
struct Issuer;
impl NativeExportIssuerSourceV1 for Issuer {
    fn scope(&self) -> v1::NativeIssuerScopeV1 {
        v1::NativeIssuerScopeV1::from_original_nonce([71; 16]).unwrap()
    }
    fn list(&self, _: &v1::NativeExportOriginV1) -> Result<v1::NativeExportListV1, ApiError> {
        v1::NativeExportListV1::new(self.scope(), Vec::new())
    }
    fn read(
        &self,
        _: v1::NativeResultRefV1,
        _: &v1::NativeExportOriginV1,
        _: bool,
    ) -> Result<v1::NativeExportResultV1, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Synthetic issuer has no result rows",
        ))
    }
}
fn sessions() -> Arc<EpisodePlaybackSessions> {
    Arc::new(EpisodePlaybackSessions::new(
        Arc::new(Results),
        Arc::new(Engine),
    ))
}
fn prepared(sessions: Arc<EpisodePlaybackSessions>) -> PreparedRoutedSkipOwner {
    PreparedRoutedSkipOwner::new(
        sessions,
        v1::NativeExportIssuer::from_original_source(Arc::new(Issuer)),
    )
}

#[test]
fn foreign_and_replayed_install_preserve_original_port_then_delegate_existing_policy() {
    let original = sessions();
    let foreign = sessions();
    let (port, installer) = LateRoutedPort::prepare(original.clone());
    let replay = LateRoutedInstaller { port: port.clone() }; // test-only replay; production installer is not Clone.
    assert_eq!(
        port.execute(&Request::new("segments.inspect"))
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    let failure = match installer.install("late-original".into(), prepared(foreign.clone())) {
        Err(failure) => failure,
        Ok(_) => panic!("Foreign sessions accepted"),
    };
    assert_eq!(failure.error.code, ErrorCode::RevisionConflict);
    assert!(Arc::ptr_eq(&failure.prepared.sessions(), &foreign));
    assert!(!port.export_capability().available);
    let installed = failure
        .installer
        .install("late-original".into(), prepared(original.clone()))
        .unwrap_or_else(|_| panic!("Original install failed"));
    let original_scope = port.export_capability().issuer_scope;
    assert!(port.export_capability().available);
    assert!(!port.capability().seek_submission);
    let replay_failure = match replay.install("late-replay".into(), prepared(original)) {
        Err(failure) => failure,
        Ok(_) => panic!("Repeated installation accepted"),
    };
    assert_eq!(replay_failure.error.code, ErrorCode::RevisionConflict);
    assert_eq!(port.export_capability().issuer_scope, original_scope);
    let request = Request::new("segments.inspect");
    let expected = installed.registry.execute(&request).unwrap_err();
    let actual = port.execute(&request).unwrap_err();
    assert_eq!(
        (actual.code, actual.message),
        (expected.code, expected.message)
    );
    drop(installed.owner);
    assert!(!port.export_capability().available);
}

#[test]
fn cancelled_pending_install_stays_closed_and_returns_original_prepared_owner() {
    let original = sessions();
    let (port, installer) = LateRoutedPort::prepare(original.clone());
    installer.close_pending().unwrap();
    let failure = match installer.install("late-cancelled".into(), prepared(original.clone())) {
        Err(failure) => failure,
        Ok(_) => panic!("Closed install accepted"),
    };
    assert_eq!(failure.error.code, ErrorCode::Cancelled);
    assert!(Arc::ptr_eq(&failure.prepared.sessions(), &original));
    assert_eq!(
        port.execute(&Request::new("segments.inspect"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    failure.installer.close_pending().unwrap();
    assert!(!port.capability().sessions);
}
