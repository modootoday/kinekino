//! Composition root: the only crate that wires adapters to ports. No `cfg` here; each
//! adapter crate registers itself behind its own gates.
//!
//! Flow: `main → entry() → entry_with(Args, Env, kinekino_host::run)`; `bootstrap(Args, Env) -> App`.
mod adblock_cli;
mod args;
pub mod config;
mod env;
pub mod mcp;
pub mod plugins;
mod report;
pub mod settings;
mod watch;

pub use args::Args;
pub use config::{AppConfig, Source};
pub use env::Env;

use kinekino_browser::engine::registry::{self as browser, BrowserRegistry};
use kinekino_core::port::credential::CredentialStore;
use kinekino_core::port::{DisplayServer, ErrorCode, OsId, PortError, PortResult, TargetPlatform};
use std::process::ExitCode;

/// Wired application: selected platform, layered config and port registries.
pub struct App {
    pub target: TargetPlatform,
    pub config: AppConfig,
    pub browser: BrowserRegistry,
    pub credentials: Box<dyn CredentialStore>,
}

/// Builds the application graph. `KINEKINO_PLATFORM=<os>[-<display>]` overrides detection.
pub fn bootstrap(args: &Args, env: &Env) -> PortResult<App> {
    let target = match env.get("KINEKINO_PLATFORM") {
        Some(s) => parse_platform(s)?,
        None => kinekino_platform::session::target(),
    };
    let config = AppConfig::load(args, env)?;
    Ok(App {
        target,
        config,
        browser: browser::registry(),
        credentials: Box::new(kinekino_platform::session::SessionCredentialStore::new()),
    })
}

fn parse_platform(s: &str) -> PortResult<TargetPlatform> {
    let (os, display) = s.split_once('-').map_or((s, None), |(o, d)| (o, Some(d)));
    let os = match OsId::parse(os) {
        OsId::Other => {
            return Err(PortError::new(
                ErrorCode::InvalidInput,
                format!("KINEKINO_PLATFORM: unknown os `{os}`"),
            ));
        }
        os => os,
    };
    let display = match display {
        None => None,
        Some("x11") => Some(DisplayServer::X11),
        Some("wayland") => Some(DisplayServer::Wayland),
        Some("win32") => Some(DisplayServer::Win32),
        Some("quartz") => Some(DisplayServer::Quartz),
        Some(d) => {
            return Err(PortError::new(
                ErrorCode::InvalidInput,
                format!("KINEKINO_PLATFORM: unknown display `{d}`"),
            ));
        }
    };
    Ok(TargetPlatform { os, display })
}

/// Process entry used by the `kinekino` bin. `kinekino platform …` is served here; every
/// other command goes to `legacy` (the SDK runner) unchanged.
pub fn entry() -> ExitCode {
    kinekino_platform::install();
    entry_with(
        Args::from_process(),
        Env::from_process(),
        kinekino_host::run,
    )
}

/// [`entry`] with explicit inputs; `legacy` serves every non-`platform` command.
pub fn entry_with(args: Args, env: Env, legacy: fn() -> u8) -> ExitCode {
    if args.first() == Some("plugins") {
        let (code, out) = run_plugins(&args, &env);
        println!("{out}");
        return ExitCode::from(code);
    }
    let routed = match args.first() {
        Some("settings") => Some(settings::run(&args, &env)),
        Some("watch") => Some(watch::run(&args)),
        Some("adblock") => Some(adblock_cli::run(&args)),
        Some("fonts") => Some(match (args.get(1), args.get(2)) {
            // Pinned Noto UI fonts (OFL-1.1): SHA-256 verified download into the font cache.
            (Some("fetch"), None) => match kinekino_settings_ui::fetch_fonts() {
                Ok(v) => (0, v.to_string()),
                Err(e) => (1, serde_json::json!({"error": e}).to_string()),
            },
            _ => (
                2,
                serde_json::json!({"error": "usage: kinekino fonts fetch"}).to_string(),
            ),
        }),
        Some("mcp") => match mcp::run(&args, &env) {
            Ok(()) => return ExitCode::SUCCESS,
            Err(e) => Some((2, e.to_json())),
        },
        _ => None,
    };
    if let Some((code, out)) = routed {
        println!("{out}");
        return ExitCode::from(code);
    }
    if args.first() != Some("platform") {
        return ExitCode::from(legacy());
    }
    let (code, out) = run_platform(&args, &env);
    println!("{out}");
    ExitCode::from(code)
}

