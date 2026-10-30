use super::{MAX_DOCUMENT_BYTES, ProposalTask, request_source_payload};
use crate::domain::{
    api_values::DecimalU64,
    model_review::{valid_id, valid_sha256},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const OUTER_POINTER: &str = "/messages/0/content";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceExtraction {
    AsrRequestAfterNoThinkLfV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSelector {
    pub outer_pointer: String,
    pub extraction: SourceExtraction,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentText {
    pub inner_pointer: String,
    pub text: String,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentTextRef {
    pub source_asset_sha256: String,
    pub outer_pointer: String,
    pub extraction: SourceExtraction,
    pub inner_pointer: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSourceSnapshot {
    pub schema_version: String,
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub source_sha256: String,
    pub asset_utf8: String,
    pub selector: DocumentSelector,
    pub texts: Vec<DocumentText>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentCorrection {
    pub source_ref: DocumentTextRef,
    pub text: String,
}

fn decode_asr_request(asset: &str) -> Result<Vec<DocumentText>, &'static str> {
    let suffix = request_source_payload(asset, ProposalTask::Correction)?;
    let strings: Vec<String> = serde_json::from_str(&suffix)
        .map_err(|_| "ASR source suffix must be one complete JSON string array")?;
    if strings.is_empty()
        || strings.len() > 4096
        || strings
            .iter()
            .any(|s| s.trim().is_empty() || s.contains('\0'))
    {
        return Err("Invalid ASR source text array");
    }
    Ok(strings
        .into_iter()
        .enumerate()
        .map(|(i, text)| DocumentText {
            inner_pointer: format!("/{i}"),
            text,
        })
        .collect())
}
impl DocumentSourceSnapshot {
    pub fn from_asset(
        source_id: String,
        source_revision: DecimalU64,
        source_sha256: String,
        asset_utf8: String,
    ) -> Result<Self, &'static str> {
        let texts = decode_asr_request(&asset_utf8)?;
        let snapshot = Self {
            schema_version: "2".into(),
            source_id,
            source_revision,
            source_sha256,
            asset_utf8,
            selector: DocumentSelector {
                outer_pointer: OUTER_POINTER.into(),
                extraction: SourceExtraction::AsrRequestAfterNoThinkLfV1,
            },
            texts,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "2"
            || !valid_id(&self.source_id)
            || !valid_sha256(&self.source_sha256)
            || self.asset_utf8.len() > MAX_DOCUMENT_BYTES
            || self.selector.outer_pointer != OUTER_POINTER
        {
            return Err("Invalid document source identity, selector or bound");
        }
        let actual = format!("{:x}", Sha256::digest(self.asset_utf8.as_bytes()));
        if actual != self.source_sha256 {
            return Err("Document source bytes differ from source asset SHA256");
        }
        if self.texts != decode_asr_request(&self.asset_utf8)? {
            return Err("Document text pointers/order/content differ from the actual source array");
        }
        Ok(())
    }
    pub fn text_reference(&self, inner_pointer: &str) -> Option<DocumentTextRef> {
        self.texts
            .iter()
            .any(|t| t.inner_pointer == inner_pointer)
            .then(|| DocumentTextRef {
                source_asset_sha256: self.source_sha256.clone(),
                outer_pointer: self.selector.outer_pointer.clone(),
                extraction: self.selector.extraction,
                inner_pointer: inner_pointer.into(),
            })
    }
    pub fn contains_reference(&self, reference: &DocumentTextRef) -> bool {
        self.text_reference(&reference.inner_pointer).as_ref() == Some(reference)
    }
}
