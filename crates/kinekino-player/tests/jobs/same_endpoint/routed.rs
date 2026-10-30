//! Narrow original-GUI endpoint caller. GUI/window lifecycle remains with its original owner.
#[path = "routed/observation.rs"]
mod observation;
#[path = "routed/owner.rs"]
pub(crate) mod owner;
use super::{ChainBudgets, EndpointInput, run_category};
use crate::{
    fixture::Fixture,
    kinekino_process::{CliFailure, OwnedHost, phase_deadline},
};
use kinekino_player::{
    application::ui_observation::{FINAL_MAX_BYTES, NATIVE_READY_KIND, VIEWER_OUTCOME_KIND},
    domain::{identifiers::valid_sha256, local_endpoint::LocalEndpoint},
    infrastructure::catalog::CatalogFiles,
};
use kinekino_process::domain::identity::ProcessIdentity;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OwnerManifest {
    pub schema_version: u32,
    pub run_id: String,
    pub exclusive_control: bool,
    pub record_validation: bool,
    pub gui_pid: u32,
    pub gui_creation_filetime: u64,
    pub gui_executable: PathBuf,
    pub gui_sha256: String,
    pub endpoint: String,
    pub job_root: PathBuf,
    pub catalog_root: PathBuf,
    pub browser_profile_root: PathBuf,
    pub mpv_executable: PathBuf,
    pub mpv_sha256: String,
}
fn validate_owner(
    manifest: &OwnerManifest,
    identity: &ProcessIdentity,
    selected_sha256: &str,
) -> io::Result<()> {
    let endpoint = LocalEndpoint::parse(&manifest.endpoint).map_err(io::Error::other)?;
    if manifest.schema_version != 1
        || !manifest.exclusive_control
        || !manifest.record_validation
        || !observation::identity_text(&manifest.run_id)
        || !manifest.gui_executable.is_absolute()
        || !manifest.catalog_root.is_absolute()
        || !manifest.job_root.is_absolute()
        || !manifest.browser_profile_root.is_absolute()
        || !manifest.mpv_executable.is_absolute()
        || !valid_sha256(&manifest.gui_sha256)
        || !valid_sha256(&manifest.mpv_sha256)
        || !valid_sha256(selected_sha256)
        || manifest.gui_sha256 != selected_sha256
        || identity.pid != manifest.gui_pid
        || endpoint.process_id != manifest.gui_pid
        || identity.creation_filetime != Some(manifest.gui_creation_filetime)
        || endpoint.process_start != manifest.gui_creation_filetime
        || identity.executable.canonicalize()? != manifest.gui_executable.canonicalize()?
    {
        return Err(io::Error::other(
            "Original GUI owner, selected pin and endpoint manifest disagree",
        ));
    }
    Ok(())
}

