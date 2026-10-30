#![cfg(windows)]
#[path = "native_gui_endpoint/cli.rs"]
mod cli;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "native_gui_endpoint/executables.rs"]
mod executables;
#[path = "native_gui_endpoint/fixture.rs"]
mod fixture;
#[path = "native_gui_endpoint/native.rs"]
mod native;
#[path = "native_gui_endpoint/scenario.rs"]
mod scenario;

#[test]
#[ignore = "Requires an explicitly leased, test-owned normal GUI + paused Sintel and PLAYER_NATIVE_GUI_FIXTURE JSON"]
fn owned_normal_gui_cli_native_controls_observe_and_restore() {
    kinekino_platform::install();
    let fixture = fixture::Fixture::load().expect("Owned GUI fixture configuration required");
    let owned =
        fixture::Owned::open(&fixture).expect("Owned GUI/source/process verification failed");
    let mut run = cli::Run::new(fixture, owned).expect("Cannot create fresh integration evidence");
    let baseline = run
        .initial()
        .expect("Initial owned paused Sintel observation failed");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        scenario::exercise(&mut run, &baseline)
    }))
    .unwrap_or_else(|_| Err("Unexpected integration panic".into()));
    let restored = scenario::restore(&mut run, &baseline);
    run.finish(&result, &restored)
        .expect("Cannot finalize integration evidence");
    assert!(
        restored.is_ok(),
        "Owned fixture restoration failed; inspect fresh report and notify UI owner"
    );
    assert!(
        result.is_ok(),
        "Native integration failed; inspect fresh report"
    );
}
