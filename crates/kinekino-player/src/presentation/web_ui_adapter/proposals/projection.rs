use kinekino_player::domain::{api_values::DecimalU64, proposals::*};
use meari::domain::Value;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceReference {
    Cue { cue_id: String },
    Document { source_ref: DocumentTextRef },
}

#[derive(Clone, Debug, Serialize)]
pub struct OriginalText {
    pub key: String,
    pub reference: SourceReference,
    pub text: String,
    pub start_us: Option<DecimalU64>,
    pub end_us: Option<DecimalU64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProposalView {
    pub proposal_id: String,
    pub document_revision: DecimalU64,
    pub source: SourceSnapshot,
    pub original: Vec<OriginalText>,
    pub task: ProposalTask,
    pub generated_raw: String,
    pub stored_draft: Option<ProposalPayload>,
    pub import_error: Option<String>,
    pub state: DraftState,
    pub acceptance: Option<AcceptanceReceipt>,
    pub historical_reference: Option<CompanionReference>,
    pub generation_provenance_bound: bool,
    pub quality_decision_known: bool,
}

/// Display projection only. Structured card/highlight drafts and receipt revisions stay canonical.
pub fn project(stored: &StoredProposal) -> Result<ProposalView, &'static str> {
    stored.validate()?;
    let document = &stored.document;
    let original = match &document.source {
        SourceSnapshot::Cue(source) => source
            .cues
            .iter()
            .map(|cue| OriginalText {
                key: format!("cue:{}", cue.cue_id),
                reference: SourceReference::Cue {
                    cue_id: cue.cue_id.clone(),
                },
                text: cue.text.clone(),
                start_us: cue.start_us,
                end_us: cue.end_us,
            })
            .collect(),
        SourceSnapshot::Document(source) => source
            .texts
            .iter()
            .map(|text| {
                let source_ref = source
                    .text_reference(&text.inner_pointer)
                    .ok_or("Document reference is unavailable")?;
                Ok(OriginalText {
                    key: format!("document:{}", text.inner_pointer),
                    reference: SourceReference::Document { source_ref },
                    text: text.text.clone(),
                    start_us: None,
                    end_us: None,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?,
    };
    Ok(ProposalView {
        proposal_id: document.proposal_id.clone(),
        document_revision: document.draft_revision,
        source: document.source.clone(),
        original,
        task: document.generated.task,
        generated_raw: document.generated.raw_text.clone(),
        stored_draft: document.draft.clone(),
        import_error: document.import_error.clone(),
        state: document.state,
        acceptance: stored.acceptance.clone(),
        historical_reference: stored
            .historical
            .as_ref()
            .map(|binding| binding.reference.clone()),
        // Stored references and structural validation do not verify evidence or output quality.
        generation_provenance_bound: false,
        quality_decision_known: false,
    })
}

/// Display only: no file access, canonical inspect, job lookup or evidence verification.
pub fn historical_source_props(reference: Option<&CompanionReference>) -> BTreeMap<String, Value> {
    let label = match reference {
        Some(reference) => format!(
            "SHA-256: {} · {} bytes · 항목: {}",
            reference.sha256, reference.bytes, reference.item_id
        ),
        None => "저장된 결합 참조 없음".into(),
    };
    BTreeMap::from([
        ("historicalReferenceLabel".into(), Value::String(label)),
        (
            "historicalVerificationLabel".into(),
            Value::String("저장된 참조 표시만 제공합니다. 증거 파일 검증·현재 작업·정밀도·라이선스·사람 품질 승인은 미확인입니다.".into()),
        ),
    ])
}
