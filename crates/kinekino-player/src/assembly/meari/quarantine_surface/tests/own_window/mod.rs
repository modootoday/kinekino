//! One ignored composite entry; ordinary Native unit gates never run its OS body.
mod fixture;
mod host;
mod supervision;
use fixture::Evidence;
use host::Controller;
use meari_native::{
    domain::Viewport,
    infrastructure::{
        document::InteractiveDocument,
        native_host::{HostScheduleLimits, HostTurnReason, NativeDocumentHandle},
        window::NativeLoopOwner,
    },
};
use std::{
    cell::RefCell,
    fs,
    io::Write,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};
use supervision::{CHILD_EVIDENCE, CHILD_MODE, EXACT_TEST, RunFailure, parent_run};

fn run_inner_window() -> serde_json::Value {
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
        .find(|element| element.dom_tag == "video")
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
    let controller = Controller {
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
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
    let evidence = evidence.borrow();
    assert!(evidence.failure.is_none(), "{:?}", evidence.failure);
    assert!(evidence.inert_partial && evidence.partial_retained && evidence.partial_released);
    assert!(
        evidence.disabled_read
            && evidence.expired_cleanup_rejected
            && evidence.remaining_zero
            && evidence.close_seen
    );
    let consumed = evidence.consumed_frame.as_ref().unwrap();
    let reflected = evidence.reflected_turn.as_ref().unwrap();
    assert_eq!(consumed.host_id, reflected.host_id);
    assert_eq!(consumed.owner, reflected.owner);
    assert_eq!(consumed.definition_revision, reflected.definition_revision);
    assert_eq!(consumed.document_revision, reflected.document_revision);
    assert_eq!(
        consumed.turn_sequence.checked_add(1),
        Some(reflected.turn_sequence)
    );
    assert_eq!(evidence.reflected_reason, Some(HostTurnReason::Completion));
    assert!(evidence.reflection_wake.is_some());
    assert!(
        Instant::now() < deadline,
        "normal own-window cleanup exceeded inner budget"
    );
    // run_startup owns the actual loop/Destroyed-event completion; deadline only requests original cleanup.
    serde_json::json!({
        "schema":"quarantine-own-window-inner-v1","test":EXACT_TEST,
        "native_cleanup_complete":true,"inert_partial":evidence.inert_partial,
        "partial_retained":evidence.partial_retained,"partial_released":evidence.partial_released,
        "disabled_read":evidence.disabled_read,"expired_cleanup_rejected":evidence.expired_cleanup_rejected,
        "remaining_zero":evidence.remaining_zero,"close_seen":evidence.close_seen,
        "consumed_frame":{"host_id":consumed.host_id,"owner":format!("{:?}",consumed.owner),"turn_sequence":consumed.turn_sequence,"document_revision":consumed.document_revision,"definition_revision":consumed.definition_revision},
        "reflected_turn":{"host_id":reflected.host_id,"owner":format!("{:?}",reflected.owner),"turn_sequence":reflected.turn_sequence,"document_revision":reflected.document_revision,"definition_revision":reflected.definition_revision,"reason":format!("{:?}",evidence.reflected_reason)},
        "own_wake_enqueued":evidence.reflection_wake,
        "elapsed_ms":started.elapsed().as_millis(),"inner_budget_ms":15000,
        "class_os_failure_injected":false,"pre_revocation_admission_preservation_tested":false,
    })
}

#[test]
#[ignore = "requires separately allocated own-window OS slot; one parent result, no media/visual/focus claim"]
fn original_registrar_frame_disabled_child_and_cleanup() -> Result<(), Box<RunFailure>> {
    match std::env::var_os(CHILD_MODE) {
        None => parent_run(),
        Some(value) if value == "child-v1" => {
            let record = run_inner_window();
            let root = PathBuf::from(
                std::env::var_os(CHILD_EVIDENCE).expect("parent must supply evidence root"),
            );
            assert!(root.is_absolute() && root.is_dir());
            let mut file = fs::OpenOptions::new()
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
