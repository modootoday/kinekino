use super::*;
use crate::domain::model_review::{valid_id, valid_sha256};
use std::collections::BTreeSet;
fn text(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 65_536 && !s.contains('\0')
}
impl CueSourceSnapshot {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !valid_id(&self.source_id)
            || !valid_sha256(&self.source_sha256)
            || self.cues.is_empty()
            || self.cues.len() > 4096
        {
            return Err("Invalid source snapshot identity or cue count");
        }
        let mut ids = BTreeSet::new();
        for cue in &self.cues {
            if !valid_id(&cue.cue_id)
                || !ids.insert(&cue.cue_id)
                || !text(&cue.text)
                || !matches!((cue.start_us, cue.end_us), (None, None))
                    && !matches!((cue.start_us, cue.end_us),(Some(a),Some(b)) if a.0 < b.0)
            {
                return Err("Invalid or duplicate source cue; time must be a verified pair");
            }
        }
        Ok(())
    }
}
impl ProposalPayload {
    pub fn validate(
        &self,
        source: &SourceSnapshot,
        task: ProposalTask,
    ) -> Result<(), &'static str> {
        if self.task() != task {
            return Err("Draft task differs from generated task");
        }
        let ids: BTreeSet<_> = source.cues().iter().map(|c| c.cue_id.as_str()).collect();
        let refs = |r: &[String]| {
            let set: BTreeSet<_> = r.iter().collect();
            !r.is_empty()
                && r.len() <= 4096
                && set.len() == r.len()
                && r.iter().all(|v| ids.contains(v.as_str()))
        };
        let valid = match self {
            Self::DocumentCorrection { edits } => {
                let unique: BTreeSet<_> = edits.iter().map(|e| &e.source_ref).collect();
                !edits.is_empty()
                    && edits.len() <= 4096
                    && unique.len() == edits.len()
                    && source.as_document().is_some_and(|s| {
                        edits
                            .iter()
                            .all(|e| s.contains_reference(&e.source_ref) && text(&e.text))
                    })
            }
            Self::Correction { edits } => {
                let unique: BTreeSet<_> = edits.iter().map(|e| &e.cue_id).collect();
                !edits.is_empty()
                    && edits.len() <= 4096
                    && unique.len() == edits.len()
                    && edits
                        .iter()
                        .all(|e| ids.contains(e.cue_id.as_str()) && text(&e.text))
            }
            Self::Cards { cards } => {
                !cards.is_empty()
                    && cards.len() <= 4096
                    && cards
                        .iter()
                        .all(|c| text(&c.front) && text(&c.back) && refs(&c.source_refs))
            }
            Self::Highlights { highlights } => {
                !highlights.is_empty()
                    && highlights.len() <= 4096
                    && highlights
                        .iter()
                        .all(|h| text(&h.title) && refs(&h.source_refs))
            }
        };
        if valid {
            Ok(())
        } else {
            Err("Draft requires bounded text and unique references to existing source cues")
        }
    }
}
