use super::*;
mod historical;
use crate::{
    application::proposals::{self, AcceptProposal, CreateProposal, EditProposal},
    domain::{api_values::DecimalU64, proposals::*},
};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "player-proposals-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn input() -> CreateProposal {
    CreateProposal {
        proposal_id: "draft-1".into(),
        source: SourceSnapshot::Cue(CueSourceSnapshot {
            source_id: "source-1".into(),
            source_revision: DecimalU64(7),
            source_sha256: "a".repeat(64),
            cues: vec![SourceCue {
                cue_id: "cue-1".into(),
                text: "source original".into(),
                start_us: None,
                end_us: None,
            }],
        }),
        generated: GeneratedResult {
            task: ProposalTask::Correction,
            raw_text: "{\"error\":\"actual model schema failure\"}".into(),
        },
    }
}
fn draft(text: &str) -> ProposalPayload {
    ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "cue-1".into(),
            text: text.into(),
        }],
    }
}
fn digest<T: serde::Serialize>(v: &T) -> String {
    content_digest::sha256(&mut Cursor::new(serde_json::to_vec(v).unwrap()), MAX_STATE)
        .unwrap()
        .1
}
#[test]
fn malformed_import_manual_repair_acceptance_and_lost_response_preserve_originals() {
    let temp = Temp::new();
    let store = ProposalFiles::open(&temp.0).unwrap();
    let before = proposals::create(&store, input()).unwrap();
    assert_eq!(before.state, DraftState::NeedsRepair);
    assert!(before.draft.is_none());
    assert!(before.import_error.is_some());
    let accept = || AcceptProposal {
        proposal_id: "draft-1".into(),
        target_source_revision: DecimalU64(7),
    };
    assert_eq!(
        proposals::accept(&store, accept(), 1).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    let mut bad = draft("fixed");
    if let ProposalPayload::Correction { edits } = &mut bad {
        edits[0].cue_id = "absent".into();
    }
    assert!(
        proposals::edit(
            &store,
            EditProposal {
                proposal_id: "draft-1".into(),
                draft: bad
            },
            1
        )
        .is_err()
    );
    let edited = proposals::edit(
        &store,
        EditProposal {
            proposal_id: "draft-1".into(),
            draft: draft("user fixed"),
        },
        1,
    )
    .unwrap();
    assert_eq!(edited.state, DraftState::Editable);
    assert_eq!(edited.draft_revision.0, 2);
    assert!(
        proposals::accept(
            &store,
            AcceptProposal {
                proposal_id: "draft-1".into(),
                target_source_revision: DecimalU64(8)
            },
            2
        )
        .is_err()
    );
    let receipt = proposals::accept(&store, accept(), 2).unwrap();
    assert!(!receipt.automatic_application_enabled);
    assert_eq!(receipt.output_revision.0, 3);
    let retry = proposals::accept(&store, accept(), 2).unwrap();
    assert_eq!(digest(&retry), digest(&receipt));
    let final_state = store.load("draft-1").unwrap();
    assert_eq!(digest(&before.source), digest(&final_state.document.source));
    assert_eq!(
        digest(&before.generated),
        digest(&final_state.document.generated)
    );
    assert!(final_state.document.source.cues()[0].start_us.is_none());
    assert_eq!(
        store.revisions(&temp.0.join("p-draft-1")).unwrap(),
        vec![1, 2, 3]
    );
    assert!(
        proposals::edit(
            &store,
            EditProposal {
                proposal_id: "draft-1".into(),
                draft: draft("stale")
            },
            2
        )
        .is_err()
    );
}
#[test]
fn same_revision_edit_accept_publication_has_one_winner() {
    let temp = Temp::new();
    let store = Arc::new(ProposalFiles::open(&temp.0).unwrap());
    proposals::create(store.as_ref(), input()).unwrap();
    proposals::edit(
        store.as_ref(),
        EditProposal {
            proposal_id: "draft-1".into(),
            draft: draft("first"),
        },
        1,
    )
    .unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let a = store.clone();
    let b = barrier.clone();
    let edit = std::thread::spawn(move || {
        b.wait();
        proposals::edit(
            a.as_ref(),
            EditProposal {
                proposal_id: "draft-1".into(),
                draft: draft("concurrent"),
            },
            2,
        )
        .map(|_| ())
    });
    let a = store.clone();
    let b = barrier.clone();
    let accept = std::thread::spawn(move || {
        b.wait();
        proposals::accept(
            a.as_ref(),
            AcceptProposal {
                proposal_id: "draft-1".into(),
                target_source_revision: DecimalU64(7),
            },
            2,
        )
        .map(|_| ())
    });
    barrier.wait();
    let results = [edit.join().unwrap(), accept.join().unwrap()];
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .filter_map(|r| r.as_ref().err())
            .all(|e| e.code == ErrorCode::RevisionConflict)
    );
    assert_eq!(store.load("draft-1").unwrap().document.draft_revision.0, 3);
}
#[test]
fn partial_or_changed_file_is_never_a_successful_publication() {
    let temp = Temp::new();
    let store = ProposalFiles::open(&temp.0).unwrap();
    proposals::create(&store, input()).unwrap();
    let revision = temp.0.join("p-draft-1/r-2");
    fs::create_dir(&revision).unwrap();
    assert_eq!(
        store.load("draft-1").unwrap_err().code,
        ErrorCode::NeedsInput
    );
    fs::write(
        revision.join(format!("{}.json", "0".repeat(64))),
        b"{\"document\":",
    )
    .unwrap();
    assert_eq!(
        store.load("draft-1").unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert!(
        proposals::edit(
            &store,
            EditProposal {
                proposal_id: "draft-1".into(),
                draft: draft("cannot skip corrupt")
            },
            1
        )
        .is_err()
    );
}
#[test]
fn typed_tasks_reject_invented_times_unknown_fields_and_cross_source_refs() {
    let source = input().source;
    for raw in [
        r#"{"task":"highlights","highlights":[{"title":"x","source_refs":["cue-1"],"start_us":"12"}]}"#,
        r#"{"task":"correction","edits":[{"cue_id":"cue-1","text":"x","extra":true}]}"#,
    ] {
        assert!(serde_json::from_str::<ProposalPayload>(raw).is_err());
    }
    for payload in [
        ProposalPayload::Highlights {
            highlights: vec![Highlight {
                title: "x".into(),
                source_refs: vec!["absent".into()],
            }],
        },
        ProposalPayload::Cards {
            cards: vec![Card {
                front: "x".into(),
                back: "y".into(),
                source_refs: vec!["cue-1".into(), "cue-1".into()],
            }],
        },
    ] {
        assert!(payload.validate(&source, payload.task()).is_err());
    }
}
#[test]
fn oversized_sdk_import_does_not_reserve_a_broken_identity() {
    let temp = Temp::new();
    let store = ProposalFiles::open(&temp.0).unwrap();
    let mut input = input();
    let SourceSnapshot::Cue(source) = &mut input.source else {
        panic!("cue fixture required")
    };
    source.cues = (0..70)
        .map(|i| SourceCue {
            cue_id: format!("cue-{i}"),
            text: "x".repeat(65_536),
            start_us: None,
            end_us: None,
        })
        .collect();
    assert_eq!(
        proposals::create(&store, input).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert!(!temp.0.join("p-draft-1").exists());
}
#[test]
fn document_source_and_receipt_survive_file_store_reopen_without_becoming_cues() {
    let temp = Temp::new();
    let store = ProposalFiles::open(&temp.0).unwrap();
    let raw=serde_json::json!({"messages":[{"role":"user","content":"Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n[\"Original ASR draft.\"]"}]}).to_string();
    let sha = content_digest::sha256(&mut Cursor::new(raw.as_bytes()), 65_536)
        .unwrap()
        .1;
    let source = proposals::source_import::from_request_asset(
        "asr-source".into(),
        DecimalU64(1),
        sha,
        raw,
        ProposalTask::Correction,
    )
    .unwrap();
    let reference = source.as_document().unwrap().text_reference("/0").unwrap();
    let original = proposals::create(
        &store,
        CreateProposal {
            proposal_id: "document-1".into(),
            source,
            generated: GeneratedResult {
                task: ProposalTask::Correction,
                raw_text: "{\"proposed\":\"truncated".into(),
            },
        },
    )
    .unwrap();
    assert_eq!(original.schema_version, "2");
    assert_eq!(original.state, DraftState::NeedsRepair);
    assert!(original.source.cues().is_empty());
    let edit = ProposalPayload::DocumentCorrection {
        edits: vec![DocumentCorrection {
            source_ref: reference,
            text: "Original ASR draft!".into(),
        }],
    };
    proposals::edit(
        &store,
        EditProposal {
            proposal_id: "document-1".into(),
            draft: edit,
        },
        1,
    )
    .unwrap();
    let receipt = proposals::accept(
        &store,
        AcceptProposal {
            proposal_id: "document-1".into(),
            target_source_revision: DecimalU64(1),
        },
        2,
    )
    .unwrap();
    drop(store);
    let reopened = ProposalFiles::open(&temp.0).unwrap();
    let restored = proposals::inspect(
        &reopened,
        proposals::InspectProposal {
            proposal_id: "document-1".into(),
        },
    )
    .unwrap();
    assert_eq!(digest(&restored.acceptance), digest(&Some(receipt)));
    assert_eq!(digest(&original.source), digest(&restored.document.source));
    assert_eq!(
        digest(&original.generated),
        digest(&restored.document.generated)
    );
    assert!(restored.document.source.cues().is_empty());
}
