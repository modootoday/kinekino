#![cfg(all(windows, feature = "native-proposal-window"))]
//! Explicit actual player.exe slots. These do not certify visual fidelity or model quality.
#[path = "native_product_launch/default_launch.rs"]
mod default_launch;
#[path = "native_product_launch/fixture.rs"]
mod fixture;
#[path = "native_product_launch/process.rs"]
mod process;
use fixture::*;
use serde_json::json;
use std::{fs, path::PathBuf};

#[test]
#[ignore = "Requires exact OS lease and a separately pinned live retained-owner control host"]
fn actual_default_noargs_native_window_closes_normally() -> Result<(), String> {
    kinekino_platform::install();
    default_launch::run_gate(default_launch::Mode::NoArgs)
}

#[test]
#[ignore = "Requires exact OS/LocalHost lease and a separately pinned live retained-owner control host"]
fn actual_default_configured_sdk_endpoint_and_frame_close_normally() -> Result<(), String> {
    kinekino_platform::install();
    default_launch::run_gate(default_launch::Mode::Configured)
}

#[test]
#[ignore = "Explicit SDK-owned actual Native process slot; closes only this fixture's windows"]
fn actual_player_native_three_tasks_and_document_repair_close_with_joined_sdk() {
    kinekino_platform::install();
    let root = evidence_root("actual");
    let binary = pin_binary(&root, &player_exe());
    for kind in [
        Kind::Cards,
        Kind::Highlights,
        Kind::CueCorrection,
        Kind::DocumentCorrection,
        Kind::RepairUnselected,
        Kind::RepairSelected,
    ] {
        let case = Fixture::new(&root, kind);
        let run = process::run(&binary, &case.root, &case.request_path, &case.store, true);
        assert_eq!(run.exit, Some(0), "{}: {:?}", case.name, run.events);
        assert_eq!(run.closed_windows, 1, "single owned visible Native window");
        let outcome = run
            .events
            .iter()
            .find(|event| event["kind"] == "native-proposal-outcome")
            .unwrap();
        assert_eq!(outcome["data"]["success"], true);
        assert_eq!(
            outcome["data"]["cleanup"],
            "SdkQuiescent { native: Closed }"
        );
        assert_eq!(outcome["data"]["original"]["result"], "Ok(())");
        assert!(
            run.events
                .iter()
                .any(|event| event["kind"] == "native-proposal-window"
                    && event["data"]["kind"] == "FrameSubmitted")
        );
        let assets: Vec<_> = run
            .events
            .iter()
            .filter(|event| event["kind"] == "native-proposal-assets")
            .collect();
        assert!(
            !assets.is_empty(),
            "actual renderer asset observation required"
        );
        assert!(
            assets
                .iter()
                .all(|event| event["data"]["rejected"] == 0
                    && event["data"]["failures"] == json!([]))
        );
        case.assert_unchanged();
    }
    fs::write(root.join("final-binary-sha256.txt"), binary_sha(&binary)).unwrap();
}

#[test]
#[ignore = "Explicit newly pinned on/off player binaries; no GUI should be created"]
fn actual_player_native_invalid_inputs_and_feature_off_fail_before_execution() {
    kinekino_platform::install();
    let root = evidence_root("invalid");
    let binary = pin_binary(&root, &player_exe());
    let case = Fixture::new(&root, Kind::Cards);
    let valid = fs::read(&case.request_path).unwrap();
    for (label, bytes) in [
        ("malformed", b"{".to_vec()),
        ("oversize", vec![b' '; 65537]),
    ] {
        let directory = root.join(label);
        fs::create_dir(&directory).unwrap();
        let input = directory.join("request.json");
        fs::write(&input, bytes).unwrap();
        let run = process::run(&binary, &directory, &input, &case.store, false);
        assert_ne!(run.exit, Some(0));
        assert!(
            !run.events
                .iter()
                .any(|event| event["kind"] == "native-proposal-execution-started")
        );
        assert_eq!(run.closed_windows, 0);
    }
    for (label, field, replacement) in [
        (
            "sdk-initial-error",
            "context",
            json!({"expected_revision":"1"}),
        ),
        (
            "stale-selection",
            "selection",
            json!({"document_revision":"0"}),
        ),
    ] {
        let directory = root.join(label);
        fs::create_dir(&directory).unwrap();
        let mut request: serde_json::Value = serde_json::from_slice(&valid).unwrap();
        for (key, value) in replacement.as_object().unwrap() {
            request[field][key] = value.clone();
        }
        let input = directory.join("request.json");
        fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();
        let run = process::run(&binary, &directory, &input, &case.store, false);
        assert_ne!(run.exit, Some(0));
        let outcome = run
            .events
            .iter()
            .find(|event| event["kind"] == "native-proposal-outcome")
            .unwrap();
        assert_eq!(outcome["data"]["success"], false);
        assert_eq!(
            outcome["data"]["cleanup"],
            "SdkQuiescent { native: NoDocument }"
        );
        assert_eq!(outcome["data"]["accepted_initial"], true);
    }
    let off = PathBuf::from(
        std::env::var_os("NATIVE_PRODUCT_OFF_EXE").expect("explicit off binary path required"),
    );
    let directory = root.join("feature-off");
    fs::create_dir(&directory).unwrap();
    let input = directory.join("request.json");
    fs::write(&input, valid).unwrap();
    fs::write(directory.join("binary-sha256.txt"), binary_sha(&off)).unwrap();
    let run = process::run(&off, &directory, &input, &case.store, false);
    assert_ne!(run.exit, Some(0));
    assert!(
        !run.events
            .iter()
            .any(|event| event["kind"] == "native-proposal-execution-started")
    );
    assert!(
        fs::read_to_string(directory.join("stderr.log"))
            .unwrap()
            .contains("unavailable in this build")
    );
    case.assert_unchanged();
}

