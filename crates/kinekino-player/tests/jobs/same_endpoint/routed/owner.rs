//! Parent adapter over the existing OwnedHost; normal-close intent precedes its explicit finish.
use super::super::ChainBudgets;
use super::{OwnerManifest, observation, run_selected_categories};
use crate::{
    fixture::Fixture,
    kinekino_process::{OwnedHost, ScenarioFailure},
};
use kinekino_player::application::catalog::Category;
use kinekino_player::application::ui_observation::{FINAL_MAX_BYTES, VIEWER_OUTCOME_KIND};
use std::{
    fs,
    io::{self, Read, Write},
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
    time::Duration,
};

struct FixtureBudgets<'a> {
    ready_cap: Duration,
    stop_cap: Duration,
    budgets: &'a ChainBudgets,
}
struct LaunchWindow<'a> {
    original: kinekino_process::domain::deadline::Deadline,
    phases: FixtureBudgets<'a>,
}
struct ScenarioChoice {
    category: Category,
    adjacent: bool,
}
struct OriginalCloseRequest<'a> {
    close_marker: &'a Path,
    nonce: &'a str,
}
#[test]
#[ignore = "Actual product GUI owner plus live provider/mpv; explicit fixed libtest/CLI pins, budgets and central OS/media allocation required"]
fn actual_first_category_through_original_product_gui_owner() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    actual_scenario(Category::Airing, false)
}
#[test]
#[ignore = "Actual original routed GUI adjacent replacement; central browser/mpv slot and original artifacts required"]
fn actual_adjacent_roundtrip_through_original_product_gui_owner() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    actual_scenario(Category::Airing, true)
}
#[test]
#[ignore = "Fresh original GUI owner for completed category; explicit artifact/input/media allocation required"]
fn actual_completed_category_through_original_product_gui_owner() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    actual_scenario(Category::Completed, false)
}
#[test]
#[ignore = "Fresh original GUI owner for movies category; no fabricated episodes or alternate SDK allowed"]
fn actual_movies_category_through_original_product_gui_owner() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    actual_scenario(Category::Movies, false)
}
fn actual_scenario(category: Category, adjacent: bool) -> Result<(), ScenarioFailure> {
    let required =
        |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("Explicit {name} required"));
    let duration = |name| {
        let value: u64 = required(name)
            .parse()
            .expect("Explicit decimal milliseconds");
        assert!(value > 0);
        Duration::from_millis(value)
    };
    let executable = std::path::PathBuf::from(required("ROUTED_GUI_FIXTURE_EXE"));
    let pin = required("ROUTED_GUI_FIXTURE_SHA256");
    let manifest_path = std::path::PathBuf::from(required("ROUTED_GUI_OWNER_MANIFEST"));
    assert!(manifest_path.is_absolute());
    let mut manifest_bytes = Vec::new();
    fs::File::open(manifest_path)
        .expect("Explicit owner manifest")
        .take(16385)
        .read_to_end(&mut manifest_bytes)
        .expect("Bounded owner manifest read");
    assert!(manifest_bytes.len() <= 16384);
    let manifest: OwnerManifest =
        serde_json::from_slice(&manifest_bytes).expect("Owner manifest schema");
    let ready = duration("ROUTED_GUI_READY_MS");
    let stop = duration("ROUTED_GUI_STOP_MS");
    let budgets = ChainBudgets {
        execution_ms: required("ROUTED_GUI_JOB_EXECUTION_MS")
            .parse()
            .expect("Job execution milliseconds"),
        observation: duration("ROUTED_GUI_JOB_OBSERVATION_MS"),
        request: duration("ROUTED_GUI_REQUEST_MS"),
    };
    assert!(budgets.execution_ms > 0);
    let original = kinekino_process::domain::deadline::Deadline::new(
        std::time::Instant::now(),
        duration("ROUTED_GUI_TOTAL_MS"),
        duration("ROUTED_GUI_CLEANUP_MS"),
    )
    .expect("Original fixture deadline");
    let category_wire = serde_json::to_value(category).expect("Original category encoding");
    let category_name = category_wire.as_str().expect("Original category string");
    let scope = if adjacent { "adjacent" } else { "first" };
    let mut fixture = Fixture::named(&format!("routed-gui-{category_name}-{scope}"));
    fixture.keep_evidence();
    fs::write(fixture.root.join("scenario-input.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "category":category,"adjacent_roundtrip":adjacent,"expected_success_jobs":if adjacent {5} else {3},
        "ownership":"fresh original GUI owner for this category only","audio":"explicit muted functional fixture"
    })).expect("Scenario encoding")).expect("Scenario input evidence");
    let environment = std::env::vars_os()
        .map(|(name, value)| {
            let name = name
                .into_string()
                .expect("Explicit ASCII environment names");
            assert!(name.is_ascii() && !name.is_empty() && !name.contains('='));
            (name, value)
        })
        .collect();
    launch_scenario(
        &fixture,
        executable,
        &pin,
        manifest,
        environment,
        LaunchWindow {
            original,
            phases: FixtureBudgets {
                ready_cap: ready,
                stop_cap: stop,
                budgets: &budgets,
            },
        },
        ScenarioChoice { category, adjacent },
    )
}

