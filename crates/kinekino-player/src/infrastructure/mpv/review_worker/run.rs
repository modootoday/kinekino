use super::arguments::arguments;
use super::*;

pub(super) fn run(
    prepare: impl FnOnce() -> Result<GuardedAdmittedReviewMedia<ReviewJobReservation>, ApiError>,
    launch: ReviewWorkerLaunch,
    check_custody: &dyn Fn() -> Result<(), ApiError>,
    check_control: &dyn Fn(
        &crate::domain::playback_control::PlaybackControl,
        &NativePlayback,
    ) -> Result<(), ApiError>,
    should_pause: &dyn Fn(&NativePlayback) -> Result<bool, ApiError>,
) -> Result<(), ApiError> {
    let ReviewWorkerLaunch {
        root,
        deadline,
        limits,
        close,
        state,
        controls,
        commands,
        gate,
    } = launch;
    let (state, controls) = (&state, &controls);
    check_custody()?;
    let input = prepare()?;
    let identity = input.identity().clone();
    input.check_current()?;
    update(state, |s| s.source = Some(input.identity().clone()));
    let (source, runtime) = input.into_parts();
    let (job, custody) = runtime.into_launch_parts();
    let (_root, _evidence_guards) = immutable_storage_paths::root(&root)?;
    let evidence = PreparedEvidence::create(&root, None, 1).map_err(io_error)?;
    let pipe = super::super::ipc_address("kinekino-review")
        .map_err(|_| ApiError::new(ErrorCode::EngineFailed, "Review IPC address unavailable"))?;
    let (sender, receiver) = tokio::sync::mpsc::channel(control::COMMAND_CAPACITY);
    let scope = HostScope::for_verified_standalone(&custody.tool);
    let control = MpvControlHandle::new_scoped(sender, scope.clone()).map_err(|_| {
        ApiError::new(
            ErrorCode::EngineFailed,
            "Review IPC owner construction failed",
        )
    })?;
    source.check_current()?;
    if !matches!(close.try_recv(), Err(mpsc::TryRecvError::Empty)) {
        update(state, |s| {
            s.closing = true;
            s.physical_closed = true;
            s.ipc_exited = true;
        });
        return Ok(());
    }
    let spec = SpawnSpec {
        executable: custody.tool.path().to_path_buf(),
        cwd: root.clone(),
        arguments: arguments(
            source.path(),
            &pipe,
            custody.profile.cpu_threads,
            &identity.classification,
        ),
        environment: None,
    };
    let _launch_record =
        super::launch_record::write(&root, &source, &custody, job.budget(), &spec)?;
    source.check_current()?;
    check_custody()?;
    let (process, paths) = match spawn_prepared(spec, evidence, deadline, job) {
        Ok(value) => value,
        Err(failure) => {
            let failure = *failure;
            update(state, |s| {
                first_error(
                    s,
                    format!(
                        "Review launch {}: {}; original termination error={:?}",
                        failure.stage, failure.error, failure.termination_error
                    ),
                )
            });
            if let Some(mut process) = failure.owner {
                update(state, |s| {
                    s.process_created = true;
                    s.pid = Some(process.identity().pid);
                    s.creation_filetime = process.identity().creation_filetime;
                    s.closing = true;
                });
                // An unassigned suspended child never ran. Observe BOTH its original
                // handle exit and original empty Job; do not turn Unknown into Empty.
                loop {
                    let exit = process.poll_child();
                    let family = process.poll_owned_job();
                    if let (Ok(Some(code)), Ok(FamilyObservation::Empty)) = (exit, family) {
                        update(state, |s| {
                            s.physical_closed = true;
                            s.child_exit = Some(code);
                            s.family_empty = true;
                            s.forced = true;
                            s.ipc_exited = true;
                        });
                        break;
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            } else {
                update(state, |s| {
                    s.physical_closed = true;
                    s.ipc_exited = true;
                });
            }
            if let Err(error) = super::launch_record::terminal(&root, state) {
                update(state, |s| first_error(s, format!("{error:?}")));
            }
            return Ok(());
        }
    };
    let mut owner = Supervisor::new(process, paths, limits.raw_output_bytes);
    update(state, |s| {
        s.process_created = true;
        s.pid = Some(owner.process().identity().pid);
        s.creation_filetime = owner.process().identity().creation_filetime;
        s.engine_instance_id = Some(control.instance_id());
    });
    let updates = Arc::new(Mutex::new(NativePlayback::default()));
    let mut ipc_worker = match ipc::connect(pipe, receiver, updates.clone(), control.state.clone())
    {
        Ok(worker) => Some(worker),
        Err(error) => {
            update(state, |s| {
                first_error(s, format!("Review IPC worker: {error}"))
            });
            None
        }
    };
    let mut closing = if ipc_worker.is_none() {
        Some(short_cleanup(deadline, limits.cleanup))
    } else {
        None
    };
    let startup = Instant::now()
        .checked_add(Duration::from_millis(custody.profile.startup_timeout_ms))
        .unwrap_or_else(Instant::now);
    let mut quit_requested = false;
    let mut boundary_pause = super::boundary_pause::BoundaryPause::default();
    loop {
        let requested_close = match gate.lock() {
            Ok(closing) => *closing,
            Err(_) => {
                update(state, |s| {
                    first_error(s, "Review control gate poisoned".into())
                });
                true
            }
        } || !matches!(close.try_recv(), Err(mpsc::TryRecvError::Empty));
        let observed = updates
            .lock()
            .map(|s| s.clone())
            .unwrap_or_else(|_| NativePlayback {
                error: Some("Review observations poisoned".into()),
                ..Default::default()
            });
        update(state, |s| s.playback = observed.clone());
        if observed.connected
            && closing.is_none()
            && let Ok(mut target) = controls.lock()
        {
            *target = Some(control.clone());
        }
        if closing.is_none() {
            let failure = source
                .check_current()
                .and_then(|()| check_custody())
                .err()
                .map(|e| format!("{e:?}"))
                .or_else(|| {
                    owner
                        .evidence()
                        .check_limit(limits.raw_output_bytes)
                        .err()
                        .map(|e| e.to_string())
                })
                .or_else(|| owner.poll_state().err().map(|e| e.error.to_string()))
                .or_else(|| observed.error.clone())
                .or_else(|| {
                    ipc_worker
                        .as_ref()
                        .filter(|worker| worker.is_finished())
                        .map(|_| "Review IPC worker ended before owned close".into())
                });
            let ended = owner.receipt().child_exit.is_some();
            if let Some(code) = owner.receipt().child_exit.filter(|code| *code != 0) {
                update(state, |s| {
                    first_error(s, format!("Review engine exited with code {code}"))
                });
            }
            let timed_out = deadline.operation_remaining(Instant::now()).is_zero()
                || (!observed.connected && Instant::now() >= startup);
            if let Some(error) = failure {
                update(state, |s| first_error(s, error));
                closing = Some(short_cleanup(deadline, limits.cleanup));
            }
            if requested_close || ended || timed_out {
                if timed_out {
                    update(state, |s| {
                        first_error(
                            s,
                            "Review engine original operation/startup deadline elapsed".into(),
                        )
                    });
                }
                closing = Some(short_cleanup(deadline, limits.cleanup));
            }
        }
        if closing.is_none() && observed.connected {
            let policy_result = should_pause(&observed).and_then(|pause| {
                if boundary_pause.should_publish(pause, Instant::now())? {
                    let closing_gate = gate.lock().map_err(|_| {
                        ApiError::new(ErrorCode::Internal, "Review pause gate poisoned")
                    })?;
                    if !*closing_gate {
                        source.check_current()?;
                        check_custody()?;
                        control.submit(crate::domain::playback_control::PlaybackControl::Pause)?;
                        boundary_pause.published(Instant::now());
                    }
                }
                Ok(())
            });
            if let Err(error) = policy_result {
                update(state, |state| {
                    let first = state.first_error.is_none();
                    first_error(state, format!("{error:?}"));
                    if first {
                        state.first_error_code = Some(error.code);
                    }
                });
                closing = Some(short_cleanup(deadline, limits.cleanup));
            }
        }
        if closing.is_some() && !quit_requested {
            let _ = control.send_legacy(serde_json::json!(["quit"]));
            quit_requested = true;
        }
        if closing.is_some()
            && let Ok(mut closing) = gate.lock()
        {
            *closing = true;
        }
        commands::process_pending(&commands, &identity, &control, deadline, &gate, &|intent| {
            source.check_current()?;
            check_custody()?;
            check_control(intent, &observed)
        });
        if let Some(cleanup) = closing {
            control
                .state
                .connected
                .store(false, std::sync::atomic::Ordering::Release);
            if let Ok(mut target) = controls.lock() {
                *target = None;
            }
            update(state, |s| s.closing = true);
            let progress = owner.poll_cleanup(cleanup, &SystemClock);
            if let Some(error) = progress.first_error() {
                update(state, |s| {
                    first_error(
                        s,
                        format!("Review cleanup {:?}: {}", error.stage, error.error),
                    )
                });
            }
            if progress.physical_closed() {
                control.state.close();
                if ipc_worker
                    .as_ref()
                    .is_none_or(thread::JoinHandle::is_finished)
                {
                    if let Some(worker) = ipc_worker.take()
                        && worker.join().is_err()
                    {
                        update(state, |s| {
                            first_error(s, "Review IPC worker panicked".into())
                        });
                    }
                    let receipt = owner.receipt();
                    update(state, |s| {
                        s.physical_closed = true;
                        s.child_exit = receipt.child_exit;
                        s.family_empty = receipt.family == FamilyObservation::Empty;
                        s.forced = receipt.forced;
                        s.ipc_exited = true;
                    });
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    // Original source, fixed tool, exclusive lock and Job owner survive every
    // normal/error/late-cleanup poll above, including after the public handle drops.
    if let Err(error) = super::launch_record::terminal(&root, state) {
        update(state, |s| first_error(s, format!("{error:?}")));
    }
    drop(owner);
    drop(custody);
    drop(source);
    drop(scope);
    Ok(())
}
fn short_cleanup(original: Deadline, cleanup: Duration) -> Deadline {
    Deadline::new(Instant::now(), cleanup.saturating_mul(2), cleanup)
        .map(|d| original.constrained_by(d))
        .unwrap_or(original)
}
