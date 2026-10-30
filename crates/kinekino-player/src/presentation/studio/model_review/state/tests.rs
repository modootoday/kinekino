use super::*;
use crate::{application::api::PlayerSdk, domain::model_review::ReviewPackage};
use std::sync::{Arc, mpsc};
mod layout;

fn state() -> ReviewState {
    let mut state = ReviewState::new(Arc::new(PlayerSdk::default()), true);
    let package: ReviewPackage = serde_json::from_value(serde_json::json!({
        "schema_version":"1", "package_id":"test-package", "items":(["first", "second"].map(|id| serde_json::json!({
            "item_id":id,"task":"correction", "source":{"asset_id":"source","sha256":"a".repeat(64),"bytes":"1","media_type":"text/plain"},
            "range":null,"baseline":null,
            "candidate":{"condition":{"model_id":"model","revision":"revision","precision":"f32","engine_id":"test","engine_revision":"1","input_conditions":{}},"assets":[],"automated_checks":[],"measured_resources":{}},
            "criteria":[{"criterion_id":"meaning","how_to_review":"Compare meaning","expected":"No omissions"}],"known_issues":[]
        })))
    })).unwrap();
    state.package = Some(Box::new(ReviewPackageView {
        package_sha256: "a".repeat(64),
        package,
        evidence_verified: true,
        review_ready: false,
        automatic_application_enabled: false,
    }));
    state.select("first");
    state
}

fn pending(state: &mut ReviewState) -> mpsc::Sender<Completion> {
    let (sender, receiver) = mpsc::channel();
    state.generation += 1;
    state.pending = Some(Pending {
        key: Key {
            generation: state.generation,
            package_sha256: "a".repeat(64),
            item_id: state.selected.clone(),
        },
        receiver,
    });
    sender
}

#[test]
fn selection_clears_draft_and_requires_explicit_verdict_and_note() {
    let mut state = state();
    assert!(!state.can_record());
    state.verdict = Some(ReviewVerdict::Deferred);
    assert!(!state.can_record());
    state.note = "판단 근거".into();
    assert!(state.can_record());
    state.select("second");
    assert_eq!(state.selected.as_deref(), Some("second"));
    assert!(state.note.is_empty());
    assert!(state.verdict.is_none());
    assert!(!state.can_record());
    state.select("unknown");
    assert_eq!(state.selected.as_deref(), Some("second"));
}

#[test]
fn pending_request_is_nonblocking_and_locks_selection() {
    let mut state = state();
    let sender = pending(&mut state);
    state.poll();
    assert!(state.busy());
    state.select("second");
    assert_eq!(state.selected.as_deref(), Some("first"));
    drop(sender);
    state.poll();
    assert!(!state.busy());
    assert!(state.error.is_some());
}

#[test]
fn stale_generation_never_populates_history() {
    let mut state = state();
    let sender = pending(&mut state);
    let mut key = state.pending.as_ref().unwrap().key.clone();
    key.generation -= 1;
    sender
        .send(Completion {
            key,
            outcome: Outcome::History(Ok(Vec::new())),
        })
        .unwrap();
    state.poll();
    assert!(state.history.is_none());
    assert!(state.error.is_some());
}

#[test]
fn text_from_another_package_is_not_cached() {
    let mut state = state();
    let sender = pending(&mut state);
    let key = state.pending.as_ref().unwrap().key.clone();
    sender
        .send(Completion {
            key,
            outcome: Outcome::Asset {
                asset_sha256: "b".repeat(64),
                result: Ok(Box::new(ReviewTextAsset {
                    package_sha256: "c".repeat(64),
                    asset_sha256: "b".repeat(64),
                    bytes: crate::domain::api_values::DecimalU64(4),
                    media_type: "text/plain".into(),
                    utf8: "text".into(),
                })),
            },
        })
        .unwrap();
    state.poll();
    assert!(state.assets.is_empty());
    assert!(state.error.is_some());
}
