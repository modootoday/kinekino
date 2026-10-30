use super::*;
use crate::domain::identifiers::{valid_id, valid_sha256};
use std::collections::BTreeSet;

fn text(value: &str, limit: usize) -> bool {
    value.len() <= limit
        && !value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
}

impl CardDocument {
    /// Valid draft, not render readiness, visual approval, or accepted model output.
    pub fn validate(&self) -> Result<(), CardDocumentError> {
        use CardDocumentError::*;
        if self.schema_version != "1" || !valid_id(&self.document_id) || self.revision.0 == 0 {
            return Err(InvalidDocument);
        }
        self.source.validate().map_err(|_| InvalidReference)?;
        let canvas = &self.layout.canvas;
        if canvas.width == 0
            || canvas.height == 0
            || canvas.width > 4096
            || canvas.height > 4096
            || !valid_id(&canvas.template_id)
            || !text(&canvas.template_version, 128)
            || canvas.template_version.trim().is_empty()
            || !valid_sha256(&canvas.template_sha256)
            || !text(&self.layout.caption_draft, 16_384)
        {
            return Err(InvalidDocument);
        }
        if self.layout.cards.len() > MAX_CARDS {
            return Err(LimitExceeded);
        }
        let mut ids = BTreeSet::new();
        for entry in &self.layout.cards {
            if !valid_id(&entry.card_id)
                || !ids.insert(&entry.card_id)
                || !text(&entry.content.front, 8192)
                || !text(&entry.content.back, 16_384)
                || !text(&entry.alt_text_draft, 8192)
            {
                return Err(InvalidDocument);
            }
            let refs = &entry.content.source_refs;
            let unique: BTreeSet<_> = refs.iter().collect();
            if refs.len() > 4096
                || refs.len() != unique.len()
                || refs
                    .iter()
                    .any(|id| !self.source.cues.iter().any(|cue| &cue.cue_id == id))
                || (entry.text_kind != CardTextKind::Manual && refs.is_empty())
            {
                return Err(InvalidReference);
            }
            if entry.text_kind == CardTextKind::Quotation {
                let original = refs
                    .iter()
                    .map(|id| {
                        self.source
                            .cues
                            .iter()
                            .find(|cue| &cue.cue_id == id)
                            .expect("validated reference")
                            .text
                            .as_str()
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if entry.content.back != original {
                    return Err(InvalidQuotation);
                }
            }
            if let Some(frame) = &entry.frame
                && (!valid_id(&frame.artifact_id)
                    || !valid_sha256(&frame.png_sha256)
                    || frame.source_sha256 != self.source.source_sha256)
            {
                return Err(InvalidReference);
            }
        }
        let bytes = serde_json::to_vec(self).map_err(|_| InvalidDocument)?;
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(LimitExceeded);
        }
        Ok(())
    }
}
