use super::*;

#[test]
fn host_close_blocks_new_work_preserves_replay_and_requires_every_physical_gate() {
    let h = Arc::new(Harness::default());
    h.control_available.store(true, Ordering::SeqCst);
    h.host_observe.store(true, Ordering::SeqCst);
    let sessions = ReviewPlaybackSessions::new("host".into(), 4, Launcher(h.clone())).unwrap();
    assert_eq!(
        sessions.observe_shutdown().unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let opened = sessions.open(selection(), "open", &Control(false)).unwrap();
    sessions.begin_close().unwrap();
    sessions.begin_close().unwrap();
    assert_eq!(h.closes.load(Ordering::SeqCst), 1);
    assert!(!sessions.observe_shutdown().unwrap().complete);
    assert_eq!(
        sessions
            .open(selection(), "new", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    let denied = sessions
        .control(&opened.session, DecimalU64(1), PlaybackControl::Pause)
        .unwrap_err();
    assert_eq!(denied.code, ErrorCode::NotFound);
    assert_eq!(denied.details["delivery"], "not_sent");
    assert_eq!(h.controls.load(Ordering::SeqCst), 0);
    assert_eq!(
        sessions
            .open(selection(), "open", &Control(true))
            .unwrap()
            .session,
        opened.session
    );
    h.deny.store(true, Ordering::SeqCst);
    *h.state.lock().unwrap() = OwnerObservation {
        process_created: true,
        owner_finished: true,
        physical_closed: true,
        ..Default::default()
    };
    assert!(!sessions.observe_shutdown().unwrap().complete);
    h.state.lock().unwrap().family_empty = true;
    assert!(!sessions.observe_shutdown().unwrap().complete);
    h.state.lock().unwrap().ipc_exited = true;
    let completed = sessions.observe_shutdown().unwrap();
    assert_eq!(completed.owners, 1);
    assert_eq!(completed.physically_finished, 1);
    assert!(completed.complete);
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
    assert_eq!(h.drops.load(Ordering::SeqCst), 0);
}

#[test]
fn host_close_error_and_observation_loss_keep_original_owner_and_admission_closed() {
    let h = Arc::new(Harness::default());
    h.host_observe.store(true, Ordering::SeqCst);
    let sessions = ReviewPlaybackSessions::new("host".into(), 4, Launcher(h.clone())).unwrap();
    sessions.open(selection(), "open", &Control(false)).unwrap();
    h.fail_close.store(true, Ordering::SeqCst);
    let first = sessions.begin_close().unwrap_err();
    assert_eq!(first.message, "close request lost");
    assert_eq!(first.details["owner_retained"], true);
    assert_eq!(
        sessions
            .open(selection(), "new", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    h.fail_observe.store(true, Ordering::SeqCst);
    assert_eq!(
        sessions.observe_shutdown().unwrap_err().details["owner_retained"],
        true
    );
    assert_eq!(h.drops.load(Ordering::SeqCst), 0);
    h.fail_observe.store(false, Ordering::SeqCst);
    h.fail_close.store(false, Ordering::SeqCst);
    sessions.begin_close().unwrap();
    *h.state.lock().unwrap() = OwnerObservation {
        owner_finished: true,
        physical_closed: true,
        ..Default::default()
    };
    assert!(sessions.observe_shutdown().unwrap().complete);
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
}

#[test]
fn empty_host_only_completes_after_admission_has_closed() {
    let h = Arc::new(Harness::default());
    let sessions = ReviewPlaybackSessions::new("host".into(), 4, Launcher(h.clone())).unwrap();
    assert!(sessions.observe_shutdown().is_err());
    sessions.begin_close().unwrap();
    assert_eq!(
        sessions.observe_shutdown().unwrap(),
        ShutdownObservation {
            owners: 0,
            physically_finished: 0,
            complete: true
        }
    );
    assert!(sessions.open(selection(), "new", &Control(false)).is_err());
    assert_eq!(h.starts.load(Ordering::SeqCst), 0);
}
