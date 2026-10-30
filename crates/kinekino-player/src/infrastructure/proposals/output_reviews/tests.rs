use super::*;
use crate::{
    application::{
        ai_provenance::{
            self, HistoricalEvidenceReader, ImportHistoricalProposal, VerifiedHistoricalSample,
        },
        proposals::{
            self, AcceptProposal, CreateProposal, EditProposal, ProposalStore,
            output_review::{OutputReviewClock, PreparedOutputDecision, criteria_for},
        },
    },
    domain::{
        api_values::DecimalU64,
        proposals::{output_review::*, *},
    },
    infrastructure::proposals::reservation::WriterReservation,
};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};
mod concurrency;
mod recovery;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "output-review-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[derive(Default)]
struct Clock {
    calls: AtomicUsize,
    fail: AtomicBool,
}
impl OutputReviewClock for Clock {
    fn observe_record_time(&self) -> Result<ObservedRecordTime, ApiError> {
        let count = self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Synthetic record clock failed",
            ));
        }
        Ok(ObservedRecordTime {
            unix_ms: DecimalU64(1000 + count as u64),
            clock: RecordClock::LocalSystemUnixMs,
        })
    }
}
fn seed(store: &ProposalFiles) {
    proposals::create(store, CreateProposal {
        proposal_id: "proposal-1".into(),
        source: SourceSnapshot::Cue(CueSourceSnapshot { source_id: "source-1".into(), source_revision: DecimalU64(7), source_sha256: "a".repeat(64), cues: vec![SourceCue { cue_id: "cue-1".into(), text: "Synthetic source.".into(), start_us: None, end_us: None }] }),
        generated: GeneratedResult { task: ProposalTask::Correction, raw_text: r#"{"task":"correction","edits":[{"cue_id":"cue-1","text":"Synthetic source!"}]}"#.into() },
    }).unwrap();
    proposals::accept(
        store,
        AcceptProposal {
            proposal_id: "proposal-1".into(),
            target_source_revision: DecimalU64(7),
        },
        1,
    )
    .unwrap();
}
fn prepared(state: &StoredProposal, id: &str) -> PreparedOutputDecision {
    let criteria = criteria_for(state.document.generated.task).unwrap();
    PreparedOutputDecision::new(
        OutputDecisionInput {
            decision_id: id.into(),
            target: OutputReviewTarget::from_stored(state).unwrap(),
            criteria: criteria.identity().unwrap(),
            verdict: OutputVerdict::Deferred,
            note: "Synthetic persistence test; no actual quality approval.".into(),
            declared_reviewer: DeclaredReviewer {
                label: "Declared synthetic reviewer".into(),
                reviewer_id: Some("fixture".into()),
            },
            claimed_review_unix_ms: Some(DecimalU64(19)),
        },
        state,
        criteria,
    )
    .unwrap()
}
fn record(
    store: &ProposalFiles,
    id: &str,
) -> crate::application::proposals::output_review::StoredOutputDecision {
    let mut session = store.open_output_review("proposal-1").unwrap().unwrap();
    let request = prepared(session.current(), id);
    session.append(&request).unwrap()
}
fn directory(root: &Temp, id: &str) -> PathBuf {
    root.0
        .join("output-decisions/p-proposal-1")
        .join(format!("d-{id}"))
}
fn file(root: &Temp, id: &str) -> PathBuf {
    fs::read_dir(directory(root, id))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
}
fn fixed_decision(store: &ProposalFiles, id: &str) -> OutputDecision {
    let state = store.load("proposal-1").unwrap();
    let prepared = prepared(&state, id);
    OutputDecision::prepare(
        prepared.input().clone(),
        &state,
        prepared.criteria().clone(),
        ObservedRecordTime {
            unix_ms: DecimalU64(23),
            clock: RecordClock::LocalSystemUnixMs,
        },
    )
    .unwrap()
}

#[test]
fn append_fresh_readback_and_same_request_keep_original_body_time_and_proposal_bytes() {
    let root = Temp::new();
    let clock = Arc::new(Clock::default());
    let store = ProposalFiles::open(&root.0)
        .unwrap()
        .with_output_review_clock(clock.clone());
    seed(&store);
    let before = serde_json::to_vec(&store.load("proposal-1").unwrap()).unwrap();
    let first = record(&store, "decision-1");
    let bytes = fs::read(file(&root, "decision-1")).unwrap();
    let retried = record(&store, "decision-1");
    assert_eq!(first.decision_sha256, retried.decision_sha256);
    assert_eq!(first.decision, retried.decision);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    assert_eq!(fs::read(file(&root, "decision-1")).unwrap(), bytes);
    assert_eq!(
        serde_json::to_vec(&store.load("proposal-1").unwrap()).unwrap(),
        before
    );
    let fresh = ProposalFiles::open(&root.0).unwrap();
    let history = fresh.output_decisions("proposal-1").unwrap();
    assert_eq!(history.records[0].decision, first.decision);
    assert_eq!(
        history.current_evidence,
        crate::application::proposals::output_review::CurrentEvidenceStatus::NotChecked
    );
    assert!(
        !first.decision.counts_as_model_sample && !first.decision.automatic_application_enabled
    );
}

#[test]
fn same_id_different_request_and_untrusted_criteria_never_replace_record() {
    let root = Temp::new();
    let store = ProposalFiles::open(&root.0).unwrap();
    seed(&store);
    record(&store, "decision-1");
    let original = fs::read(file(&root, "decision-1")).unwrap();
    let mut session = store.open_output_review("proposal-1").unwrap().unwrap();
    let base = prepared(session.current(), "decision-1");
    for case in 0..4 {
        let mut input = base.input().clone();
        match case {
            0 => input.note.push('!'),
            1 => input.verdict = OutputVerdict::NeedsChanges,
            2 => input.claimed_review_unix_ms = Some(DecimalU64(20)),
            _ => input.declared_reviewer.label.push('!'),
        }
        let changed =
            PreparedOutputDecision::new(input, session.current(), base.criteria().clone()).unwrap();
        assert_eq!(
            session.append(&changed).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
    }
    let mut criteria = base.criteria().clone();
    criteria.purpose.push('!');
    let mut input = base.input().clone();
    input.decision_id = "decision-2".into();
    input.criteria = criteria.identity().unwrap();
    let changed = PreparedOutputDecision::new(input, session.current(), criteria).unwrap();
    assert_eq!(
        session.append(&changed).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(fs::read(file(&root, "decision-1")).unwrap(), original);
    assert!(!directory(&root, "decision-2").exists());
}

#[test]
fn clock_failure_has_no_namespace_or_reserved_decision() {
    let root = Temp::new();
    let clock = Arc::new(Clock::default());
    clock.fail.store(true, Ordering::SeqCst);
    let store = ProposalFiles::open(&root.0)
        .unwrap()
        .with_output_review_clock(clock);
    seed(&store);
    let mut session = store.open_output_review("proposal-1").unwrap().unwrap();
    let request = prepared(session.current(), "decision-1");
    assert_eq!(
        session.append(&request).unwrap_err().code,
        ErrorCode::EngineFailed
    );
    assert!(!root.0.join("output-decisions").exists());
}

#[test]
fn history_is_independent_of_proposal_and_clock_and_has_deterministic_id_order() {
    let root = Temp::new();
    let store = ProposalFiles::open(&root.0).unwrap();
    seed(&store);
    record(&store, "z");
    record(&store, "a");
    fs::remove_dir_all(root.0.join("p-proposal-1")).unwrap();
    let clock = Arc::new(Clock::default());
    clock.fail.store(true, Ordering::SeqCst);
    let fresh = ProposalFiles::open(&root.0)
        .unwrap()
        .with_output_review_clock(clock.clone());
    let history = fresh.output_decisions("proposal-1").unwrap();
    assert_eq!(
        history
            .records
            .iter()
            .map(|r| r.decision.input.decision_id.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "z"]
    );
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    assert!(fresh.open_output_review("proposal-1").is_err());
}

#[test]
fn history_capacity_preserves_existing_retry_and_rejects_new_reservation() {
    let root = Temp::new();
    let store = ProposalFiles::open(&root.0).unwrap();
    seed(&store);
    let original = record(&store, "decision-0");
    let prototype = fixed_decision(&store, "fixture");
    {
        let writer = WriterReservation::acquire(&store).unwrap();
        for index in 1..MAX_HISTORY {
            let mut value = prototype.clone();
            value.input.decision_id = format!("decision-{index}");
            publication::publish(&store, &writer, &value).unwrap();
        }
    }
    assert_eq!(
        record(&store, "decision-0").decision_sha256,
        original.decision_sha256
    );
    let mut session = store.open_output_review("proposal-1").unwrap().unwrap();
    let request = prepared(session.current(), "overflow");
    assert_eq!(
        session.append(&request).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert!(!directory(&root, "overflow").exists());
}

struct Reader {
    sample: VerifiedHistoricalSample,
    present: AtomicBool,
    tampered: AtomicBool,
}
impl HistoricalEvidenceReader for Reader {
    fn verify(&self, _: &CompanionReference) -> Result<VerifiedHistoricalSample, ApiError> {
        if !self.present.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Synthetic evidence missing",
            ));
        }
        let mut sample = self.sample.clone();
        if self.tampered.load(Ordering::SeqCst) {
            sample.candidate_raw_utf8.replace_range(0..1, "x");
        }
        Ok(sample)
    }
}
#[test]
fn evidence_loss_blocks_new_record_but_keeps_stored_history_readable() {
    let root = Temp::new();
    let reader = Arc::new(Reader {
        sample: crate::test_support::historical_fixture::correction(),
        present: AtomicBool::new(true),
        tampered: AtomicBool::new(false),
    });
    let store = ProposalFiles::open(&root.0)
        .unwrap()
        .with_historical_reader(reader.clone());
    let imported = ai_provenance::import(
        &store,
        ImportHistoricalProposal {
            proposal_id: "proposal-1".into(),
            source_id: "source-1".into(),
            source_revision: DecimalU64(7),
            reference: reader.sample.reference.clone(),
        },
    )
    .unwrap();
    let SourceSnapshot::Document(source) = &imported.document.source else {
        panic!("document fixture")
    };
    proposals::edit(
        &store,
        EditProposal {
            proposal_id: "proposal-1".into(),
            draft: ProposalPayload::DocumentCorrection {
                edits: vec![DocumentCorrection {
                    source_ref: source.text_reference("/0").unwrap(),
                    text: "Synthetic edited text.".into(),
                }],
            },
        },
        1,
    )
    .unwrap();
    proposals::accept(
        &store,
        AcceptProposal {
            proposal_id: "proposal-1".into(),
            target_source_revision: DecimalU64(7),
        },
        2,
    )
    .unwrap();
    record(&store, "decision-1");
    let mut session = store.open_output_review("proposal-1").unwrap().unwrap();
    let request = prepared(session.current(), "decision-2");
    reader.present.store(false, Ordering::SeqCst);
    assert_eq!(
        session.append(&request).unwrap_err().code,
        ErrorCode::NotFound
    );
    let retry = prepared(session.current(), "decision-1");
    assert_eq!(
        session.append(&retry).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert!(!directory(&root, "decision-2").exists());
    reader.present.store(true, Ordering::SeqCst);
    reader.tampered.store(true, Ordering::SeqCst);
    assert_eq!(
        session.append(&request).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert!(!directory(&root, "decision-2").exists());
    drop(session);
    assert_eq!(
        store.output_decisions("proposal-1").unwrap().records.len(),
        1
    );
    assert!(store.open_output_review("proposal-1").is_err());
}

#[test]
fn unrelated_legacy_candidate_fixture_bytes_are_untouched() {
    let root = Temp::new();
    let store = ProposalFiles::open(&root.0).unwrap();
    seed(&store);
    let fixture = root.0.join("legacy-candidate-fixture");
    fs::create_dir(&fixture).unwrap();
    let originals = [
        ("package.json", b"{\"synthetic_package\":1}".as_slice()),
        ("candidate.txt", b"Candidate fixed bytes\r\n".as_slice()),
        ("decision.json", b"{\"scope\":\"sample_only\"}".as_slice()),
    ];
    for (name, bytes) in originals {
        fs::write(fixture.join(name), bytes).unwrap();
    }
    record(&store, "decision-1");
    for (name, bytes) in originals {
        assert_eq!(fs::read(fixture.join(name)).unwrap(), bytes);
    }
    // Byte non-mutation of explicit fixtures only; not a complete legacy schema/CLI test.
}

#[test]
fn invalid_ids_and_unexpected_history_entries_are_not_silently_ignored() {
    let root = Temp::new();
    let store = ProposalFiles::open(&root.0).unwrap();
    assert_eq!(
        store.output_decisions("../invalid").unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert!(!root.0.join("output-decisions").exists());
    assert!(store.output_decisions("absent").unwrap().records.is_empty());
    seed(&store);
    record(&store, "decision-1");
    fs::write(
        root.0.join("output-decisions/p-proposal-1/unexpected"),
        b"unknown",
    )
    .unwrap();
    assert_eq!(
        store.output_decisions("proposal-1").unwrap_err().code,
        ErrorCode::NeedsInput
    );
}
