//! Ignored original-window OS observation. No media, routed readiness or visual acceptance.
mod child;
mod fixture;
use crate::test_support::{
    native_window_host::Controller,
    native_window_supervision::{RunFailure, parent_run_for},
};
use meari_native::{
    domain::Viewport,
    infrastructure::{
        document::InteractiveDocument,
        native_host::{HostScheduleLimits, NativeDocumentHandle},
        window::NativeLoopOwner,
    },
};
use std::{
    cell::RefCell,
    io::Write,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};
const CHILD_MODE: &str = "KINEKINO_VISIBLE_CHILD_OWN_WINDOW_CHILD";
const CHILD_EVIDENCE: &str = "KINEKINO_VISIBLE_CHILD_OWN_WINDOW_EVIDENCE";
const EXACT_TEST: &str = "infrastructure::mpv::parent_surface::win32::visible_render_only::actual::original_visible_frame_input_and_cleanup";
const SCHEMA: &str = "visible-render-only-own-window-v1";
#[derive(Default)]
pub(crate) struct Evidence {
    inert_registered: bool,
    fresh_visible_input_focus: bool,
    changed_frame_rejected: bool,
    first_error_retained: bool,
    hide_observed: bool,
    destroy_ack_and_class_release: bool,
    remaining_zero: bool,
    close_seen: bool,
    failure: Option<String>,
    cleanup_error: Option<String>,
    expected_first_error: Option<String>,
    show_frame: Option<String>,
    changed_frame_notice: Option<String>,
    original_window: Option<String>,
    original_child: Option<u64>,
}
fn explain(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}
fn run_inner() -> serde_json::Value {
    let started = Instant::now();
    let deadline = started + Duration::from_secs(15);
    let source = meari::parse(
        "type Props={};export function App(props:Props){return <video />;}",
        meari::application::Language::Tsx,
    )
    .unwrap();
    let document = InteractiveDocument::new(
        source,
        "video{display:block;width:120px;height:60px}",
        Viewport {
            width: 360,
            height: 240,
            dark: false,
        },
    )
    .unwrap();
    let video = document
        .elements()
        .iter()
        .find(|e| e.dom_tag == "video")
        .unwrap()
        .id
        .clone();
    let document = NativeDocumentHandle::new(document, 1).unwrap();
    let mut owner = NativeLoopOwner::new(
        HostScheduleLimits::new(Duration::from_millis(10), Duration::from_secs(1), 1024).unwrap(),
    )
    .unwrap();
    let startup = owner.startup().unwrap();
    let evidence = Rc::new(RefCell::new(Evidence::default()));
    let controller = Controller::<fixture::Fixture> {
        pending: Some((document, video)),
        shared: Rc::new(RefCell::new(None)),
        evidence: evidence.clone(),
        deadline,
        cancel: None,
        timer: None,
        attach_failure: None,
    };
    let mut outcome = owner.run_startup(startup, controller);
    if let Some(cancel) = outcome.controller.cancel.take() {
        let _ = cancel.send(());
    }
    if let Some(timer) = outcome.controller.timer.take() {
        timer.join().unwrap();
    }
    let e = evidence.borrow();
    let record = observation_record(
        &e,
        outcome.result.is_ok(),
        format!("{:?}", outcome.result),
        started.elapsed().as_millis(),
    );
    let diagnostic = write_observation(&record);
    if let Err(error) = &diagnostic {
        eprintln!(
            "observation ledger secondary: {error}; original failure: {:?}",
            e.failure
        );
    }
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    assert!(e.failure.is_none(), "{:?}", e.failure);
    assert!(e.cleanup_error.is_none(), "{:?}", e.cleanup_error);
    assert!(
        e.inert_registered
            && e.fresh_visible_input_focus
            && e.changed_frame_rejected
            && e.first_error_retained
    );
    assert!(e.hide_observed && e.destroy_ack_and_class_release && e.remaining_zero && e.close_seen);
    assert!(
        Instant::now() < deadline,
        "normal visible-child cleanup exceeded original budget"
    );
    assert!(
        diagnostic.is_ok(),
        "observation ledger delivery failed: {diagnostic:?}"
    );
    record
}
fn write_observation(record: &serde_json::Value) -> std::io::Result<()> {
    let root = PathBuf::from(std::env::var_os(CHILD_EVIDENCE).expect("original evidence root"));
    let bytes = serde_json::to_vec_pretty(record)?;
    if bytes.len() > 4096 {
        return Err(std::io::Error::other(
            "observation ledger exceeds original bound",
        ));
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("inner-observation.json"))?;
    file.write_all(&bytes)?;
    file.flush()
}
fn observation_record(
    e: &Evidence,
    native_ok: bool,
    native_result: String,
    elapsed: u128,
) -> serde_json::Value {
    serde_json::json!({"schema":SCHEMA,"test":EXACT_TEST,"native_cleanup_complete":native_ok && e.failure.is_none() && e.cleanup_error.is_none() && e.hide_observed && e.destroy_ack_and_class_release && e.remaining_zero && e.close_seen,
        "inert_registered":e.inert_registered,"fresh_visible_input_focus":e.fresh_visible_input_focus,
        "changed_frame_rejected":e.changed_frame_rejected,"first_error_retained":e.first_error_retained,"hide_observed":e.hide_observed,
        "destroy_ack_and_class_release":e.destroy_ack_and_class_release,"remaining_zero":e.remaining_zero,
        "close_seen":e.close_seen,"expected_first_error":e.expected_first_error,
        "show_frame":e.show_frame,"changed_frame_notice":e.changed_frame_notice,
        "original_window":e.original_window,"original_child":e.original_child,
        "elapsed_ms":elapsed,"inner_budget_ms":15000,
        "failure":e.failure,"cleanup_error":e.cleanup_error,"native_loop_result":native_result,
        "style_trace":format!("{:?}",super::style_trace::style_trace()),
        "observer_fault_injected":false,"media_or_ready_route":false,"visual_acceptance":false})
}
#[test]
#[ignore = "requires separately assigned original-window OS slot; no media/GUI quality acceptance"]
fn original_visible_frame_input_and_cleanup() -> Result<(), Box<RunFailure>> {
    match std::env::var_os(CHILD_MODE) {
        None => parent_run_for(
            EXACT_TEST,
            CHILD_MODE,
            CHILD_EVIDENCE,
            SCHEMA,
            "runtime/quality-review/visible-render-only-child-process",
        ),
        Some(mode) if mode == "child-v1" => {
            let record = run_inner();
            let root =
                PathBuf::from(std::env::var_os(CHILD_EVIDENCE).expect("original evidence root"));
            assert!(root.is_absolute() && root.is_dir());
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("inner-success.json"))
                .unwrap();
            file.write_all(&serde_json::to_vec_pretty(&record).unwrap())
                .unwrap();
            file.flush().unwrap();
            Ok(())
        }
        Some(_) => Err(Box::new(RunFailure {
            stage: "child mode",
            error: "unknown mode rejected".into(),
            evidence: PathBuf::new(),
            owner: None,
            receipt: None,
            secondary: Vec::new(),
        })),
    }
}
