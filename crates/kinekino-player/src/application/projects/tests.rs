use super::*;
use crate::{application::proposals::StoredProposal, domain::proposals::Correction};
use std::sync::Mutex;
struct Memory(Mutex<Vec<ProjectRevision>>);
impl ProjectStore for Memory {
    fn load(&self, _: &str, revision: Option<u64>) -> Result<StoredProjectRevision, ApiError> {
        let records = self.0.lock().unwrap();
        let latest = records
            .last()
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "missing"))?;
        let project = records
            .iter()
            .find(|p| p.revision.0 == revision.unwrap_or(latest.revision.0))
            .unwrap()
            .clone();
        Ok(StoredProjectRevision {
            revision_sha256: canonical_sha256(&project).unwrap(),
            latest_revision: latest.revision,
            project,
        })
    }
    fn publish(
        &self,
        expected: Option<u64>,
        project: &ProjectRevision,
    ) -> Result<StoredProjectRevision, ApiError> {
        let mut records = self.0.lock().unwrap();
        if records.last().map(|p| p.revision.0) != expected {
            return Err(conflict("changed"));
        }
        records.push(project.clone());
        Ok(StoredProjectRevision {
            revision_sha256: canonical_sha256(project).unwrap(),
            latest_revision: project.revision,
            project: project.clone(),
        })
    }
}
struct Proposals(StoredProposal);
impl ProposalStore for Proposals {
    fn load(&self, _: &str) -> Result<StoredProposal, ApiError> {
        Ok(self.0.clone())
    }
    fn publish(&self, _: Option<u64>, _: &StoredProposal) -> Result<(), ApiError> {
        unreachable!()
    }
}
#[test]
fn invalid_reference_and_empty_context_do_not_publish() {
    let store = Memory(Mutex::new(vec![]));
    let proposals = Proposals(crate::domain::projects::tests::proposal());
    let input = |proposals| CreateProject {
        project_id: "work".into(),
        title: "Working".into(),
        content: ProjectContentInput {
            watching: None,
            proposals,
        },
    };
    assert_eq!(
        create(&store, &proposals, input(vec![])).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        create(
            &store,
            &proposals,
            input(vec![ProposalRevisionRef::Live {
                proposal_id: "draft-a".into(),
                draft_revision: DecimalU64(2),
                working_draft: None
            }])
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        create(
            &store,
            &proposals,
            input(vec![ProposalRevisionRef::Saved {
                proposal_id: "draft-a".into(),
                capture_sha256: "a".repeat(64),
                working_draft: None
            }])
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert!(store.0.lock().unwrap().is_empty());
}
#[test]
fn saved_capture_reopens_and_saves_without_original_proposal_backend() {
    let captured = crate::domain::projects::tests::proposal();
    let proposals = Proposals(captured.clone());
    let store = Memory(Mutex::new(vec![]));
    let first = create(
        &store,
        &proposals,
        CreateProject {
            project_id: "work".into(),
            title: "Working".into(),
            content: ProjectContentInput {
                watching: None,
                proposals: vec![ProposalRevisionRef::Live {
                    proposal_id: "draft-a".into(),
                    draft_revision: DecimalU64(1),
                    working_draft: None,
                }],
            },
        },
    )
    .unwrap();
    let input = SaveProjectRevision {
        project_id: "work".into(),
        title: "Edited".into(),
        content: ProjectContentInput {
            watching: None,
            proposals: vec![ProposalRevisionRef::Saved {
                proposal_id: "draft-a".into(),
                capture_sha256: first.project.proposals[0].capture_sha256.clone(),
                working_draft: Some(ProposalPayload::Correction {
                    edits: vec![Correction {
                        cue_id: "cue-1".into(),
                        text: "edited".into(),
                    }],
                }),
            }],
        },
    };
    let second = save(
        &store,
        &crate::application::proposals::UnavailableProposalStore,
        1,
        input.clone(),
    )
    .unwrap();
    assert_eq!(
        second.project.proposals[0].working_state,
        WorkingState::Valid
    );
    assert_eq!(
        canonical_sha256(&captured).unwrap(),
        second.project.proposals[0].capture_sha256
    );
    assert_eq!(
        save(&store, &proposals, 1, input).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert!(first.project.proposals[0].working_draft.is_none());
}
