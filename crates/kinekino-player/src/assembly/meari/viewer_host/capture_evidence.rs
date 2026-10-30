//! Parent consumption of acknowledged carriers through the existing immutable artifact port.
use super::*;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::{CaptureMetadata, CatalogArtifacts, CatalogCaptureKind, RawArtifact},
    },
    domain::api_values::DecimalU64,
    presentation::web_ui_adapter::proposals::capture_mutation::{
        CaptureExecutionAcknowledgement, CaptureExecutionDisposition,
    },
};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

struct Entry {
    original: CaptureExecutionAcknowledgement,
    prepared: Option<(Vec<u8>, CaptureMetadata)>,
}
pub(super) struct CaptureEvidenceOwner {
    artifacts: Arc<dyn CatalogArtifacts>,
    pending: VecDeque<Entry>,
    current_error: Option<Arc<ApiError>>,
    first_error: Option<Arc<ApiError>>,
    last_saved: Option<RawArtifact>,
    saved: u64,
}
pub struct CaptureEvidenceStatus {
    pub saved: u64,
    pub pending: usize,
    pub last_saved: Option<RawArtifact>,
    pub first_error: Option<Arc<ApiError>>,
}
impl CaptureEvidenceOwner {
    fn new(artifacts: Arc<dyn CatalogArtifacts>) -> Self {
        Self {
            artifacts,
            pending: VecDeque::new(),
            current_error: None,
            first_error: None,
            last_saved: None,
            saved: 0,
        }
    }
    fn accept(&mut self, original: Vec<CaptureExecutionAcknowledgement>) {
        self.pending
            .extend(original.into_iter().map(|original| Entry {
                original,
                prepared: None,
            }));
    }
    fn record(
        entry: &CaptureExecutionAcknowledgement,
    ) -> Result<(Vec<u8>, CaptureMetadata), ApiError> {
        let disposition = match &entry.disposition {
            CaptureExecutionDisposition::Consumed => serde_json::json!({"kind":"consumed"}),
            CaptureExecutionDisposition::Discarded { reason } => {
                serde_json::json!({"kind":"discarded","reason":reason})
            }
        };
        let terminal = entry.terminal.as_ref().map(|terminal| serde_json::json!({
            "request":terminal.request.as_ref(), "response":terminal.response.as_ref(),
            "cancellation_requested":terminal.cancellation_requested,"queued_cancelled":terminal.queued_cancelled,
            "encoding_measurement":format!("{:?}",terminal.measurement),
        }));
        let bytes = serde_json::to_vec(&serde_json::json!({
            "schema_version":"1","representation":"native_ui_sdk_execution_diagnostic",
            "request":entry.request,"terminal":terminal,"disposition":disposition,"first_fault":entry.first_fault.as_deref(),
            "operation_debug":entry.operation.as_ref().map(|operation|format!("{operation:?}")),
            "operation_debug_is_durable_identity":false,"acknowledged":true,
        })).map_err(|error| ApiError::new(ErrorCode::EngineFailed, format!("Viewer evidence serialization: {error}")))?;
        let observed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Viewer evidence clock is unavailable",
                )
            })?
            .as_millis()
            .to_string();
        let metadata = CaptureMetadata {
            requested_url: format!("sdk://local/{}", entry.request.command),
            final_url: format!("sdk://local/{}", entry.request.command),
            redirects: Vec::new(),
            observed_at: observed,
            parser_revision: "com.kinekino.player.ui-execution.v1".into(),
            capture_kind: CatalogCaptureKind::NativeEngineDiagnostic,
            status: None,
            content_type: "application/json".into(),
            content_encoding: None,
            body_sha256: format!("{:x}", Sha256::digest(&bytes)),
            body_bytes: DecimalU64(bytes.len() as u64),
            browser: None,
        };
        Ok((bytes, metadata))
    }
    fn flush(&mut self, retry: bool) -> Result<(), Arc<ApiError>> {
        if !retry && let Some(error) = &self.current_error {
            return Err(error.clone());
        }
        while let Some(entry) = self.pending.front_mut() {
            let attempt = (|| {
                let next = self.saved.checked_add(1).ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Viewer evidence counter exhausted",
                    )
                })?;
                if entry.prepared.is_none() {
                    entry.prepared = Some(Self::record(&entry.original)?);
                }
                let (bytes, metadata) = entry
                    .prepared
                    .as_ref()
                    .expect("original entry retains prepared bytes");
                let artifact = self.artifacts.save(bytes, metadata)?;
                if artifact.body_sha256 != metadata.body_sha256
                    || artifact.body_bytes != metadata.body_bytes
                {
                    return Err(ApiError::new(
                        ErrorCode::SourceExpired,
                        "Viewer evidence storage returned a different body identity",
                    ));
                }
                Ok((artifact, next))
            })();
            match attempt {
                Ok((artifact, next)) => {
                    self.last_saved = Some(artifact);
                    self.saved = next;
                    self.pending.pop_front(); // Durable immutable publication now owns the bytes.
                    self.current_error = None;
                }
                Err(error) => {
                    let error = Arc::new(error);
                    self.first_error.get_or_insert_with(|| error.clone());
                    self.current_error = Some(error.clone());
                    return Err(error);
                }
            }
        }
        Ok(())
    }
}
impl cards::CardRoute {
    pub(super) fn persist_capture_evidence(
        &mut self,
        force_retry: bool,
    ) -> Result<(), Arc<ApiError>> {
        let Some(owner) = &mut self.capture_evidence else {
            return Ok(());
        };
        let requested = self
            .capture_surface
            .as_mut()
            .is_some_and(|surface| std::mem::take(&mut surface.evidence_retry));
        let retry = force_retry || requested;
        if owner.pending.is_empty() {
            if let Some(surface) = &mut self.capture_surface {
                owner.accept(surface.take_acknowledged());
            }
            if let Some(receipt) = self.capture_close_receipt.take() {
                owner.accept(vec![receipt]);
            }
        }
        let result = loop {
            if let Err(error) = owner.flush(retry) {
                break Err(error);
            }
            // A retained close may also own a carrier behind the recovered batch.
            // Transfer all already-acknowledged evidence before reporting publication complete.
            if let Some(surface) = &mut self.capture_surface {
                owner.accept(surface.take_acknowledged());
            }
            if let Some(receipt) = self.capture_close_receipt.take() {
                owner.accept(vec![receipt]);
            }
            if owner.pending.is_empty() {
                break Ok(());
            }
        };
        if let Some(surface) = &mut self.capture_surface {
            let blocked = result.is_err() || !owner.pending.is_empty();
            surface.dirty |= surface.evidence_blocked != blocked;
            surface.evidence_blocked = blocked;
            if let Err(error) = &result {
                surface
                    .error
                    .get_or_insert_with(|| ViewerHostError::Sdk(error.clone()));
            }
        }
        result
    }
}
impl ViewerHost {
    pub fn install_capture_evidence(
        &mut self,
        artifacts: Arc<dyn CatalogArtifacts>,
    ) -> Result<(), ViewerHostError> {
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        if self.shutdown_started || cards.capture_evidence.is_some() || cards.capture.is_some() {
            return Err(ViewerHostError::UnsupportedCallback);
        }
        cards.capture_evidence = Some(CaptureEvidenceOwner::new(artifacts));
        Ok(())
    }
    pub fn captured_evidence_status(&self) -> Option<CaptureEvidenceStatus> {
        let cards = self.cards.try_borrow().ok()?;
        let owner = cards.capture_evidence.as_ref()?;
        Some(CaptureEvidenceStatus {
            saved: owner.saved,
            pending: owner.pending.len(),
            last_saved: owner.last_saved.clone(),
            first_error: owner.first_error.clone(),
        })
    }
    /// Trusted recovery during retained close; retries only the original storage publication.
    pub fn retry_captured_evidence(&mut self) -> Result<(), ViewerHostError> {
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        if cards.capture_evidence.is_none() {
            return Err(ViewerHostError::UnsupportedCallback);
        }
        cards
            .persist_capture_evidence(true)
            .map_err(ViewerHostError::Sdk)
    }
}