fn launch_scenario(
    fixture: &Fixture,
    executable: std::path::PathBuf,
    selected_pin: &str,
    mut manifest: OwnerManifest,
    mut environment: std::collections::BTreeMap<String, std::ffi::OsString>,
    window: LaunchWindow<'_>,
    choice: ScenarioChoice,
) -> Result<(), ScenarioFailure> {
    let LaunchWindow { original, phases } = window;
    let FixtureBudgets {
        ready_cap,
        stop_cap,
        budgets,
    } = phases;
    let ScenarioChoice { category, adjacent } = choice;
    let error = |stage, error| {
        crate::kinekino_process::CliFailure::not_spawned(stage, error, &fixture.root)
    };
    let configuration = fixture.root.join("routed-owner-configuration.json");
    let marker = fixture.root.join("routed-owner-close.request");
    let nonce = manifest.run_id.clone();
    let setup = (|| -> io::Result<()> {
        if !configuration.is_absolute()
            || !executable.is_absolute()
            || selected_pin != manifest.gui_sha256
            || manifest.gui_executable != executable
        {
            return Err(io::Error::other(
                "Fixed original executable and configuration disagree",
            ));
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&configuration)?;
        serde_json::to_writer(
            &mut file,
            &serde_json::json!({
                "schema_version":1,"job_root":manifest.job_root,"catalog_root":manifest.catalog_root,
                "browser_profile_root":manifest.browser_profile_root,"mpv_executable":manifest.mpv_executable,
                "mpv_sha256":manifest.mpv_sha256,"close_marker":marker,"nonce":nonce,
                "lifetime_ms":u64::try_from(original.operation_remaining(std::time::Instant::now()).as_millis())
                    .map_err(io::Error::other)?,
            }),
        )?;
        file.flush()
    })();
    setup.map_err(|cause| ScenarioFailure::host(error("routed fixture configuration", cause)))?;
    environment.insert(
        "ROUTED_GUI_FIXTURE_CONFIGURATION".into(),
        configuration.into_os_string(),
    );
    let arguments = [
        "--exact",
        "assembly::meari::routed_owner_fixture::child::original_product_gui_owner",
        "--ignored",
        "--nocapture",
        "--quiet",
        "--test-threads=1",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    let mut gui = OwnedHost::launch_with_environment(
        executable,
        arguments,
        &fixture.root,
        original,
        Some(environment),
    )
    .map_err(ScenarioFailure::host)?;
    let discovered = (|| -> io::Result<()> {
        let cutoff = crate::kinekino_process::phase_deadline(original, ready_cap)?;
        for index in 0..64 {
            let bytes = gui.next_observation_record(FINAL_MAX_BYTES + 1, cutoff)?;
            fs::write(
                fixture
                    .root
                    .join(format!("logs/routed-initial-{index}.jsonl")),
                &bytes,
            )?;
            if !bytes.starts_with(b"{") {
                continue;
            }
            let value: serde_json::Value = serde_json::from_slice(&bytes)?;
            if value["kind"] == "local-host-ready" {
                if value["data"]["is_running"] != true {
                    return Err(io::Error::other("Original LocalHost not running"));
                }
                manifest.endpoint = value["data"]["endpoint"]
                    .as_str()
                    .ok_or_else(|| io::Error::other("Original endpoint missing"))?
                    .into();
                let identity = gui.original_identity();
                manifest.gui_pid = identity.pid;
                manifest.gui_creation_filetime = identity
                    .creation_filetime
                    .ok_or_else(|| io::Error::other("Original creation identity missing"))?;
                super::validate_owner(&manifest, identity, selected_pin)?;
                fs::write(
                    fixture.root.join("routed-original-owner.json"),
                    serde_json::to_vec_pretty(&manifest)?,
                )?;
                return Ok(());
            }
            if value["kind"] == VIEWER_OUTCOME_KIND {
                return Err(io::Error::other("GUI returned before LocalHost"));
            }
        }
        Err(io::Error::other("Original LocalHost event missing"))
    })();
    if let Err(cause) = discovered {
        let request = request_close(&marker, &nonce).err();
        let stopped = gui
            .finish(
                stop_cap,
                &fixture.root.join("logs/routed-early-stop.stdout"),
                &fixture.root.join("logs/routed-early-stop.stderr"),
            )
            .map(|terminal| terminal.receipt);
        return ScenarioFailure::conclude(
            Ok(Err(error("routed original endpoint", cause))),
            stopped,
            request,
        );
    }
    run_owner_scenario(
        fixture,
        gui,
        &manifest,
        selected_pin,
        OriginalCloseRequest {
            close_marker: &marker,
            nonce: &nonce,
        },
        FixtureBudgets {
            ready_cap,
            stop_cap,
            budgets,
        },
        ScenarioChoice { category, adjacent },
    )
}

fn request_close(path: &Path, nonce: &str) -> io::Result<()> {
    if !path.is_absolute()
        || nonce.is_empty()
        || nonce.len() > 128
        || !nonce
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(io::Error::other("Invalid original fixture close request"));
    }
    // Publish complete bytes once; watcher cannot accept a partially written marker.
    let pending = path.with_extension("unpublished");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)?;
    writeln!(file, "{nonce}")?;
    file.flush()?;
    drop(file);
    fs::rename(pending, path)
}

