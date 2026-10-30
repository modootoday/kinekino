use super::*;
mod providers;
mod schema;
mod wire;
use crate::{
    application::{
        ai_provenance::HistoricalEvidenceReader,
        proposals::{ProposalStore, StoredProposal, UnavailableProposalStore},
    },
    domain::{api_values::DecimalU64, proposals::*},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct Calls {
    load: AtomicUsize,
    publish: AtomicUsize,
    evidence: AtomicUsize,
}
#[derive(Default)]
struct LegacyStore {
    calls: Calls,
}
impl ProposalStore for LegacyStore {
    fn load(&self, _: &str) -> Result<StoredProposal, ApiError> {
        self.calls.load.fetch_add(1, Ordering::SeqCst);
        Err(unavailable())
    }
    fn publish(&self, _: Option<u64>, _: &StoredProposal) -> Result<(), ApiError> {
        self.calls.publish.fetch_add(1, Ordering::SeqCst);
        Err(unavailable())
    }
    fn historical_reader(&self) -> Option<&dyn HistoricalEvidenceReader> {
        self.calls.evidence.fetch_add(1, Ordering::SeqCst);
        None
    }
}
fn no_calls(store: &LegacyStore) {
    assert_eq!(store.calls.load.load(Ordering::SeqCst), 0);
    assert_eq!(store.calls.publish.load(Ordering::SeqCst), 0);
    assert_eq!(store.calls.evidence.load(Ordering::SeqCst), 0);
}

#[test]
fn existing_store_implementation_inherits_optional_none_without_extra_requirements() {
    let store = LegacyStore::default();
    assert!(store.open_output_review("proposal-1").unwrap().is_none());
    no_calls(&store);
}
#[test]
fn independent_history_defaults_to_unsupported_without_loading_current_state() {
    let store = LegacyStore::default();
    assert_eq!(
        store.output_decisions("proposal-1").unwrap_err().code,
        ErrorCode::Unsupported
    );
    no_calls(&store);
}
#[test]
fn unsupported_session_never_loads_publishes_or_verifies_evidence() {
    let store = LegacyStore::default();
    assert!(
        matches!(open_session(&store, "proposal-1"), Err(error) if error.code == ErrorCode::Unsupported)
    );
    assert!(
        matches!(open_session(&store, "../invalid"), Err(error) if error.code == ErrorCode::InvalidArgument)
    );
    no_calls(&store);
}
#[test]
fn unavailable_and_trait_object_paths_keep_explicit_unsupported_status() {
    let store: &dyn ProposalStore = &UnavailableProposalStore;
    assert!(
        matches!(open_session(store, "proposal-1"), Err(error) if error.code == ErrorCode::Unsupported)
    );
    assert_eq!(
        store.output_decisions("proposal-1").unwrap_err().code,
        ErrorCode::Unsupported
    );
    let legacy = LegacyStore::default();
    let object: &dyn ProposalStore = &legacy;
    assert!(object.open_output_review("proposal-1").unwrap().is_none());
    no_calls(&legacy);
}

struct BorrowingStore {
    snapshot: StoredProposal,
    sessions: Arc<AtomicUsize>,
}
struct BorrowedSession<'a> {
    store: &'a BorrowingStore,
}
impl Drop for BorrowedSession<'_> {
    fn drop(&mut self) {
        self.store.sessions.fetch_sub(1, Ordering::SeqCst);
    }
}
impl OutputReviewSession for BorrowedSession<'_> {
    fn current(&self) -> &StoredProposal {
        &self.store.snapshot
    }
    fn verify_current_evidence(&self) -> Result<(), ApiError> {
        Err(unavailable())
    }
    fn append(&mut self, _: &PreparedOutputDecision) -> Result<StoredOutputDecision, ApiError> {
        Err(unavailable())
    }
}
impl ProposalStore for BorrowingStore {
    fn load(&self, _: &str) -> Result<StoredProposal, ApiError> {
        panic!("session must not load")
    }
    fn publish(&self, _: Option<u64>, _: &StoredProposal) -> Result<(), ApiError> {
        panic!("session must not publish")
    }
    fn open_output_review(
        &self,
        _: &str,
    ) -> Result<Option<Box<dyn OutputReviewSession + '_>>, ApiError> {
        self.sessions.fetch_add(1, Ordering::SeqCst);
        Ok(Some(Box::new(BorrowedSession { store: self })))
    }
    fn output_decisions(&self, _: &str) -> Result<OutputDecisionHistory, ApiError> {
        Ok(OutputDecisionHistory {
            records: Vec::new(),
            current_evidence: CurrentEvidenceStatus::NotChecked,
        })
    }
}
fn snapshot() -> StoredProposal {
    StoredProposal {
        document: ProposalDocument {
            schema_version: "1".into(),
            proposal_id: "proposal-1".into(),
            source: SourceSnapshot::Cue(CueSourceSnapshot {
                source_id: "source-1".into(),
                source_revision: DecimalU64(1),
                source_sha256: "a".repeat(64),
                cues: vec![SourceCue {
                    cue_id: "cue-1".into(),
                    text: "Synthetic source".into(),
                    start_us: None,
                    end_us: None,
                }],
            }),
            generated: GeneratedResult {
                task: ProposalTask::Correction,
                raw_text: "synthetic unparsed text".into(),
            },
            draft_revision: DecimalU64(1),
            state: DraftState::NeedsRepair,
            draft: None,
            import_error: Some("Synthetic fixture".into()),
        },
        acceptance: None,
        historical: None,
    }
}
#[test]
fn borrowed_session_retains_snapshot_until_drop_and_history_does_not_claim_verification() {
    let sessions = Arc::new(AtomicUsize::new(0));
    let store = BorrowingStore {
        snapshot: snapshot(),
        sessions: sessions.clone(),
    };
    {
        let session = open_session(&store, "proposal-1").unwrap();
        assert_eq!(sessions.load(Ordering::SeqCst), 1);
        assert!(std::ptr::eq(session.current(), &store.snapshot));
        assert_eq!(
            session.verify_current_evidence().unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
    assert_eq!(sessions.load(Ordering::SeqCst), 0);
    let history = store.output_decisions("proposal-1").unwrap();
    assert!(history.records.is_empty());
    assert_eq!(history.current_evidence, CurrentEvidenceStatus::NotChecked);
}
