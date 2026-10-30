use super::*;
use crate::application::api::ErrorCode;

pub(super) fn episode() -> (BrowserTicket, BrowserWork) {
    let request = ValidatedBrowserRequest::new(BrowserRequest {
        request_id: "activation".into(),
        operation: BrowserOperation::ResolveEpisode,
        url: "https://catalog.example.test/e/fixture".into(),
        timeout_ms: DecimalU64(1000),
        blocking_enabled: true,
    })
    .unwrap();
    let ticket = BrowserTicket::new("activation-session".into(), "activation".into(), 1).unwrap();
    let mut work = BrowserWork::new(ticket.clone(), request).unwrap();
    work.apply(
        &ticket,
        BrowserEvent::Started {
            engine_version: "fixture".into(),
            blocking_installed: true,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://catalog.example.test/e/fixture".into(),
            document_generation: 1,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::NavigationIdentity {
            document_generation: 1,
            native_navigation_id: 42,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::NavigationCompleted(MainNavigationDiagnostic {
            observed_unix_ms: DecimalU64(1),
            document_generation: DecimalU64(1),
            native_navigation_id: DecimalU64(42),
            http_status_code: Some(200),
            is_success: true,
            web_error_status: 0,
        }),
    );
    work.apply(
        &ticket,
        BrowserEvent::DocumentReady(BrowserDocument {
            kind: CaptureKind::DomSnapshot,
            final_url: "https://catalog.example.test/e/fixture".into(),
            document_generation: DecimalU64(1),
            observed_unix_ms: DecimalU64(1),
            body: Arc::from(b"fixture".as_slice()),
            response: None,
        }),
    );
    (ticket, work)
}
fn frame() -> PlayerFrameObservation {
    PlayerFrameObservation {
        kind: PlayerDocumentObservationKind::FrameNavigation,
        owner_document_generation_at_observation: DecimalU64(1),
        owner_navigation_id_at_observation: DecimalU64(42),
        frame_navigation_id: Some(DecimalU64(100)),
        frame_id: None,
        parent_frame_id: None,
        request_source_kind: None,
        lineage_verified: false,
        url: "https://frame.example.test/tv-show.php?v=fixture".into(),
        observed_unix_ms: DecimalU64(2),
        request_method: None,
        referer: None,
    }
}

#[test]
fn admission_is_once_and_invocation_and_candidate_are_not_media_success() {
    let (ticket, mut work) = episode();
    work.reserve_episode_activation(1).unwrap();
    assert_eq!(
        work.snapshot().activation.unwrap().status,
        EpisodeActivationStatus::Enqueued
    );
    assert_eq!(
        work.reserve_episode_activation(1).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    work.apply(
        &ticket,
        BrowserEvent::ActivationInvoked {
            document_generation: 1,
            native_navigation_id: 42,
        },
    );
    work.apply(&ticket, BrowserEvent::PlayerFrame(frame()));
    let snapshot = work.snapshot();
    assert_eq!(snapshot.state, BrowserWorkState::Running);
    assert!(snapshot.result.is_none());
    assert!(!snapshot.player_frames[0].lineage_verified);
    assert!(snapshot.player_frames[0].parent_frame_id.is_none());
}
#[test]
fn observed_wrapper_and_secondary_iframe_require_original_activation_and_remain_unverified_candidates()
 {
    let (ticket, mut work) = episode();
    let mut child = frame();
    child.url = "https://michealcdn.com/video/0123456789abcdef0123456789abcdef".into();
    assert_eq!(
        work.apply(&ticket, BrowserEvent::PlayerFrame(child.clone())),
        EventDisposition::Stale
    );
    work.reserve_episode_activation(1).unwrap();
    work.apply(
        &ticket,
        BrowserEvent::ActivationInvoked {
            document_generation: 1,
            native_navigation_id: 42,
        },
    );
    let mut wrapper = frame();
    wrapper.kind = PlayerDocumentObservationKind::PopupIntent;
    wrapper.frame_navigation_id = None;
    work.apply(&ticket, BrowserEvent::PlayerFrame(wrapper));
    work.apply(&ticket, BrowserEvent::PlayerFrame(child.clone()));
    let snapshot = work.snapshot();
    assert_eq!(snapshot.state, BrowserWorkState::Running);
    assert!(snapshot.result.is_none());
    assert!(snapshot.media_requests.is_empty());
    assert_eq!(snapshot.player_frames.len(), 2);
    assert!(
        snapshot.player_frames[0]
            .url
            .starts_with("https://frame.example.test/tv-show.php?")
    );
    assert_eq!(snapshot.player_frames[1].url, child.url);
    assert!(
        snapshot
            .player_frames
            .iter()
            .all(|frame| !frame.lineage_verified)
    );
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://catalog.example.test/e/fixture".into(),
            document_generation: 2,
        },
    );
    assert_eq!(
        work.apply(&ticket, BrowserEvent::PlayerFrame(child)),
        EventDisposition::Stale
    );
}
#[test]
fn same_url_reload_rejects_old_activation_status_and_frames() {
    let (ticket, mut work) = episode();
    work.reserve_episode_activation(1).unwrap();
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://catalog.example.test/e/fixture".into(),
            document_generation: 2,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::NavigationIdentity {
            document_generation: 2,
            native_navigation_id: 43,
        },
    );
    assert_eq!(
        work.apply(
            &ticket,
            BrowserEvent::ActivationInvoked {
                document_generation: 1,
                native_navigation_id: 42
            }
        ),
        EventDisposition::Stale
    );
    assert_eq!(
        work.apply(&ticket, BrowserEvent::PlayerFrame(frame())),
        EventDisposition::Stale
    );
    assert_eq!(
        work.apply(
            &ticket,
            BrowserEvent::NavigationCompleted(MainNavigationDiagnostic {
                observed_unix_ms: DecimalU64(2),
                document_generation: DecimalU64(1),
                native_navigation_id: DecimalU64(42),
                http_status_code: Some(200),
                is_success: true,
                web_error_status: 0
            })
        ),
        EventDisposition::Stale
    );
    assert!(work.snapshot().navigation_diagnostic.is_none());
}
#[test]
fn cancel_wins_and_zero_status_is_not_promoted_to_http_success() {
    let (ticket, mut work) = episode();
    work.apply(
        &ticket,
        BrowserEvent::NavigationCompleted(MainNavigationDiagnostic {
            observed_unix_ms: DecimalU64(2),
            document_generation: DecimalU64(1),
            native_navigation_id: DecimalU64(42),
            http_status_code: Some(0),
            is_success: false,
            web_error_status: 1,
        }),
    );
    assert_eq!(
        work.snapshot()
            .navigation_diagnostic
            .unwrap()
            .http_status_code,
        Some(0)
    );
    assert_eq!(
        work.reserve_episode_activation(1).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    let (ticket, mut work) = episode();
    work.reserve_episode_activation(1).unwrap();
    work.cancel();
    assert_eq!(
        work.apply(&ticket, BrowserEvent::PlayerFrame(frame())),
        EventDisposition::Stale
    );
    work.apply(&ticket, BrowserEvent::Cancelled);
    assert_eq!(work.snapshot().state, BrowserWorkState::Cancelled);
    assert!(work.snapshot().player_frames.is_empty());
}
#[test]
fn candidate_limits_and_unverified_lineage_claims_fail_closed() {
    let (ticket, mut work) = episode();
    work.reserve_episode_activation(1).unwrap();
    for _ in 0..17 {
        work.apply(&ticket, BrowserEvent::PlayerFrame(frame()));
    }
    assert_eq!(
        work.snapshot().failure.unwrap().code,
        ErrorCode::ResourceExhausted
    );
    let (ticket, mut work) = episode();
    work.reserve_episode_activation(1).unwrap();
    let mut candidate = frame();
    candidate.lineage_verified = true;
    work.apply(&ticket, BrowserEvent::PlayerFrame(candidate));
    assert_eq!(
        work.snapshot().failure.unwrap().code,
        ErrorCode::InvalidArgument
    );
}

fn player_navigation(ticket: &BrowserTicket, work: &mut BrowserWork) {
    work.apply(
        ticket,
        BrowserEvent::Navigated {
            url: "https://frame.example.test/tv-show.php?v=fixture".into(),
            document_generation: 2,
        },
    );
    work.apply(
        ticket,
        BrowserEvent::NavigationIdentity {
            document_generation: 2,
            native_navigation_id: 43,
        },
    );
}
fn player_completion() -> MainNavigationDiagnostic {
    MainNavigationDiagnostic {
        observed_unix_ms: DecimalU64(3),
        document_generation: DecimalU64(2),
        native_navigation_id: DecimalU64(43),
        http_status_code: Some(0),
        is_success: false,
        web_error_status: 1,
    }
}
fn player_dom() -> BrowserDocument {
    BrowserDocument {
        kind: CaptureKind::DomSnapshot,
        final_url: "https://frame.example.test/tv-show.php?v=fixture".into(),
        document_generation: DecimalU64(2),
        observed_unix_ms: DecimalU64(4),
        body: Arc::from(b"fixture player error document".as_slice()),
        response: None,
    }
}
#[test]
fn activation_history_survives_navigation_without_promoting_late_invocation() {
    let (ticket, mut work) = episode();
    work.reserve_episode_activation(1).unwrap();
    player_navigation(&ticket, &mut work);
    assert!(work.snapshot().activation.is_none());
    assert_eq!(work.snapshot().activation_history.len(), 1);
    assert_eq!(
        work.snapshot().activation_history[0].native_navigation_id,
        DecimalU64(42)
    );
    assert_eq!(
        work.apply(
            &ticket,
            BrowserEvent::ActivationInvoked {
                document_generation: 1,
                native_navigation_id: 42
            }
        ),
        EventDisposition::Stale
    );
    assert_eq!(
        work.snapshot().activation_history[0].status,
        EpisodeActivationStatus::Enqueued
    );
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://catalog.example.test/e/fixture".into(),
            document_generation: 3,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::NavigationIdentity {
            document_generation: 3,
            native_navigation_id: 44,
        },
    );
    let mut completion = player_completion();
    completion.document_generation = DecimalU64(3);
    completion.native_navigation_id = DecimalU64(44);
    completion.is_success = true;
    completion.http_status_code = Some(200);
    work.apply(&ticket, BrowserEvent::NavigationCompleted(completion));
    let mut document = player_dom();
    document.document_generation = DecimalU64(3);
    document.final_url = "https://catalog.example.test/e/fixture".into();
    work.apply(&ticket, BrowserEvent::DocumentReady(document));
    assert_eq!(
        work.reserve_episode_activation(3).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
}
#[test]
fn player_readiness_requires_matching_completion_and_dom_and_preserves_error_status() {
    use super::episode_jobs::player_document_ready;
    let (ticket, mut work) = episode();
    work.reserve_episode_activation(1).unwrap();
    player_navigation(&ticket, &mut work);
    assert!(!player_document_ready(&work.snapshot(), DecimalU64(1)));
    work.apply(
        &ticket,
        BrowserEvent::NavigationCompleted(player_completion()),
    );
    assert!(!player_document_ready(&work.snapshot(), DecimalU64(1)));
    let mut stale = player_dom();
    stale.document_generation = DecimalU64(1);
    assert_eq!(
        work.apply(&ticket, BrowserEvent::DocumentReady(stale)),
        EventDisposition::Stale
    );
    work.apply(&ticket, BrowserEvent::DocumentReady(player_dom()));
    let snapshot = work.snapshot();
    assert!(player_document_ready(&snapshot, DecimalU64(1)));
    assert_eq!(
        snapshot.navigation_diagnostic.unwrap().http_status_code,
        Some(0)
    );
    assert_eq!(snapshot.state, BrowserWorkState::Running);
    assert!(snapshot.result.is_none());
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://frame.example.test/tv-show.php?v=fixture".into(),
            document_generation: 3,
        },
    );
    assert!(!player_document_ready(&work.snapshot(), DecimalU64(1)));
}
#[test]
fn requested_media_is_bounded_evidence_and_does_not_end_document_capture() {
    let (ticket, mut work) = episode();
    work.reserve_episode_activation(1).unwrap();
    player_navigation(&ticket, &mut work);
    let candidate = MediaRequestObservation {
        document_url: player_dom().final_url,
        document_generation: DecimalU64(2),
        native_navigation_id: DecimalU64(43),
        url: "https://cdn.example/video.m3u8".into(),
        referer: None,
        request_method: "GET".into(),
        observed_unix_ms: DecimalU64(3),
    };
    work.apply(&ticket, BrowserEvent::MediaRequested(candidate.clone()));
    assert_eq!(work.snapshot().state, BrowserWorkState::Running);
    assert!(work.snapshot().result.is_none());
    assert!(work.snapshot().media_requests[0].referer.is_none());
    work.apply(
        &ticket,
        BrowserEvent::NavigationCompleted(player_completion()),
    );
    work.apply(&ticket, BrowserEvent::DocumentReady(player_dom()));
    assert!(super::episode_jobs::player_document_ready(
        &work.snapshot(),
        DecimalU64(1)
    ));
    work.cancel();
    assert_eq!(
        work.apply(&ticket, BrowserEvent::MediaRequested(candidate)),
        EventDisposition::Stale
    );
    assert_eq!(work.snapshot().media_requests.len(), 1);
}