/// Only one first-open category until routed close/reopen has its own actual support evidence.
/// The caller launches the fixed cfg(test) child through launch_with_environment; normal player
/// remains a separately tracked final gate. No PID reattachment or second SDK is permitted.
fn run_owner_scenario(
    fixture: &Fixture,
    mut gui: OwnedHost,
    manifest: &OwnerManifest,
    selected_pin: &str,
    close: OriginalCloseRequest<'_>,
    phases: FixtureBudgets<'_>,
    choice: ScenarioChoice,
) -> Result<(), ScenarioFailure> {
    let OriginalCloseRequest {
        close_marker,
        nonce,
    } = close;
    let FixtureBudgets {
        ready_cap,
        stop_cap,
        budgets,
    } = phases;
    let ScenarioChoice { category, adjacent } = choice;
    let category_wire = serde_json::to_value(category).expect("Original category encoding");
    let category_name = category_wire.as_str().expect("Original category string");
    let action = catch_unwind(AssertUnwindSafe(|| {
        let selected = if adjacent {
            super::run_selected_categories_with_adjacent
        } else {
            run_selected_categories
        };
        selected(
            fixture,
            &mut gui,
            manifest,
            selected_pin,
            ready_cap,
            budgets,
            &[category_name],
        )?;
        Ok(())
    }));
    // Every success/error/panic requests the same original close before supervisor cleanup.
    let mut secondary = request_close(close_marker, nonce).err();
    let mut final_line = None;
    let final_deadline = crate::kinekino_process::phase_deadline(gui.deadline(), stop_cap);
    if let Ok(deadline) = final_deadline {
        for _ in 0..64 {
            match gui.next_observation_record(FINAL_MAX_BYTES + 1, deadline) {
                Ok(bytes) => {
                    if bytes.starts_with(b"{")
                        && observation::event_kind(&bytes).ok().as_deref()
                            == Some(VIEWER_OUTCOME_KIND)
                    {
                        final_line = Some(bytes);
                        break;
                    }
                }
                Err(error) => {
                    if secondary.is_none() {
                        secondary = Some(error);
                    }
                    break;
                }
            }
        }
    } else if secondary.is_none() {
        secondary = final_deadline.err();
    }
    let stopped = gui
        .finish(
            stop_cap,
            &fixture.root.join("logs/routed-stop.stdout"),
            &fixture.root.join("logs/routed-stop.stderr"),
        )
        .and_then(|terminal| {
            let validate = || -> io::Result<()> {
                super::validate_owner(manifest, &terminal.receipt.identity, selected_pin)?;
                if terminal.receipt.forced
                    || terminal.receipt.child_exit != Some(0)
                    || terminal.receipt.family
                        != kinekino_process::domain::observation::FamilyObservation::Empty
                {
                    return Err(io::Error::other("Original fixture did not return normally"));
                }
                let bytes = final_line
                    .as_ref()
                    .ok_or_else(|| io::Error::other("Actual final report missing"))?;
                fs::write(fixture.root.join("routed-final.jsonl"), bytes)?;
                observation::final_report(bytes, if adjacent { 5 } else { 3 })?;
                Ok(())
            };
            validate().map_err(|error| {
                crate::kinekino_process::CliFailure::terminal(
                    "routed final receipts",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            })?;
            Ok(terminal.receipt)
        });
    ScenarioFailure::conclude(action, stopped, secondary)
}
