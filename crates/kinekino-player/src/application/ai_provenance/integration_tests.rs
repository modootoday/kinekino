use super::*;
use crate::{
    application::proposals::{self, ProposalStore},
    domain::{api_values::DecimalU64, proposals::*},
};
use std::sync::Mutex;

#[derive(Default)]
struct MemoryStore(Mutex<Option<StoredProposal>>);
impl ProposalStore for MemoryStore {
    fn load(&self, _: &str) -> Result<StoredProposal, ApiError> {
        self.0
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "missing proposal"))
    }
    fn publish(&self, expected: Option<u64>, state: &StoredProposal) -> Result<(), ApiError> {
        let mut slot = self.0.lock().unwrap();
        if slot.as_ref().map(|s| s.document.draft_revision.0) != expected {
            return Err(ApiError::new(ErrorCode::RevisionConflict, "stale proposal"));
        }
        state.validate().map_err(invalid)?;
        *slot = Some(state.clone());
        Ok(())
    }
}
struct Reader(Option<VerifiedHistoricalSample>);
impl HistoricalEvidenceReader for Reader {
    fn verify(&self, _: &CompanionReference) -> Result<VerifiedHistoricalSample, ApiError> {
        self.0
            .clone()
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "missing companion"))
    }
}

#[test]
fn synthetic_historical_binding_survives_sdk_create_edit_accept_and_serialized_readback() {
    let sample = crate::test_support::historical_fixture::correction();
    let source = sample.source_request_utf8.clone();
    let raw = sample.candidate_raw_utf8.clone();
    let reference = sample.reference.clone();
    let snapshot = proposals::source_import::from_request_asset(
        "historical-source".into(),
        DecimalU64(1),
        sample.source_sha256.clone(),
        source,
        ProposalTask::Correction,
    )
    .unwrap();
    let input = CreateProposal {
        proposal_id: "historical-binding-test".into(),
        source: snapshot,
        generated: GeneratedResult {
            task: ProposalTask::Correction,
            raw_text: raw.clone(),
        },
    };
    assert!(prepare(&Reader(None), reference.clone(), input.clone()).is_err());
    struct UnexpectedIo;
    impl HistoricalEvidenceReader for UnexpectedIo {
        fn verify(&self, _: &CompanionReference) -> Result<VerifiedHistoricalSample, ApiError> {
            panic!("Malformed reference must not reach evidence I/O")
        }
    }
    let mut invalid_reference = reference.clone();
    invalid_reference.item_id = "../objects".into();
    assert!(prepare(&UnexpectedIo, invalid_reference, input.clone()).is_err());
    let mut wrong = sample.clone();
    wrong.reference.item_id = "wrong-item".into();
    assert!(prepare(&Reader(Some(wrong)), reference.clone(), input.clone()).is_err());
    let reader = Reader(Some(sample));
    let (input, bound) = prepare(&reader, reference, input).unwrap().into_parts();
    let store = MemoryStore::default();
    let created = proposals::create(&store, input).unwrap();
    assert_eq!(created.state, DraftState::NeedsRepair);
    let SourceSnapshot::Document(document) = &created.source else {
        panic!("document expected")
    };
    let draft = ProposalPayload::DocumentCorrection {
        edits: vec![DocumentCorrection {
            source_ref: document.text_reference("/0").unwrap(),
            text: "Technical edit fixture; not a quality verdict.".into(),
        }],
    };
    verify_stored(
        &reader,
        &bound,
        &store.load("historical-binding-test").unwrap(),
    )
    .unwrap();
    let edit = proposals::EditProposal {
        proposal_id: created.proposal_id.clone(),
        draft,
    };
    proposals::edit(&store, edit.clone(), 1).unwrap();
    assert!(proposals::edit(&store, edit, 1).is_err());
    verify_stored(
        &reader,
        &bound,
        &store.load("historical-binding-test").unwrap(),
    )
    .unwrap();
    let receipt = proposals::accept(
        &store,
        proposals::AcceptProposal {
            proposal_id: created.proposal_id,
            target_source_revision: DecimalU64(1),
        },
        2,
    )
    .unwrap();
    assert!(!receipt.automatic_application_enabled);
    let saved = serde_json::to_vec(&store.load("historical-binding-test").unwrap()).unwrap();
    let reopened: StoredProposal = serde_json::from_slice(&saved).unwrap();
    verify_stored(&reader, &bound, &reopened).unwrap();
    assert_eq!(reopened.document.generated.raw_text, raw);
    let mut tampered = reopened.clone();
    tampered.document.generated.raw_text.push(' ');
    assert!(verify_stored(&reader, &bound, &tampered).is_err());
    assert!(verify_stored(&Reader(None), &bound, &reopened).is_err());
}
