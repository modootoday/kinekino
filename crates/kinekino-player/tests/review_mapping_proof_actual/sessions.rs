//! Explicit actual CPU session admission; called only by the allocated ignored test.
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode},
        review_playback::{
            ReviewChildOwnership, ReviewRuntimeBackend, ReviewRuntimeProfile,
            sessions::{Selection, SessionRef},
        },
        review_source::MutationControl,
    },
    domain::api_values::DecimalU64,
    infrastructure::{
        review_playback_sessions::LocalReviewSessionConfig,
        timeline_evidence::TrustedTimelineAdmission,
    },
};
use kinekino_process::infrastructure::family::JobBudget;
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};
struct Control(bool);
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0 {
            Err(ApiError::new(ErrorCode::Cancelled, "cancelled"))
        } else {
            Ok(())
        }
    }
}
pub(super) fn fixed_config(
    review_root: &Path,
    user_root: &Path,
    admission: TrustedTimelineAdmission,
    runtime_root: std::path::PathBuf,
    executable: std::path::PathBuf,
    budget: JobBudget,
) -> LocalReviewSessionConfig {
    LocalReviewSessionConfig {
        review_root: review_root.into(),
        user_root: user_root.into(),
        runtime_root,
        executable,
        admission,
        budget,
        profile: ReviewRuntimeProfile {
            backend: ReviewRuntimeBackend::OwnedLocalMpv,
            engine_sha256: "f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790"
                .into(),
            max_source_bytes: 128 * 1024 * 1024,
            max_job_committed_bytes: budget.max_committed_bytes as u64,
            cpu_threads: 1,
            startup_timeout_ms: 10000,
            child_ownership: ReviewChildOwnership::OwnedJobUntilPhysicalExit,
        },
        total: Duration::from_secs(35),
        cleanup: Duration::from_secs(2),
        raw_output_bytes: 1024 * 1024,
    }
}
pub fn verify(
    workspace: &Path,
    review_root: &Path,
    user_root: &Path,
    admission: TrustedTimelineAdmission,
    output: &Path,
    use_cli: bool,
) -> Value {
    let tool = workspace.join("runtime/toolchains/mpv-dde0671072-review-r1/mpv.exe");
    let tool_guard = super::custody::Guard::acquire(&tool);
    assert_eq!(
        tool_guard.sha256,
        "f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790"
    );
    let (mut mask, mut system) = (0usize, 0usize);
    unsafe {
        windows::Win32::System::Threading::GetProcessAffinityMask(
            windows::Win32::System::Threading::GetCurrentProcess(),
            &mut mask,
            &mut system,
        )
        .unwrap();
    }
    let budget = JobBudget {
        max_committed_bytes: 1024 * 1024 * 1024,
        affinity_mask: mask.isolate_lowest_one(),
    };
    let runtime_root = output.join("runtime");
    fs::create_dir(&runtime_root).unwrap();
    let selection = Selection {
        case: admission.case.clone(),
        association_id: if use_cli {
            "cli-configured"
        } else {
            "sealed-sprite"
        }
        .into(),
        association_revision: DecimalU64(2),
    };
    super::write_new(
        &output.join("RUNTIME-WITNESS.json"),
        &json!({"tool":tool_guard.evidence(),"selection":selection,"affinity_mask":budget.affinity_mask,"job_committed_bytes":budget.max_committed_bytes,"cpu_threads":1,"total_ms":35000,"cleanup_ms":2000,"headless":true}),
    );
    let configured_user = output.join("configured-user");
    let user_root = if use_cli {
        fs::create_dir(&configured_user).unwrap();
        configured_user.as_path()
    } else {
        user_root
    };
    let config = || {
        fixed_config(
            review_root,
            user_root,
            admission.clone(),
            runtime_root.clone(),
            tool.clone(),
            budget,
        )
    };
    let (sessions, client) = if use_cli {
        let assembly = kinekino_player::assembly::review_playback::assemble_sdk(
            kinekino_player::application::api::PlayerSdk::default(),
            config(),
            true,
            4,
        )
        .unwrap();
        (assembly.sessions, Some(assembly.client))
    } else {
        (
            kinekino_player::assembly::review_playback::assemble_local(config(), 4).unwrap(),
            None,
        )
    };
    if let Some(client) = client {
        let host = kinekino_player::assembly::review_playback::start_transport(
            client,
            sessions.clone(),
            kinekino_player::infrastructure::local_host::TransportOptions::default(),
        )
        .unwrap();
        let mut result = super::cli_sessions::verify(host, sessions.clone(), selection, output);
        let old: SessionRef =
            serde_json::from_value(result["connected"]["session"].clone()).unwrap();
        let restarted =
            kinekino_player::assembly::review_playback::assemble_local(config(), 4).unwrap();
        assert_eq!(
            restarted.status(&old).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        let mut unverified = old.selection.clone();
        unverified.association_revision = DecimalU64(1);
        let new = restarted.open(unverified, "open", &Control(false)).unwrap();
        assert_eq!(new.session.ordinal, old.ordinal);
        assert_ne!(new.session.host_scope, old.host_scope);
        assert_eq!(
            sessions.status(&new.session).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        let cutoff = Instant::now() + Duration::from_secs(10);
        let terminal = loop {
            let current = restarted.status(&new.session).unwrap();
            if current.observation.owner_finished {
                break current;
            }
            assert!(
                Instant::now() < cutoff,
                "restarted unverified owner did not finish"
            );
            thread::sleep(Duration::from_millis(20));
        };
        assert!(!terminal.observation.process_created && terminal.observation.physical_closed);
        assert_eq!(
            terminal.observation.failure.as_ref().unwrap().code,
            ErrorCode::NeedsInput
        );
        result["restart"] = json!({"old_reference_denied":true,"new_reference_denied_by_old_host":true,"ordinal_reused_without_authority_reuse":true,"new":terminal,"new_decode_execution":false});
        tool_guard.assert_unchanged();
        return result;
    }
    let opened = sessions
        .open(selection.clone(), "open", &Control(false))
        .unwrap();
    super::write_new(&output.join("OPEN.json"), &json!(opened));
    let replay = sessions
        .open(selection.clone(), "open", &Control(true))
        .unwrap();
    assert_eq!(replay.session, opened.session);
    let cutoff = Instant::now() + Duration::from_secs(15);
    let connected = loop {
        let current = sessions.status(&opened.session).unwrap();
        if current.observation.has_error {
            super::write_new(&output.join("STARTUP-FAILED.json"), &json!(current));
            sessions.close(&opened.session).unwrap();
            panic!("actual session startup failed: {current:?}");
        }
        if current.observation.connected && current.observation.engine_instance_id.is_some() {
            break current;
        }
        assert!(
            Instant::now() < cutoff,
            "session did not connect: {current:?}"
        );
        thread::sleep(Duration::from_millis(20));
    };
    super::write_new(&output.join("CONNECTED.json"), &json!(connected));
    let mut foreign = opened.session.clone();
    foreign.host_scope = "foreign-host".into();
    assert_eq!(
        sessions.close(&foreign).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let closing = sessions.close(&opened.session).unwrap();
    assert!(closing.close_requested);
    sessions.close(&opened.session).unwrap();
    super::write_new(&output.join("CLOSE.json"), &json!(closing));
    let cutoff = Instant::now() + Duration::from_secs(10);
    let closed = loop {
        let current = sessions.status(&opened.session).unwrap();
        if current.observation.owner_finished
            && current.observation.physical_closed
            && current.observation.family_empty
            && current.observation.ipc_exited
        {
            break current;
        }
        assert!(
            Instant::now() < cutoff,
            "session did not physically close: {current:?}"
        );
        thread::sleep(Duration::from_millis(20));
    };
    assert!(!closed.observation.has_error);
    assert_eq!(
        sessions.open(selection, "open", &Control(true)).unwrap(),
        closed
    );
    super::write_new(&output.join("CLOSED.json"), &json!(closed));
    let mut unverified = opened.session.selection.clone();
    unverified.association_revision = DecimalU64(1);
    let failed = sessions
        .open(unverified, "unverified", &Control(false))
        .unwrap();
    let cutoff = Instant::now() + Duration::from_secs(10);
    let rejected = loop {
        let current = sessions.status(&failed.session).unwrap();
        if current.observation.owner_finished {
            break current;
        }
        assert!(
            Instant::now() < cutoff,
            "unverified owner did not terminate"
        );
        thread::sleep(Duration::from_millis(20));
    };
    assert!(
        rejected.observation.has_error
            && rejected.observation.physical_closed
            && !rejected.observation.process_created
    );
    assert_eq!(
        rejected.observation.failure.as_ref().unwrap().code,
        ErrorCode::NeedsInput
    );
    super::write_new(&output.join("UNVERIFIED-NO-CHILD.json"), &json!(rejected));
    let live = sessions
        .open(
            opened.session.selection.clone(),
            "host-shutdown",
            &Control(false),
        )
        .unwrap();
    let cutoff = Instant::now() + Duration::from_secs(15);
    let live_connected = loop {
        let current = sessions.status(&live.session).unwrap();
        assert!(
            !current.observation.has_error,
            "host shutdown runtime failed: {current:?}"
        );
        if current.observation.connected {
            break current;
        }
        assert!(
            Instant::now() < cutoff,
            "host shutdown runtime did not connect"
        );
        thread::sleep(Duration::from_millis(20));
    };
    sessions.begin_close().unwrap();
    sessions.begin_close().unwrap();
    assert_eq!(
        sessions
            .open(
                live.session.selection.clone(),
                "after-host-close",
                &Control(false)
            )
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    let denied = sessions
        .control(
            &live.session,
            live_connected.observation.engine_instance_id.unwrap(),
            kinekino_player::domain::playback_control::PlaybackControl::Pause,
        )
        .unwrap_err();
    assert_eq!(denied.details["delivery"], "not_sent");
    assert_eq!(
        sessions
            .open(
                live.session.selection.clone(),
                "host-shutdown",
                &Control(true)
            )
            .unwrap()
            .session,
        live.session
    );
    let cutoff = Instant::now() + Duration::from_secs(10);
    let host_completion = loop {
        let current = sessions.observe_shutdown().unwrap();
        if current.complete {
            break current;
        }
        assert!(
            Instant::now() < cutoff,
            "host shutdown remained incomplete: {current:?}"
        );
        thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(host_completion.owners, 3);
    assert_eq!(host_completion.physically_finished, 3);
    let host_closed = sessions.status(&live.session).unwrap();
    assert!(!host_closed.observation.has_error);
    assert!(
        host_closed.observation.process_created
            && host_closed.observation.family_empty
            && host_closed.observation.ipc_exited
            && host_closed.observation.owner_finished
            && host_closed.observation.physical_closed
    );
    super::write_new(
        &output.join("HOST-SHUTDOWN.json"),
        &json!({"connected":live_connected,"closed":host_closed,"completion":host_completion,"new_open_denied":true,"new_control_not_sent":true,"same_key_replay_preserved":true}),
    );
    tool_guard.assert_unchanged();
    json!({"connected":connected,"closed":closed,"unverified":rejected,"host_shutdown":{"closed":host_closed,"completion":host_completion},"same_key_no_new_owner":true,"foreign_host_denied":true,"tool_unchanged":true,"transport_commands_registered":false,"quality_approved":false})
}
