use super::*;

pub(super) fn revision(
    store: &dyn ProposalStore,
    project_id: String,
    title: String,
    input: ProjectContentInput,
    previous: Option<&ProjectRevision>,
) -> Result<ProjectRevision, ApiError> {
    if input.proposals.len() > 32 {
        return Err(invalid("Too many project proposal references"));
    }
    let mut proposals = Vec::with_capacity(input.proposals.len());
    let mut ids = std::collections::BTreeSet::new();
    for reference in input.proposals {
        let (captured, working_draft, proposal_origin) = match reference {
            ProposalRevisionRef::Live {
                proposal_id,
                draft_revision,
                working_draft,
            } => {
                if !crate::domain::identifiers::valid_id(&proposal_id) || draft_revision.0 == 0 {
                    return Err(invalid("Invalid live proposal reference"));
                }
                let capture = store.capture_current(&proposal_id, draft_revision.0)?;
                let captured = capture.state;
                if captured.document.proposal_id != proposal_id
                    || captured.document.draft_revision != draft_revision
                {
                    return Err(conflict("Live proposal revision changed"));
                }
                (captured, working_draft, capture.origin)
            }
            ProposalRevisionRef::Saved {
                proposal_id,
                capture_sha256,
                working_draft,
            } => {
                let old = previous
                    .and_then(|p| {
                        p.proposals.iter().find(|p| {
                            p.captured.document.proposal_id == proposal_id
                                && p.capture_sha256 == capture_sha256
                        })
                    })
                    .ok_or_else(|| {
                        invalid("Saved capture is absent from the expected project revision")
                    })?;
                (
                    old.captured.clone(),
                    working_draft,
                    old.proposal_origin.clone(),
                )
            }
        };
        captured.validate().map_err(invalid)?;
        crate::application::ai_provenance::verify_bound(store.historical_reader(), &captured)?;
        if !ids.insert(captured.document.proposal_id.clone()) {
            return Err(invalid("Duplicate project proposal reference"));
        }
        let mut snapshot = ProjectProposalSnapshot {
            capture_sha256: canonical_sha256(&captured).map_err(invalid)?,
            captured,
            working_draft,
            working_state: WorkingState::Absent,
            proposal_origin,
        };
        snapshot.working_state = snapshot.working_state().map_err(invalid)?;
        proposals.push(snapshot);
    }
    let parent_revision = previous.map(|p| p.revision);
    let revision = parent_revision
        .map_or(Some(1), |n| n.0.checked_add(1))
        .ok_or_else(|| invalid("Project revision overflow"))?;
    let schema_version = if previous.is_some_and(|p| p.schema_version == "2")
        || proposals.iter().any(|p| p.proposal_origin.is_some())
    {
        "2"
    } else {
        "1"
    };
    let mut project = ProjectRevision {
        schema_version: schema_version.into(),
        project_id,
        revision: DecimalU64(revision),
        parent_revision,
        title,
        watching: input.watching,
        proposals,
        work_state: ProjectWorkState::default(),
    };
    project.work_state = project.derived_work_state();
    project.validate().map_err(invalid)?;
    Ok(project)
}