#[test]
#[ignore = "Explicit trusted Native factory-failure slot; no replacement loop or OS view"]
fn trusted_mount_failure_returns_original_factory_cause_and_joined_owner() {
    kinekino_platform::install();
    use kinekino_player::{
        assembly::meari::product_native_launch::run_product_native,
        infrastructure::meari::product_styles::prepare_product_styles,
    };
    let root = evidence_root("mount-failure");
    let case = Fixture::new(&root, Kind::Cards);
    let request = serde_json::from_slice(&fs::read(&case.request_path).unwrap()).unwrap();
    let mut styles = prepare_product_styles().unwrap();
    styles
        .css
        .push_str("\n.probe{background-image:url('https://invalid.example/disallowed.svg')}");
    let outcome = run_product_native(case.client(), request, styles);
    let report = outcome.report();
    fs::write(
        root.join("outcome.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(!outcome.succeeded());
    assert_eq!(report["cleanup"], "SdkQuiescent { native: NoDocument }");
    assert_eq!(report["original"]["result"], "Err(Consumer(Mount))");
    assert!(!report["original"]["mount_cause"].is_null());
    case.assert_unchanged();
}

#[test]
#[ignore = "Explicit trusted deferred Native attach-failure slot; teardown remains unconfirmed"]
fn trusted_deferred_attach_retains_original_runtime_and_recovery_through_product_cleanup() {
    kinekino_platform::install();
    use kinekino_player::{
        application::sdk_execution::ExecutionBudget,
        assembly::meari::{
            product_native_cleanup::finish_owned_product,
            proposal_window_startup::prepare_proposal_execution,
        },
        infrastructure::meari::product_styles::prepare_product_styles,
        presentation::web_ui_adapter::proposals::{product_window::*, window_cleanup::*},
    };
    use meari_native::{
        domain::Viewport,
        infrastructure::{
            native_host::HostScheduleLimits,
            window::{NativeAttachProblem, NativeRunError, NativeStartupError},
        },
    };
    use std::{rc::Rc, time::Duration};
    let root = evidence_root("deferred-attach");
    let case = Fixture::new(&root, Kind::Cards);
    let limits =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(100), 100).unwrap();
    let (owner, startup, execution) = prepare_proposal_execution(
        case.client(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        limits,
    )
    .unwrap()
    .into_parts();
    let styles = prepare_product_styles().unwrap();
    let request = serde_json::from_slice(&fs::read(&case.request_path).unwrap()).unwrap();
    let controller = prepare_product_window(
        case.client(),
        execution,
        request,
        ProductWindowConfig {
            labels: ProductWindowLabels::korean(),
            visuals: ProductWindowVisuals {
                css: styles.css,
                assets: Some(styles.assets),
            },
            viewport: Viewport {
                width: 1180,
                height: 760,
                dark: true,
            },
            limits: HostScheduleLimits::new(
                Duration::from_millis(3),
                Duration::from_millis(100),
                100,
            )
            .unwrap(),
            worker_exit_delay: Duration::from_millis(5),
            observer: Box::new(|_| Ok(())),
        },
    )
    .ok()
    .expect("trusted product controller");
    let returned = owner.run_startup(startup, controller);
    assert!(matches!(
        returned.result,
        Err(NativeRunError::Startup(NativeStartupError::Attach(
            NativeAttachProblem::Host(_)
        )))
    ));
    assert!(returned.controller.attach_failure.is_none());
    assert!(returned.recovery.is_some());
    let runtime = returned.runtime.clone().expect("original runtime");
    let original_result = format!("{:?}", returned.result);
    let cleanup = finish_owned_product(ProductWindowOwners::Returned(Box::new(returned)));
    assert_eq!(
        cleanup.status,
        CleanupStatus::SdkQuiescent {
            native: NativeCleanupState::Unconfirmed
        }
    );
    let ProductWindowOwners::Returned(returned) = &cleanup.owners else {
        panic!("same returned owner")
    };
    assert!(Rc::ptr_eq(&runtime, returned.runtime.as_ref().unwrap()));
    assert!(Rc::ptr_eq(
        &runtime,
        returned.controller.runtime.as_ref().unwrap()
    ));
    assert!(returned.recovery.is_some());
    assert_eq!(format!("{:?}", returned.result), original_result);
    fs::write(root.join("outcome.json"),json!({"original_result":original_result,"runtime_identity_preserved":true,"recovery_retained":returned.recovery.is_some(),"cleanup":format!("{:?}",cleanup.status),"sdk":format!("{:?}",cleanup.evidence.last_sdk),"normal_os_exit":false}).to_string()).unwrap();
    case.assert_unchanged();
}

/// `player` is a `kinekino-app` bin since ADR p1-port-inversion; `CARGO_BIN_EXE_*` covers only
/// this package, so resolve `<profile>/player.exe` next to the test runner (`PLAYER_EXE` overrides).
/// Build first: `cargo build -p kinekino-app --features native-proposal-window --bin player`.
fn player_exe() -> std::path::PathBuf {
    if let Some(path) = std::env::var_os("PLAYER_EXE") {
        return path.into();
    }
    let runner = std::env::current_exe().unwrap();
    let profile = runner.parent().and_then(std::path::Path::parent).unwrap();
    profile.join(format!("player{}", std::env::consts::EXE_SUFFIX))
}
