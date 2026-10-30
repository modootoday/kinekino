//! Thin actual consumers of the accepted retaining CLI paths. Uses the existing
//! original-handle/Deadline/EOF harness; does not define another process driver.
use super::{
    fixture::{Fixture, request},
    kinekino_process::{
        CliFailure, HostTerminal, OwnedHost, ScenarioFailure, close_request, phase_deadline,
    },
    playback_owner::BoundedNativeOwner,
    provider::try_terminal_until,
    retaining_storage_paths,
};
use kinekino_jobs::infrastructure::RootedJobPaths;
use kinekino_player::application::jobs::{JobId, JobState};
use kinekino_process::domain::{
    deadline::Deadline,
    observation::{FamilyObservation, ProcessReceipt},
};
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs, io,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const READY: Duration = Duration::from_secs(10);
const REQUEST: Duration = Duration::from_secs(10);
const STOP: Duration = Duration::from_secs(15);

// Test-only port composition consumes the exact product guard source. The
// product's private module/API remains private and no guard policy is copied.
struct ConsumerPaths;
impl RootedJobPaths for ConsumerPaths {
    fn root(
        &self,
        path: &Path,
    ) -> Result<(PathBuf, Vec<fs::File>), kinekino_player::application::api::ApiError> {
        retaining_storage_paths::root(path)
    }
    fn directory(
        &self,
        path: &Path,
    ) -> Result<fs::File, kinekino_player::application::api::ApiError> {
        retaining_storage_paths::directory(path)
    }
    fn file(&self, path: &Path) -> Result<fs::File, kinekino_player::application::api::ApiError> {
        retaining_storage_paths::file(path)
    }
    fn lock(&self, path: &Path) -> Result<fs::File, kinekino_player::application::api::ApiError> {
        retaining_storage_paths::lock(path)
    }
}

