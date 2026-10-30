//! Same fixed lib-test executable, original production assembly; no simulated Ready.
use super::Scope;
use crate::{
    application::{api::PlayerSdk, launch::browser_provider::BrowserProviderConfiguration},
    assembly::meari::product_viewer_launch::{
        self,
        routed::{ProductPlaybackConfiguration, prepare_product_playback},
    },
    infrastructure::{
        self,
        local_host::{LocalHost, TransportDiagnostics, TransportOptions},
    },
};
use serde::Deserialize;
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

pub(super) const EXACT: &str =
    "assembly::meari::routed_owner_fixture::child::original_product_gui_owner";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    schema_version: u32,
    job_root: PathBuf,
    catalog_root: PathBuf,
    browser_profile_root: PathBuf,
    mpv_executable: PathBuf,
    mpv_sha256: String,
    close_marker: PathBuf,
    nonce: String,
    lifetime_ms: u64,
}
fn configuration() -> io::Result<Configuration> {
    let path = PathBuf::from(
        std::env::var_os("ROUTED_GUI_FIXTURE_CONFIGURATION")
            .ok_or_else(|| io::Error::other("Explicit fixture configuration is required"))?,
    );
    if !path.is_absolute() {
        return Err(io::Error::other("Configuration path must be absolute"));
    }
    let mut bytes = Vec::new();
    File::open(path)?.take(16385).read_to_end(&mut bytes)?;
    if bytes.len() > 16384 {
        return Err(io::Error::other("Fixture configuration exceeds limit"));
    }
    let configuration: Configuration = serde_json::from_slice(&bytes)?;
    if configuration.schema_version != 1
        || configuration.lifetime_ms == 0
        || !configuration.job_root.is_absolute()
        || !configuration.catalog_root.is_absolute()
        || !configuration.browser_profile_root.is_absolute()
        || !configuration.mpv_executable.is_absolute()
        || !crate::domain::identifiers::valid_sha256(&configuration.mpv_sha256)
    {
        return Err(io::Error::other("Invalid explicit fixture configuration"));
    }
    Ok(configuration)
}

#[test]
#[ignore = "Actual product GUI/media owner; central OS/browser/mpv allocation and explicit original-supervisor configuration required"]
fn original_product_gui_owner() -> io::Result<()> {
    let configuration = configuration()?;
    let started = Instant::now();
    let deadline = started
        .checked_add(Duration::from_millis(configuration.lifetime_ms))
        .ok_or_else(|| io::Error::other("Fixture deadline overflow"))?;
    let trace_path = configuration
        .close_marker
        .with_file_name("routed-local-transport-trace.json");
    let mut scope = Scope::enter(configuration.close_marker, &configuration.nonce, deadline)?;
    infrastructure::diagnostics::configure(true);
    let provider = BrowserProviderConfiguration {
        job_root: configuration.job_root,
        catalog_root: configuration.catalog_root,
        profile_root: configuration.browser_profile_root,
    };
    // This is the one production configured SDK. The same client starts LocalHost and execution.
    let playback = match prepare_product_playback(
        PlayerSdk::new(infrastructure::subtitle_files::SubtitleFiles),
        ProductPlaybackConfiguration {
            provider: &provider,
            executable: &configuration.mpv_executable,
            sha256: &configuration.mpv_sha256,
        },
    ) {
        Ok(playback) => playback,
        Err(failure) => {
            eprintln!("{}", failure.report());
            // Preserve failed original actors through this abnormal process boundary.
            std::process::exit(1);
        }
    };
    let transport_trace = TransportDiagnostics::default();
    let trace_started_elapsed_us = started.elapsed().as_micros();
    let transport_options = TransportOptions::default();
    let endpoint = match LocalHost::start_observed(
        playback.client(),
        transport_options,
        transport_trace.clone(),
    ) {
        Ok(endpoint) => endpoint,
        Err(error) => {
            let failure = playback.abort_setup(error);
            eprintln!("{}", failure.report());
            std::process::exit(1);
        }
    };
    if let Some(error) = product_viewer_launch::write_viewer_event(
        "local-host-ready",
        &serde_json::json!({"endpoint":endpoint.endpoint(),
            "is_running":endpoint.is_running(),"fixture":EXACT}),
        true,
        started.elapsed().as_millis(),
        "native-viewer-initialization",
        &mut io::stdout().lock(),
    ) {
        eprintln!("Initial diagnostic failed: {error}");
        std::process::exit(1);
    }
    // Production frame consumer emits its own actual Ready. This fixture emits none.
    let outcome = playback.run(endpoint, None);
    let close = scope.finish();
    let close_valid =
        close.requested && matches!(close.delivered, Some(Ok(_))) && close.watcher.is_ok();
    let trace_write = (|| -> io::Result<()> {
        let bytes = serde_json::to_vec(
            &serde_json::json!({"schema_version":1,"trace_started_fixture_elapsed_us":trace_started_elapsed_us,"observed_fixture_elapsed_us":started.elapsed().as_micros(),"server_timeout_ms":transport_options.timeout.as_millis(),"truncated":transport_trace.truncated(),"events":transport_trace.snapshot(),"native_owner_succeeded":outcome.succeeded()}),
        )?;
        if bytes.len() > 256 * 1024 {
            return Err(io::Error::other(
                "Bounded transport diagnostic exceeds byte limit",
            ));
        }
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(trace_path)?
            .write_all(&bytes)
    })();
    let succeeded = outcome.succeeded() && close_valid && trace_write.is_ok();
    let diagnostic = outcome.write_report(
        true,
        started.elapsed().as_millis(),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    );
    eprintln!(
        "{}",
        serde_json::json!({"kind":"fixture-original-close",
        "requested":close.requested,"delivered":format!("{:?}",close.delivered),
        "watcher_error":close.watcher.err().map(|error|error.to_string()),
        "normal_owner_result":outcome.succeeded(),"fixture_success":succeeded,
        "transport_trace_error":trace_write.err().map(|error|format!("{:?}",error.kind()))})
    );
    if !succeeded || diagnostic.stdout.is_some() || diagnostic.stderr.is_some() {
        std::process::exit(1);
    }
    match outcome.end_review() {
        Ok(()) => Ok(()),
        Err(retained) => {
            eprintln!("{}", retained.report());
            std::process::exit(1);
        }
    }
}
