use super::*;
use crate::domain::{
    identifiers::valid_id,
    local_video::LocalVideo,
    proposals::{DraftState, SourceSnapshot},
};
use std::{collections::BTreeSet, path::PathBuf};

fn text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
impl WatchingSelection {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !text(&self.label, 1024) {
            return Err("Invalid watching label");
        }
        match &self.source {
            WatchingSource::LocalVideo { source_id, path } => {
                if !valid_id(source_id)
                    || path.len() > 32768
                    || path.chars().any(char::is_control)
                    || LocalVideo::selected(PathBuf::from(path)).is_err()
                {
                    return Err("Invalid selected local video locator");
                }
            }
            WatchingSource::EpisodePage {
                source_id,
                episode_url,
            } => {
                let url = url::Url::parse(episode_url).map_err(|_| "Invalid episode locator")?;
                if !valid_id(source_id)
                    || episode_url.len() > 8192
                    || !text(episode_url, 8192)
                    || url.scheme() != "https"
                    || url.host_str() != Some("catalog.example.test")
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.port().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
                    || !url
                        .path()
                        .strip_prefix("/e/")
                        .is_some_and(|tail| !tail.is_empty())
                {
                    return Err("Invalid selected episode locator");
                }
            }
        }
        Ok(())
    }
}
impl ProjectProposalSnapshot {
    pub fn working_state(&self) -> Result<WorkingState, &'static str> {
        let Some(draft) = &self.working_draft else {
            return Ok(WorkingState::Absent);
        };
        let source = &self.captured.document.source;
        let correct_kind = matches!(
            (source, draft),
            (
                SourceSnapshot::Document(_),
                ProposalPayload::DocumentCorrection { .. }
            ) | (
                SourceSnapshot::Cue(_),
                ProposalPayload::Correction { .. }
                    | ProposalPayload::Cards { .. }
                    | ProposalPayload::Highlights { .. }
            )
        );
        if draft.task() != self.captured.document.generated.task || !correct_kind {
            return Err("Working draft task or source kind differs from captured proposal");
        }
        Ok(
            if draft
                .validate(source, self.captured.document.generated.task)
                .is_ok()
            {
                WorkingState::Valid
            } else {
                WorkingState::NeedsInput
            },
        )
    }
}
impl ProjectRevision {
    pub fn derived_work_state(&self) -> ProjectWorkState {
        let mut state = ProjectWorkState::default();
        for proposal in &self.proposals {
            match proposal.captured.document.state {
                DraftState::NeedsRepair => state.needs_repair += 1,
                DraftState::Editable => state.editable += 1,
            }
            state.accepted_output += u32::from(proposal.captured.acceptance.is_some());
            state.working_needs_input +=
                u32::from(proposal.working_state == WorkingState::NeedsInput);
        }
        state
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if !matches!(self.schema_version.as_str(), "1" | "2")
            || !valid_id(&self.project_id)
            || !text(&self.title, 1024)
            || self.revision.0 == 0
            || self.parent_revision.map(|n| n.0)
                != self.revision.0.checked_sub(1).filter(|n| *n > 0)
            || (self.watching.is_none() && self.proposals.is_empty())
            || self.proposals.len() > 32
        {
            return Err("Invalid project identity, revision, or empty context");
        }
        if let Some(watching) = &self.watching {
            watching.validate()?;
        }
        let mut ids = BTreeSet::new();
        let mut working_bytes = 0usize;
        for proposal in &self.proposals {
            if let Some(origin) = &proposal.proposal_origin {
                if self.schema_version != "2" {
                    return Err("Proposal origin requires project schema2");
                }
                origin.validate()?;
            }
            proposal.captured.validate()?;
            if !ids.insert(&proposal.captured.document.proposal_id)
                || canonical_sha256(&proposal.captured)? != proposal.capture_sha256
                || proposal.working_state()? != proposal.working_state
            {
                return Err("Invalid or duplicate captured proposal snapshot");
            }
            working_bytes = working_bytes
                .checked_add(
                    serde_json::to_vec(&proposal.working_draft)
                        .map_err(|_| "Invalid working draft")?
                        .len(),
                )
                .ok_or("Working draft size overflow")?;
        }
        if working_bytes > MAX_WORKING_BYTES
            || self.work_state != self.derived_work_state()
            || serde_json::to_vec(self)
                .map_err(|_| "Invalid project JSON")?
                .len()
                > MAX_RECORD_BYTES
        {
            return Err("Invalid project work state or excessive record size");
        }
        Ok(())
    }
}