/// `platform capabilities` | `platform config`; JSON on stdout, errors as `PortError` JSON.
pub fn run_platform(args: &Args, env: &Env) -> (u8, String) {
    let result = bootstrap(args, env).and_then(|app| match args.get(1) {
        Some("capabilities") => Ok(report::capabilities(&app)),
        Some("config") => Ok(report::config(&app.config)),
        other => Err(PortError::new(
            ErrorCode::InvalidInput,
            format!("usage: kinekino platform <capabilities|config>, got {other:?}"),
        )),
    });
    match result {
        Ok(json) => (0, json),
        Err(e) => (2, e.to_json()),
    }
}

/// `plugins list`; JSON on stdout, errors as `PortError` JSON.
pub fn run_plugins(args: &Args, env: &Env) -> (u8, String) {
    match args.get(1) {
        Some("list") => (0, plugins::list(env)),
        other => (
            2,
            PortError::new(
                ErrorCode::InvalidInput,
                format!("usage: kinekino plugins list, got {other:?}"),
            )
            .to_json(),
        ),
    }
}

/// Fake `source/1` plugin for end-to-end tests: the SDK fixture plugin behind the SDK's own
/// `serve` loop (plugin side written only against `kinekino-plugin-sdk`).
pub fn fake_source_plugin() {
    let mut plugin = kinekino_plugin_sdk::fixture::FixturePlugin {
        provider: "com.example.fixture".into(),
    };
    let _ = kinekino_plugin_sdk::serve(&mut plugin);
}

/// Desktop `player` process entry (bin moved from `kinekino-player`, ADR p1-port-inversion):
/// installs the OS policy, then runs the host body; plain viewing opens the Meari player
/// window (same as `kinekino watch`). Errors print like an `anyhow` main.
pub fn player() -> ExitCode {
    kinekino_platform::install();
    match kinekino_host::player_gui::main() {
        Ok(kinekino_host::player_gui::Desktop::Done) => ExitCode::SUCCESS,
        Ok(kinekino_host::player_gui::Desktop::Watch { smoke }) => {
            let argv: &[&str] = if smoke {
                &["watch", "--smoke"]
            } else {
                &["watch"]
            };
            let (code, out) = watch::run(&args::Args::new(argv.iter().copied()));
            println!("{out}");
            ExitCode::from(code)
        }
        Err(error) => {
            eprintln!("Error: {error:?}");
            ExitCode::FAILURE
        }
    }
}

/// Entries for fixture/worker bins moved out of domain crates (ADR p1-port-inversion).
/// `kinekino-vad-worker` process: installs the OS policy, then runs the worker body.
pub fn vad_worker() -> ExitCode {
    kinekino_platform::install();
    kinekino_vad_worker::process_entry::main()
}

/// Each installs the OS policy before running, like [`entry`].
pub mod fixtures {
    /// `owned-process-test-child`.
    #[cfg(feature = "test-support")]
    pub fn owned_process_test_child() {
        kinekino_platform::install();
        kinekino_host::fixtures::owned_process_test_child::main();
    }
    /// `source-sdk-fixture-worker`.
    #[cfg(feature = "source-adapter-test-support")]
    pub fn source_sdk_fixture_worker() {
        kinekino_platform::install();
        kinekino_host::fixtures::source_sdk_fixture_worker::main();
    }
    /// `media-source-worker`.
    #[cfg(feature = "worker-ipc")]
    pub fn media_source_worker() {
        kinekino_platform::install();
        kinekino_host::fixtures::media_source_worker::main();
    }
}
