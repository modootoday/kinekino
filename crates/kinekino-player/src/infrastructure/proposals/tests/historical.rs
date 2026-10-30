use super::*;
use crate::application::ai_provenance::{
    self, CompanionReference, HistoricalEvidenceReader, ImportHistoricalProposal,
    VerifiedHistoricalSample,
};
use std::sync::atomic::AtomicBool;

struct Reader {
    sample: VerifiedHistoricalSample,
    available: AtomicBool,
}
impl HistoricalEvidenceReader for Reader {
    fn verify(&self, reference: &CompanionReference) -> Result<VerifiedHistoricalSample, ApiError> {
        if !self.available.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "technical fixture evidence unavailable",
            ));
        }
        if reference != &self.sample.reference {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "technical fixture reference differs",
            ));
        }
        Ok(self.sample.clone())
    }
}
fn reader() -> Arc<Reader> {
    Arc::new(Reader {
        sample: crate::test_support::historical_fixture::correction(),
        available: AtomicBool::new(true),
    })
}

#[test]
fn single_publication_binding_survives_edit_accept_reopen_and_rejects_missing_evidence() {
    let temp = Temp::new();
    let reader = reader();
    let store = ProposalFiles::open(&temp.0)
        .unwrap()
        .with_historical_reader(reader.clone());
    let created = ai_provenance::import(
        &store,
        ImportHistoricalProposal {
            proposal_id: "bound".into(),
            source_id: "import-source".into(),
            source_revision: DecimalU64(1),
            reference: reader.sample.reference.clone(),
        },
    )
    .unwrap();
    assert_eq!(created.document.state, DraftState::NeedsRepair);
    assert_eq!(fs::read_dir(temp.0.join("p-bound/r-1")).unwrap().count(), 1);
    let SourceSnapshot::Document(source) = &created.document.source else {
        panic!("document source required")
    };
    let draft = ProposalPayload::DocumentCorrection {
        edits: vec![DocumentCorrection {
            source_ref: source.text_reference("/0").unwrap(),
            text: "Technical fixture only; no quality approval.".into(),
        }],
    };
    proposals::edit(
        &store,
        EditProposal {
            proposal_id: "bound".into(),
            draft: draft.clone(),
        },
        1,
    )
    .unwrap();
    assert_eq!(
        proposals::edit(
            &store,
            EditProposal {
                proposal_id: "bound".into(),
                draft
            },
            1
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    let accept = || AcceptProposal {
        proposal_id: "bound".into(),
        target_source_revision: DecimalU64(1),
    };
    let receipt = proposals::accept(&store, accept(), 2).unwrap();
    assert!(!receipt.automatic_application_enabled);
    assert_eq!(
        proposals::accept(&store, accept(), 2)
            .unwrap()
            .output_revision
            .0,
        3
    );
    drop(store);
    let unconfigured = ProposalFiles::open(&temp.0).unwrap();
    assert_eq!(
        unconfigured.load("bound").unwrap_err().code,
        ErrorCode::Unsupported
    );
    drop(unconfigured);
    let store = ProposalFiles::open(&temp.0)
        .unwrap()
        .with_historical_reader(reader.clone());
    let reopened = store.load("bound").unwrap();
    assert_eq!(reopened.historical, created.historical);
    assert_eq!(
        reopened.document.generated.raw_text,
        created.document.generated.raw_text
    );
    let mut changed = reopened.clone();
    changed.document.draft_revision = DecimalU64(4);
    changed.acceptance = None;
    changed.historical = None;
    assert_eq!(
        store.publish(Some(3), &changed).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    reader.available.store(false, Ordering::SeqCst);
    assert_eq!(store.load("bound").unwrap_err().code, ErrorCode::NotFound);
    assert_eq!(
        proposals::accept(&store, accept(), 2).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert!(!temp.0.join("p-bound/r-4").exists());
}