/// Caller retains the same original owner; no host is started by this borrowed-endpoint adapter.
pub(crate) fn run_selected_categories(
    fixture: &Fixture,
    gui: &mut OwnedHost,
    manifest: &OwnerManifest,
    selected_gui_sha256: &str,
    ready_cap: Duration,
    budgets: &ChainBudgets,
    categories: &[&str],
) -> Result<Vec<Value>, Box<CliFailure>> {
    run_selected_scenario(
        fixture,
        gui,
        manifest,
        selected_gui_sha256,
        ready_cap,
        budgets,
        ScenarioCategories {
            categories,
            adjacent: false,
        },
    )
}
pub(crate) fn run_selected_categories_with_adjacent(
    fixture: &Fixture,
    gui: &mut OwnedHost,
    manifest: &OwnerManifest,
    selected_gui_sha256: &str,
    ready_cap: Duration,
    budgets: &ChainBudgets,
    categories: &[&str],
) -> Result<Vec<Value>, Box<CliFailure>> {
    run_selected_scenario(
        fixture,
        gui,
        manifest,
        selected_gui_sha256,
        ready_cap,
        budgets,
        ScenarioCategories {
            categories,
            adjacent: true,
        },
    )
}
struct ScenarioCategories<'a, 'b> {
    categories: &'a [&'b str],
    adjacent: bool,
}
fn run_selected_scenario(
    fixture: &Fixture,
    gui: &mut OwnedHost,
    manifest: &OwnerManifest,
    selected_gui_sha256: &str,
    ready_cap: Duration,
    budgets: &ChainBudgets,
    selection: ScenarioCategories<'_, '_>,
) -> Result<Vec<Value>, Box<CliFailure>> {
    let ScenarioCategories {
        categories,
        adjacent,
    } = selection;
    let failure = |stage, error| CliFailure::not_spawned(stage, error, &fixture.root);
    validate_owner(manifest, gui.original_identity(), selected_gui_sha256)
        .map_err(|error| failure("routed owner manifest", error))?;
    let original = gui.deadline();
    let ready_deadline = phase_deadline(original, ready_cap)
        .map_err(|error| failure("routed ready cutoff", error))?;
    let mut ready_seen = false;
    for index in 0..64 {
        let bytes = gui
            .next_observation_record(FINAL_MAX_BYTES + 1, ready_deadline)
            .map_err(|error| failure("routed ready original record", error))?;
        fs::write(
            fixture
                .root
                .join(format!("logs/routed-ready-record-{index}.jsonl")),
            &bytes,
        )
        .map_err(|error| failure("routed ready raw evidence", error))?;
        // libtest owns banner/result lines. Preserve them without treating them as Ready.
        if !bytes.starts_with(b"{") {
            continue;
        }
        let kind = observation::event_kind(&bytes)
            .map_err(|error| failure("routed ready envelope", error))?;
        if kind == VIEWER_OUTCOME_KIND {
            return Err(failure(
                "routed ready",
                io::Error::other("Original GUI returned before activation was observed"),
            ));
        }
        if kind == NATIVE_READY_KIND {
            let observed = observation::ready(&bytes)
                .map_err(|error| failure("routed ready scalar schema", error))?;
            fs::write(
                fixture.root.join("routed-ready.json"),
                serde_json::to_vec_pretty(&observed)
                    .map_err(|error| failure("routed ready encoding", io::Error::other(error)))?,
            )
            .map_err(|error| failure("routed ready evidence", error))?;
            ready_seen = true;
            break;
        }
    }
    if !ready_seen || ready_deadline.operation_remaining(Instant::now()).is_zero() {
        return Err(failure(
            "routed ready",
            io::Error::other("Original routed activation remains unobserved"),
        ));
    }
    validate_owner(manifest, gui.original_identity(), selected_gui_sha256)
        .map_err(|error| failure("routed owner before commands", error))?;
    let files = CatalogFiles::open(&manifest.catalog_root).map_err(|error| {
        failure(
            "routed catalog evidence reader",
            io::Error::other(format!("{error:?}")),
        )
    })?;
    let endpoint = EndpointInput {
        endpoint: &manifest.endpoint,
        original,
    };
    let mut observations: Vec<Value> = Vec::new();
    for name in categories.iter().copied() {
        let observed = if adjacent {
            super::chain::run_category_with_adjacent(fixture, &endpoint, &files, name, budgets)?
        } else {
            run_category(fixture, &endpoint, &files, name, budgets)?
        };
        if observations
            .iter()
            .any(|prior| prior["engine_instance_id"] == observed["engine_instance_id"])
        {
            return Err(failure(
                "routed engine identity",
                io::Error::other("Closed native instance was reused"),
            ));
        }
        observations.push(observed);
        fs::write(
            fixture.root.join("categories.json"),
            serde_json::to_vec_pretty(&observations)
                .map_err(|error| failure("routed categories encoding", io::Error::other(error)))?,
        )
        .map_err(|error| failure("routed categories evidence", error))?;
    }
    Ok(observations)
}

#[cfg(test)]
#[path = "routed/tests.rs"]
mod tests;
