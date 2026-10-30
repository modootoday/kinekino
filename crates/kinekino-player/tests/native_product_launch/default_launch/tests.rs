use super::*;
use kinekino_process::{
    application::supervisor::ProcessFamily,
    domain::{identity::ProcessIdentity, observation::FamilyObservation},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::{cell::Cell, io, rc::Rc};
use windows::Win32::UI::WindowsAndMessaging::{
    WS_CAPTION, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MAXIMIZE, WS_POPUP,
    WS_SYSMENU, WS_VISIBLE,
};

struct PanicProof(Arc<AtomicUsize>);
impl Drop for PanicProof {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn host_log_retains_typed_control_and_observation_errors_without_replacing_case_first() {
    let case = owner::Outcome::NotStarted(Failure::new(Stage::Sdk, "case-first"));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut log = control::FailureLog::default();
    log.record(
        control::ErrorLane::Control,
        Failure::new(Stage::Close, io::Error::from_raw_os_error(5)),
    );
    log.record(
        control::ErrorLane::Observation,
        Failure::panic(Box::new(PanicProof(drops.clone()))),
    );
    {
        let retained = log.retained().collect::<Vec<_>>();
        assert_eq!(retained.len(), 2);
        assert_eq!(
            retained[0]
                .original
                .downcast_ref::<io::Error>()
                .unwrap()
                .raw_os_error(),
            Some(5)
        );
        assert!(
            retained[1]
                .original
                .downcast_ref::<Box<dyn Any + Send>>()
                .unwrap()
                .downcast_ref::<PanicProof>()
                .is_some()
        );
    }
    let value = log.evidence();
    assert_eq!(value["first"]["lane"], "Control");
    assert_eq!(value["secondary"][0]["lane"], "Observation");
    assert_eq!(case.description()["error"]["message"], "case-first");
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(log);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
#[test]
fn bounded_error_log_marks_overflow_and_preserves_the_original_first_object() {
    let mut log = control::FailureLog::default();
    log.record(
        control::ErrorLane::Control,
        Failure::new(Stage::Close, io::Error::from_raw_os_error(5)),
    );
    for code in 6..14 {
        log.record(
            control::ErrorLane::Observation,
            Failure::new(Stage::Identity, io::Error::from_raw_os_error(code)),
        );
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let overflow = log.record(
        control::ErrorLane::Transport,
        Failure::panic(Box::new(PanicProof(drops.clone()))),
    );
    assert_eq!(overflow["original_retained"], false);
    assert_eq!(log.retained().count(), 9);
    assert_eq!(log.evidence()["overflow"], 1);
    assert_eq!(log.evidence()["transport_fenced"], true);
    assert_eq!(
        log.retained()
            .next()
            .unwrap()
            .original
            .downcast_ref::<io::Error>()
            .unwrap()
            .raw_os_error(),
        Some(5)
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
#[test]
fn pin_binding_rejects_substituted_file_or_final_path_even_when_sha_matches() {
    let sha = "a".repeat(64);
    assert!(verify_pin(&sha, &sha).is_ok());
    let original = ImageIdentity {
        volume: 1,
        file_id: 2,
        final_path: PathBuf::from(r"C:\fixed\player.exe"),
    };
    assert!(check_binding(&original, &original).is_ok());
    let mut changed = original.clone();
    changed.file_id = 3;
    assert_eq!(
        check_binding(&original, &changed).unwrap_err().stage,
        Stage::Pin
    );
    changed = original.clone();
    changed.volume = 4;
    assert!(check_binding(&original, &changed).is_err());
    changed = original.clone();
    changed.final_path = PathBuf::from(r"C:\other\player.exe");
    assert!(check_binding(&original, &changed).is_err());
}
#[test]
fn pin_requires_ordinary_local_input_and_window_checks_do_not_reresolve_it() {
    let fixed = PathBuf::from(r"C:\fixed\player.exe");
    assert!(ordinary_executable(&fixed));
    for input in [
        r"relative.exe",
        r"C:\fixed\..\other.exe",
        r"\\server\share\player.exe",
        r"\\?\C:\fixed\player.exe",
    ] {
        assert!(!ordinary_executable(Path::new(input)));
    }
    let original = ProcessIdentity {
        pid: 7,
        creation_filetime: Some(9),
        executable: fixed.clone(),
    };
    assert!(window::check_identity(&original, &fixed, None).is_ok());
    assert!(window::check_identity(&original, &fixed, Some(0)).is_err());
    assert!(window::check_identity(&original, Path::new(r"C:\other.exe"), None).is_err());
}

fn probe() -> Probe {
    Probe {
        root: std::env::temp_dir(),
        id: "known-project".into(),
        revision: 2,
        sha256: "a".repeat(64),
    }
}

#[test]
fn noargs_is_exactly_empty_and_cannot_silently_take_a_store() {
    assert!(arguments(Mode::NoArgs, None).unwrap().is_empty());
    assert_eq!(
        arguments(Mode::NoArgs, Some(&probe())).unwrap_err().stage,
        Stage::Admission
    );
}
#[test]
fn configured_arguments_select_only_recording_and_one_explicit_project_store() {
    let probe = probe();
    let args = arguments(Mode::Configured, Some(&probe)).unwrap();
    assert_eq!(
        args,
        vec![
            OsString::from("--record-validation"),
            OsString::from("--project-root"),
            probe.root.into_os_string()
        ]
    );
    assert_eq!(
        arguments(Mode::Configured, None).unwrap_err().stage,
        Stage::Admission
    );
}
#[test]
fn malformed_project_probe_fails_before_any_process_ownership() {
    let mut value = probe();
    value.root = "relative".into();
    assert!(arguments(Mode::Configured, Some(&value)).is_err());
    value = probe();
    value.id = "../foreign".into();
    assert!(arguments(Mode::Configured, Some(&value)).is_err());
    value = probe();
    value.revision = 0;
    assert!(arguments(Mode::Configured, Some(&value)).is_err());
    value = probe();
    value.sha256 = "not-a-digest".into();
    assert!(arguments(Mode::Configured, Some(&value)).is_err());
}
#[test]
fn mismatched_or_invalid_image_digest_is_a_pin_error() {
    let expected = "a".repeat(64);
    assert!(verify_pin(&expected, &expected).is_ok());
    assert!(verify_pin(&expected.to_ascii_uppercase(), &expected).is_ok());
    assert_eq!(
        verify_pin(&expected, &"b".repeat(64)).unwrap_err().stage,
        Stage::Pin
    );
    assert_eq!(verify_pin("", "").unwrap_err().stage, Stage::Pin);
}
#[test]
fn original_io_and_process_error_objects_survive_classification() {
    let first = Failure::new(Stage::Window, io::Error::from_raw_os_error(5));
    assert_eq!(
        first
            .original
            .downcast_ref::<io::Error>()
            .unwrap()
            .raw_os_error(),
        Some(5)
    );
    let original = kinekino_process::domain::observation::FailureCause {
        stage: kinekino_process::domain::observation::FailureStage::FamilyObservation,
        error: io::Error::from_raw_os_error(6),
    };
    let classified = Failure::debug(Stage::Identity, original);
    assert_eq!(
        classified
            .original
            .downcast_ref::<kinekino_process::domain::observation::FailureCause>()
            .unwrap()
            .error
            .raw_os_error(),
        Some(6)
    );
}
#[test]
fn window_admission_rejects_foreign_child_hidden_and_unidentified_roots() {
    let valid = viewer_window_identity();
    assert!(window::accepts(&valid, 7));
    assert!(!window::accepts(&valid, 9));
    let mut invalid = valid.clone();
    invalid.handle = 0;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.visible = false;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.top_level = false;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.thread = 0;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.class.clear();
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.class = "Winit Thread Event Target".into();
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.parent = 1;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.owner = 1;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.style |= WS_POPUP.0;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.style &= !WS_CAPTION.0;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.ex_style |= WS_EX_TOOLWINDOW.0;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.ex_style |= WS_EX_NOACTIVATE.0;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid.clone();
    invalid.ex_style &= !WS_EX_APPWINDOW.0;
    assert!(!window::accepts(&invalid, 7));
    invalid = valid;
    invalid.title = "foreign caller title".into();
    assert!(!window::accepts(&invalid, 7));
}

fn viewer_window_identity() -> window::Identity {
    window::Identity {
        handle: 123,
        pid: 7,
        thread: 8,
        visible: true,
        top_level: true,
        class: "Window Class".into(),
        title: meari_native::application::WindowOptions::default().title,
        parent: 0,
        owner: 0,
        style: WS_VISIBLE.0 | WS_CAPTION.0 | WS_SYSMENU.0,
        ex_style: WS_EX_APPWINDOW.0,
    }
}

#[test]
fn viewer_selection_waits_past_internal_targets_for_the_application_window() {
    let viewer = viewer_window_identity();
    let mut helper = viewer.clone();
    helper.handle = 59314770;
    helper.class = "Winit Thread Event Target".into();
    helper.title.clear();
    helper.style = WS_VISIBLE.0 | WS_POPUP.0;
    helper.ex_style = WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0;
    assert!(window::select([helper.clone()], 7).unwrap().is_none());
    let mut hidden = viewer.clone();
    hidden.visible = false;
    assert!(
        window::select([helper.clone(), hidden], 7)
            .unwrap()
            .is_none()
    );
    helper.title = viewer.title.clone();
    assert!(!window::accepts(&helper, 7));
    let selected = window::select([helper, viewer.clone()], 7)
        .unwrap()
        .unwrap();
    assert_eq!(selected.handle, viewer.handle);
}

#[test]
fn viewer_selection_rejects_multiple_owned_application_windows() {
    let viewer = viewer_window_identity();
    let mut second = viewer.clone();
    second.handle += 1;
    let error = window::select([viewer.clone(), second], 7).unwrap_err();
    assert_eq!(error.stage, Stage::Window);
    assert_eq!(error.message, "Multiple owned visible root windows");
    let mut foreign = viewer.clone();
    foreign.pid = 9;
    assert_eq!(
        window::select([foreign, viewer.clone()], 7)
            .unwrap()
            .unwrap()
            .handle,
        viewer.handle
    );
}

#[test]
fn viewer_close_recheck_rejects_changed_or_destroyed_window_without_replacing_query_error() {
    let original = viewer_window_identity();
    assert!(window::check_close_target(&original, Ok(original.clone()), 7).is_ok());
    let mut maximized = original.clone();
    maximized.style |= WS_MAXIMIZE.0;
    assert!(window::check_close_target(&original, Ok(maximized), 7).is_ok());
    for field in 0..10 {
        let mut changed = original.clone();
        match field {
            0 => changed.handle += 1,
            1 => changed.pid += 1,
            2 => changed.thread += 1,
            3 => changed.class = "Winit Thread Event Target".into(),
            4 => changed.title = "changed caller title".into(),
            5 => changed.owner = 1,
            6 => changed.style |= WS_POPUP.0,
            7 => changed.visible = false,
            8 => changed.parent = 1,
            _ => changed.ex_style |= WS_EX_TOOLWINDOW.0,
        }
        assert_eq!(
            window::check_close_target(&original, Ok(changed), 7)
                .unwrap_err()
                .stage,
            Stage::Close
        );
    }
    let destroyed = Failure::new(Stage::Window, io::Error::from_raw_os_error(1400));
    let message = destroyed.message.clone();
    let original_error =
        destroyed.original.downcast_ref::<io::Error>().unwrap() as *const io::Error;
    let error = window::check_close_target(&original, Err(destroyed), 7).unwrap_err();
    assert_eq!(error.stage, Stage::Window);
    assert_eq!(error.message, message);
    assert_eq!(
        error.original.downcast_ref::<io::Error>().unwrap() as *const io::Error,
        original_error
    );
    assert_eq!(
        error
            .original
            .downcast_ref::<io::Error>()
            .unwrap()
            .raw_os_error(),
        Some(1400)
    );
    let first_query = Failure::new(Stage::Window, io::Error::from_raw_os_error(5));
    let first_message = first_query.message.clone();
    let first_object =
        first_query.original.downcast_ref::<io::Error>().unwrap() as *const io::Error;
    let later_query = Failure::new(Stage::Window, io::Error::from_raw_os_error(1400));
    let later_message = later_query.message.clone();
    let later_object =
        later_query.original.downcast_ref::<io::Error>().unwrap() as *const io::Error;
    let mut query_error = None;
    window::retain_first_query_error(&mut query_error, first_query);
    window::retain_first_query_error(&mut query_error, later_query);
    let kept = query_error.unwrap();
    assert_eq!(kept.stage, Stage::Window);
    assert_eq!(kept.message, first_message);
    assert_ne!(kept.message, later_message);
    let kept_io = kept.original.downcast_ref::<io::Error>().unwrap();
    assert_eq!(kept_io.raw_os_error(), Some(5));
    assert_eq!(kept_io as *const io::Error, first_object);
    assert_ne!(kept_io as *const io::Error, later_object);
}

struct Family {
    identity: ProcessIdentity,
    deadline: Deadline,
    terminated: Rc<Cell<usize>>,
    dropped: Rc<Cell<usize>>,
}
impl ProcessFamily for Family {
    fn deadline(&self) -> Deadline {
        self.deadline
    }
    fn identity(&self) -> &ProcessIdentity {
        &self.identity
    }
    fn close_stdin(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn poll_child(&mut self) -> io::Result<Option<u32>> {
        Ok(None)
    }
    fn poll_family(&mut self) -> io::Result<FamilyObservation> {
        Ok(FamilyObservation::Active(1))
    }
    fn terminate_owned_family(&mut self) -> io::Result<()> {
        self.terminated.set(self.terminated.get() + 1);
        Ok(())
    }
}
impl Drop for Family {
    fn drop(&mut self) {
        self.dropped.set(self.dropped.get() + 1);
    }
}
#[test]
fn noforce_records_denial_without_terminating_or_consuming_the_original_family() {
    let terminated = Rc::new(Cell::new(0));
    let dropped = Rc::new(Cell::new(0));
    let family = Family {
        identity: ProcessIdentity {
            pid: 7,
            creation_filetime: Some(9),
            executable: "original.exe".into(),
        },
        deadline: Deadline::new(
            Instant::now(),
            Duration::from_secs(2),
            Duration::from_secs(1),
        )
        .unwrap(),
        terminated: terminated.clone(),
        dropped: dropped.clone(),
    };
    let mut wrapped = owner::NoForce::new(family);
    assert_eq!(
        wrapped.terminate_owned_family().unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(wrapped.denied_requests(), 1);
    assert_eq!(wrapped.identity().creation_filetime, Some(9));
    assert_eq!(terminated.get(), 0);
    assert_eq!(dropped.get(), 0);
    let panic = capture_phase::<()>(|| {
        let _borrowed_original = wrapped.identity();
        panic!("controlled observer failure");
    })
    .unwrap_err();
    assert_eq!(panic.stage, Stage::Panic);
    assert!(
        panic
            .original
            .downcast_ref::<Box<dyn Any + Send>>()
            .is_some()
    );
    assert_eq!(dropped.get(), 0);
    drop(wrapped);
    assert_eq!(dropped.get(), 1); // Scope retention is not cross-process handoff.
}
#[test]
fn wrong_handoff_version_or_mode_is_not_os_admission() {
    assert!(control::ready_for_mode(1, Mode::NoArgs, Mode::NoArgs));
    assert!(!control::ready_for_mode(2, Mode::NoArgs, Mode::NoArgs));
    assert!(!control::ready_for_mode(1, Mode::NoArgs, Mode::Configured));
}

#[test]
fn bounded_control_header_and_cleanup_case_scope_are_required() {
    let mut request = control::Request {
        version: 1,
        request_id: "req-1".into(),
        nonce: "a".repeat(64),
        case_id: None,
        command: control::Command::State,
    };
    assert!(control::valid_header(&request));
    request.nonce.clear();
    assert!(!control::valid_header(&request));
    request.nonce = "a".repeat(64);
    request.version = 2;
    assert!(!control::valid_header(&request));
    request.version = 1;
    request.request_id = "../path".into();
    assert!(!control::valid_header(&request));
    request.request_id = "x".repeat(65);
    assert!(!control::valid_header(&request));
    request.request_id = "req-1".into();
    request.command = control::Command::Close;
    assert!(!control::valid_header(&request));
    request.case_id = Some("original-1".into());
    assert!(control::valid_header(&request));
}
#[test]
fn controller_cannot_close_release_finish_or_abort_original_owner() {
    assert!(control::permitted(
        &control::Command::Run { mode: Mode::NoArgs },
        false
    ));
    assert!(control::permitted(&control::Command::State, false));
    assert!(!control::permitted(
        &control::Command::Run { mode: Mode::NoArgs },
        true
    ));
    for command in [
        control::Command::Close,
        control::Command::ReleaseClosed,
        control::Command::FinishHost,
        control::Command::AbortHost {
            approval: "token".into(),
        },
    ] {
        assert!(!control::permitted(&command, false));
        assert!(control::permitted(&command, true));
    }
}
#[test]
fn request_replay_does_not_authorize_a_new_body_or_run_retry() {
    assert_eq!(control::replay(None, "a"), control::Replay::Miss);
    assert_eq!(control::replay(Some("a"), "a"), control::Replay::Same);
    assert_eq!(control::replay(Some("a"), "b"), control::Replay::Conflict);
    assert!(control::run_allowed(
        false,
        false,
        Mode::NoArgs,
        Mode::NoArgs
    ));
    assert!(!control::run_allowed(
        true,
        false,
        Mode::NoArgs,
        Mode::NoArgs
    ));
    assert!(!control::run_allowed(
        false,
        true,
        Mode::NoArgs,
        Mode::NoArgs
    ));
    assert!(!control::run_allowed(
        false,
        false,
        Mode::NoArgs,
        Mode::Configured
    ));
}
#[test]
fn finish_host_requires_the_original_carrier_to_be_released() {
    assert!(!control::finish_allowed(true));
    assert!(control::finish_allowed(false));
}
#[test]
fn unobserved_native_outcome_is_report_failure_and_noargs_does_not_invent_telemetry() {
    let observations = Observations::new();
    assert!(observations.check_report(Mode::NoArgs).is_ok());
    assert_eq!(
        observations
            .check_report(Mode::Configured)
            .unwrap_err()
            .stage,
        Stage::Report
    );
}