fn original(fixture: &Fixture) -> Result<Deadline, ScenarioFailure> {
    Deadline::new(
        Instant::now(),
        Duration::from_secs(90),
        Duration::from_secs(10),
    )
    .map_err(|error| {
        ScenarioFailure::host(CliFailure::not_spawned(
            "consumer budget",
            error,
            &fixture.root,
        ))
    })
}
fn error(message: &str) -> io::Error {
    io::Error::other(message)
}
fn lease(path: &Path, held: bool) -> io::Result<()> {
    match ConsumerPaths.lock(path) {
        Err(error)
            if held
                && error.code == kinekino_player::application::api::ErrorCode::RevisionConflict =>
        {
            Ok(())
        }
        Ok(guard) if !held => {
            drop(guard);
            Ok(())
        }
        Ok(_) => Err(error("Original live host released its exclusive lease")),
        Err(actual) => Err(io::Error::other(format!(
            "Original host lease observation: {actual:?}"
        ))),
    }
}
fn physical(fixture: &Fixture, receipt: &ProcessReceipt) -> io::Result<()> {
    if receipt.child_exit != Some(0)
        || receipt.family != FamilyObservation::Empty
        || receipt.forced
        || !receipt.stdin_eof_requested
    {
        return Err(error("Original host did not close gracefully after EOF"));
    }
    for name in ["stop-export.json", "stderr-export.json"] {
        let metadata: Value =
            serde_json::from_slice(&fs::read(fixture.root.join("owned").join(name))?)?;
        if metadata["complete"] != true || metadata["limit_exceeded"] != false {
            return Err(error("Original raw output was not complete and bounded"));
        }
    }
    Ok(())
}
fn joined(stopped: &Value) -> io::Result<()> {
    let status = &stopped["local_host_shutdown"];
    if status["pipe"] != "joined"
        || status["sdk"] != "joined"
        || status["dispatch_poisoned"] != false
        || !matches!(status["dispatch"].as_str(), Some("idle" | "completed"))
    {
        return Err(error("Original local transport and SDK were not joined"));
    }
    Ok(())
}
fn simple_ready(
    fixture: &Fixture,
    provider: bool,
    deadline: Deadline,
) -> Result<(OwnedHost, String), ScenarioFailure> {
    let mut arguments: Vec<OsString> = vec![
        if provider {
            "--serve-provider"
        } else {
            "--serve-jobs"
        }
        .into(),
        "--job-root".into(),
        fixture.root.join("store").into_os_string(),
    ];
    if provider {
        for directory in ["catalog", "profile"] {
            fs::create_dir(fixture.root.join(directory)).map_err(|error| {
                ScenarioFailure::host(CliFailure::not_spawned(
                    "consumer directories",
                    error,
                    &fixture.root,
                ))
            })?;
        }
        arguments.extend([
            "--catalog-root".into(),
            fixture.root.join("catalog").into_os_string(),
            "--browser-profile-root".into(),
            fixture.root.join("profile").into_os_string(),
        ]);
    }
    let cutoff = phase_deadline(deadline, READY).map_err(|error| {
        ScenarioFailure::host(CliFailure::not_spawned(
            "consumer ready cutoff",
            error,
            &fixture.root,
        ))
    })?;
    OwnedHost::launch(fixture.cli.clone(), arguments, &fixture.root, deadline)
        .and_then(|host| {
            host.ready(cutoff, &fixture.root.join("logs/ready.stdout"), |bytes| {
                let ready: Value = serde_json::from_slice(bytes)?;
                let event = if provider {
                    "provider_host_ready"
                } else {
                    "job_host_ready"
                };
                if ready["event"] != event || ready["scope"] != "current_user_same_privilege" {
                    return Err(error("Original retaining host ready contract differs"));
                }
                ready["endpoint"]
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| error("Missing original retaining endpoint"))
            })
        })
        .map_err(ScenarioFailure::host)
}
fn finish_simple(
    fixture: &Fixture,
    terminal: HostTerminal,
    provider: bool,
) -> Result<ProcessReceipt, Box<CliFailure>> {
    let result = (|| -> io::Result<()> {
        physical(fixture, &terminal.receipt)?;
        let stopped: Value = serde_json::from_slice(&terminal.tail)?;
        joined(&stopped)?;
        let event = if provider {
            "provider_host_stopped"
        } else {
            "job_host_stopped"
        };
        if stopped["event"] != event {
            return Err(error("Original retaining stop event differs"));
        }
        let jobs = if provider {
            if stopped["receipt"]["browser_worker_exited"] != true {
                return Err(error("Original provider browser exit was not observed"));
            }
            &stopped["receipt"]["jobs"]
        } else {
            &stopped["receipt"]
        };
        if jobs["completed_total"] != if provider { json!(0) } else { json!(1) }
            || jobs["cancelled_total"] != 0
            || jobs["interrupted_total"] != 0
        {
            return Err(error("Original retaining job totals differ"));
        }
        lease(&fixture.root.join("store/jobs-owner.lock"), false)?;
        if provider {
            lease(&fixture.root.join("profile/provider-owner.lock"), false)?;
        }
        if terminal.deadline.remaining(Instant::now()).is_zero() {
            return Err(error(
                "Consumer receipt checks exceeded original stop cutoff",
            ));
        }
        Ok(())
    })();
    result.map_err(|error| {
        CliFailure::terminal(
            "retaining EOF contract",
            error,
            &fixture.root,
            terminal.receipt.clone(),
        )
    })?;
    Ok(terminal.receipt)
}

#[test]
#[ignore = "Allocated actual retaining jobs CLI: one short subtitle conversion and original owned EOF"]
fn actual_retaining_jobs_short_conversion_and_owned_eof() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("retaining-jobs");
    fixture.keep_evidence();
    let deadline = original(&fixture)?;
    let (host, endpoint) = simple_ready(&fixture, false, deadline)?;
    let action = close_request(catch_unwind(AssertUnwindSafe(
        || -> Result<(), Box<CliFailure>> {
            let source = fixture.root.join("original.srt");
            let output = fixture.root.join("converted.vtt");
            let original = "1\n00:00:01,001 --> 00:00:03,456\n안녕하세요 日本語\n\n";
            fs::write(&source, original).unwrap();
            lease(&fixture.root.join("store/jobs-owner.lock"), true).unwrap();
            let mut submit = request(
                "jobs.submit",
                json!({"operation":"subtitles.convert",
            "parameters":{"input_path":source,"output_path":output,"input_format":"srt","output_format":"vtt"},
            "execution_timeout_ms":"10000"}),
            );
            submit.context.idempotency_key = Some("retaining-one".into());
            let admitted =
                fixture.try_call_until("submit", &submit, &endpoint, deadline, REQUEST, 0)?;
            assert_eq!(admitted["data"]["created"], true);
            let id: JobId =
                serde_json::from_value(admitted["data"]["job"]["job_id"].clone()).unwrap();
            let completed = try_terminal_until(
                &endpoint,
                &id,
                deadline,
                Duration::from_secs(15),
                &fixture.root,
            )?;
            assert_eq!(completed.state, JobState::Completed);
            let verified = completed.output.as_ref().unwrap();
            assert_eq!(
                fs::canonicalize(&verified.path).unwrap(),
                output.canonicalize().unwrap()
            );
            assert!(
                fs::read_to_string(&output)
                    .unwrap()
                    .contains("안녕하세요 日本語")
            );
            assert_eq!(fs::read_to_string(&source).unwrap(), original);
            let actual = kinekino_player::infrastructure::content_digest::sha256(
                &mut fs::File::open(&output).unwrap(),
                1024 * 1024,
            )
            .unwrap();
            assert_eq!(actual.0, verified.bytes.0);
            assert_eq!(actual.1, verified.sha256);
            Ok(())
        },
    )));
    let stopped = host
        .finish(
            STOP,
            &fixture.root.join("logs/stop.stdout"),
            &fixture.root.join("logs/host.stderr"),
        )
        .and_then(|terminal| finish_simple(&fixture, terminal, false));
    ScenarioFailure::conclude(action, stopped, None)
}

#[test]
#[ignore = "Allocated retaining provider ready/EOF only; actual browser startup, no catalog request"]
fn actual_retaining_provider_empty_ready_and_owned_eof() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("retaining-provider");
    fixture.keep_evidence();
    let deadline = original(&fixture)?;
    let (host, _) = simple_ready(&fixture, true, deadline)?;
    let action = close_request(catch_unwind(AssertUnwindSafe(
        || -> Result<(), Box<CliFailure>> {
            lease(&fixture.root.join("store/jobs-owner.lock"), true).unwrap();
            lease(&fixture.root.join("profile/provider-owner.lock"), true).unwrap();
            Ok(())
        },
    )));
    let stopped = host
        .finish(
            STOP,
            &fixture.root.join("logs/stop.stdout"),
            &fixture.root.join("logs/host.stderr"),
        )
        .and_then(|terminal| finish_simple(&fixture, terminal, true));
    ScenarioFailure::conclude(action, stopped, None)
}

#[test]
#[ignore = "Allocated retaining playback ready/EOF only; pinned mpv validation and browser startup, no open"]
fn actual_retaining_playback_empty_ready_and_owned_eof() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("retaining-playback");
    fixture.keep_evidence();
    let deadline = original(&fixture)?;
    let input = (|| -> io::Result<_> {
        let catalog = std::env::var_os("NATIVE_EXPORT_CATALOG_ROOT")
            .ok_or_else(|| error("Explicit catalog root required"))?;
        let mpv = std::env::var_os("NATIVE_MPV_EXE")
            .ok_or_else(|| error("Explicit mpv executable required"))?;
        let sha = std::env::var("NATIVE_MPV_SHA256").map_err(io::Error::other)?;
        Ok((catalog, mpv, sha))
    })()
    .map_err(|error| {
        ScenarioFailure::host(CliFailure::not_spawned(
            "consumer playback inputs",
            error,
            &fixture.root,
        ))
    })?;
    let host = BoundedNativeOwner::try_start(
        &fixture,
        Path::new(&input.0),
        Path::new(&input.1),
        &input.2,
        deadline,
        READY,
    )
    .map_err(ScenarioFailure::host)?;
    let action = close_request(catch_unwind(AssertUnwindSafe(
        || -> Result<(), Box<CliFailure>> {
            lease(&fixture.root.join("transition-jobs/jobs-owner.lock"), true).unwrap();
            lease(
                &fixture.root.join("transition-profile/provider-owner.lock"),
                true,
            )
            .unwrap();
            Ok(())
        },
    )));
    let stopped = host.try_stop_chain(0, STOP).and_then(|receipt| {
        let checked = (|| -> io::Result<()> {
            physical(&fixture, &receipt)?;
            let stopped: Value =
                serde_json::from_slice(&fs::read(fixture.root.join("logs/native-stop.stdout"))?)?;
            if stopped["receipt"]["native"].get("Ok") != Some(&Value::Null) {
                return Err(error("No-open consumer unexpectedly reported active media"));
            }
            lease(&fixture.root.join("transition-jobs/jobs-owner.lock"), false)?;
            lease(
                &fixture.root.join("transition-profile/provider-owner.lock"),
                false,
            )
        })();
        checked.map_err(|error| {
            CliFailure::terminal(
                "retaining no-open EOF",
                error,
                &fixture.root,
                receipt.clone(),
            )
        })?;
        Ok(receipt)
    });
    ScenarioFailure::conclude(action, stopped, None)
}
