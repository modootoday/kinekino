use super::*;
use crate::application::{
    api::ErrorCode,
    catalog::{
        BrowserCaptureProvenance, BrowserFrameCaptureProvenance, BrowserNetworkCaptureProvenance,
        CaptureMetadata, CatalogArtifacts, CatalogCaptureKind, RawArtifact,
    },
};
use sha2::{Digest, Sha256};

pub(super) struct BrowserEvidenceArchive {
    artifacts: Arc<dyn CatalogArtifacts>,
    clock: Arc<dyn BrowserEvidenceClock>,
}
pub trait BrowserEvidenceClock: Send + Sync {
    fn now_ms(&self) -> DecimalU64;
}
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ObservationEndReason {
    ManifestVerified,
    ResponseRejected,
    ProviderFailed,
    ExecutionDeadline,
    Cancelled,
}
impl BrowserEvidenceArchive {
    pub fn new(artifacts: Arc<dyn CatalogArtifacts>, clock: Arc<dyn BrowserEvidenceClock>) -> Self {
        Self { artifacts, clock }
    }
    pub fn document(
        &self,
        request: &ValidatedBrowserRequest,
        snapshot: &BrowserSnapshot,
    ) -> Result<RawArtifact, ApiError> {
        let document = snapshot
            .document
            .as_deref()
            .ok_or_else(|| failed("Browser document was not observed"))?;
        if document.kind != CaptureKind::DomSnapshot
            || document.response.is_some()
            || document.document_generation != snapshot.document_generation
            || snapshot.current_url.as_deref() != Some(document.final_url.as_str())
            || !request.navigation_allowed(&document.final_url)
        {
            return Err(failed(
                "Browser document evidence does not match the owned work",
            ));
        }
        self.save(
            request,
            snapshot,
            &document.body,
            CatalogCaptureKind::DomSnapshot,
            "text/html; charset=utf-8",
            document.observed_unix_ms,
        )
    }
    pub fn trace_ended(
        &self,
        request: &ValidatedBrowserRequest,
        snapshot: &BrowserSnapshot,
        end: ObservationEndReason,
    ) -> Result<RawArtifact, ApiError> {
        let frames = snapshot.frame_documents.iter().map(|frame| serde_json::json!({"identity":frame.identity,"completion":frame.completion,"destroyed":frame.destroyed,"superseded":frame.superseded,"document_observed":frame.document.is_some(),"runtime_observations":frame.runtime.observations(),"runtime_pending":frame.runtime.pending(),"runtime_sampling_finished":frame.runtime.finished()})).collect::<Vec<_>>();
        let network = snapshot.network.as_ref().map(|network| serde_json::json!({"events":network.events,"requests":network.requests,"body_count":network.bodies.len(),"unbound_session_events":network.unbound_session_events,"unavailable_bodies":network.unavailable_bodies,"unavailable_captures":network.unavailable_captures,"unscoped_requests":network.unscoped_requests}));
        let bytes=serde_json::to_vec(&serde_json::json!({"schema_version":"1","representation":"browser_event_trace","observation_end":end,"provider_state":snapshot.state,"activation":snapshot.activation,"activation_history":snapshot.activation_history,"main_navigation":snapshot.navigation_diagnostic,"player_candidates":snapshot.player_frames,"media_requests":snapshot.media_requests,"frame_documents":frames,"network":network,"candidate_is_media":false})).map_err(|_|failed("Browser event evidence serialization failed"))?;
        let time = self.clock.now_ms();
        self.save(
            request,
            snapshot,
            &bytes,
            CatalogCaptureKind::BrowserEventTrace,
            "application/json",
            time,
        )
    }
    pub fn resolution_metadata(
        &self,
        request: &ValidatedBrowserRequest,
        snapshot: &BrowserSnapshot,
        bytes: &[u8],
    ) -> Result<crate::application::catalog::CaptureMetadata, ApiError> {
        self.metadata(
            request,
            snapshot,
            bytes,
            CatalogCaptureKind::EpisodeResolution,
            "application/json",
            self.clock.now_ms(),
        )
    }
    pub fn frame_document(
        &self,
        request: &ValidatedBrowserRequest,
        snapshot: &BrowserSnapshot,
        frame: &FrameDocumentState,
    ) -> Result<RawArtifact, ApiError> {
        let document = frame
            .document
            .as_deref()
            .ok_or_else(|| failed("Child DOM was not observed"))?;
        let completion = frame
            .completion
            .clone()
            .ok_or_else(|| failed("Child completion was not observed"))?;
        if frame.identity.owner_document_generation != snapshot.document_generation
            || Some(frame.identity.owner_native_navigation_id) != snapshot.native_navigation_id
            || frame.identity.document_generation != document.document_generation
            || frame.identity.url != document.final_url
            || document.kind != CaptureKind::DomSnapshot
            || document.response.is_some()
            || !crate::domain::navigation::player_page(&document.final_url)
            || !request.navigation_allowed(&document.final_url)
        {
            return Err(failed("Child DOM provenance does not match its owner"));
        }
        let mut metadata = self.metadata(
            request,
            snapshot,
            &document.body,
            CatalogCaptureKind::FrameDomSnapshot,
            "text/html; charset=utf-8",
            document.observed_unix_ms,
        )?;
        metadata.final_url.clone_from(&document.final_url);
        metadata
            .browser
            .as_mut()
            .ok_or_else(|| failed("Child capture owner is unavailable"))?
            .frame = Some(BrowserFrameCaptureProvenance {
            identity: frame.identity.clone(),
            completion,
        });
        self.artifacts.save(&document.body, &metadata)
    }
    pub fn network_body(
        &self,
        request: &ValidatedBrowserRequest,
        snapshot: &BrowserSnapshot,
        body: &crate::application::browser_network::TimedResponseBody,
    ) -> Result<RawArtifact, ApiError> {
        let capture = &body.body.capture;
        let response = capture.response();
        if !(100..=599).contains(&response.status)
            || !crate::domain::navigation::media_transport_allowed(&response.url)
        {
            return Err(failed(
                "Owned response body is outside its admitted transport",
            ));
        }
        let mut metadata = self.metadata(
            request,
            snapshot,
            &body.body.bytes,
            CatalogCaptureKind::BrowserResponseBody,
            &response.mime_type,
            body.observed_unix_ms,
        )?;
        metadata.final_url.clone_from(&response.url);
        metadata.status = Some(response.status);
        metadata.parser_revision = "owned-cdp-browser-response-v1".into();
        metadata
            .browser
            .as_mut()
            .ok_or_else(|| failed("Owned response capture owner is unavailable"))?
            .network = Some(BrowserNetworkCaptureProvenance {
            request_headers: capture.request_headers().cloned(),
            request_method: Some(capture.request_method().into()),
            resource_type: capture.resource_type().map(Into::into),
            purpose: Some(capture.purpose().clone()),
            session: body.session.clone(),
            scope: capture.scope().clone(),
            request_id: capture.request_id().into(),
            attempt: DecimalU64(capture.attempt()),
            response: response.clone(),
            referer: capture.referer().map(Into::into),
            finished_encoded_bytes: DecimalU64(capture.finished_encoded_bytes()),
            owner_context_at_body_observation: body.owner_context.clone(),
        });
        self.artifacts.save(&body.body.bytes, &metadata)
    }
    fn save(
        &self,
        request: &ValidatedBrowserRequest,
        snapshot: &BrowserSnapshot,
        bytes: &[u8],
        kind: CatalogCaptureKind,
        content_type: &str,
        time: DecimalU64,
    ) -> Result<RawArtifact, ApiError> {
        let metadata = self.metadata(request, snapshot, bytes, kind, content_type, time)?;
        self.artifacts.save(bytes, &metadata)
    }
    fn metadata(
        &self,
        request: &ValidatedBrowserRequest,
        snapshot: &BrowserSnapshot,
        bytes: &[u8],
        kind: CatalogCaptureKind,
        content_type: &str,
        time: DecimalU64,
    ) -> Result<CaptureMetadata, ApiError> {
        if bytes.len() > MAX_CAPTURE_BYTES
            || snapshot.activation_history.len() > 2
            || snapshot.media_requests.len() > 16
            || snapshot.frame_documents.len() > 16
            || snapshot.ticket.request_id() != request.request().request_id
            || snapshot.document_generation.0 == 0
            || snapshot.blocked_count > snapshot.requested_count
            || snapshot.blocking_requested && !snapshot.blocking_installed
        {
            return Err(failed("Invalid browser evidence bounds"));
        }
        let final_url = snapshot
            .current_url
            .clone()
            .ok_or_else(|| failed("Browser evidence has no document identity"))?;
        if snapshot.navigation_history.len() > MAX_NAVIGATIONS
            || snapshot.navigation_history.last().is_none_or(|navigation| {
                navigation.document_generation != snapshot.document_generation
                    || navigation.url != final_url
            })
            || snapshot
                .navigation_diagnostic
                .as_ref()
                .is_some_and(|diagnostic| {
                    diagnostic.document_generation != snapshot.document_generation
                        || Some(diagnostic.native_navigation_id) != snapshot.native_navigation_id
                })
        {
            return Err(failed(
                "Browser evidence navigation identity is inconsistent",
            ));
        }
        if !request.navigation_allowed(&final_url) {
            return Err(failed("Browser evidence left its admitted scope"));
        }
        let observed_at =
            time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(time.0) * 1_000_000)
                .map_err(|_| failed("Invalid browser observation time"))?
                .format(&time::format_description::well_known::Rfc3339)
                .map_err(|_| failed("Invalid browser observation time"))?;
        let metadata = CaptureMetadata {
            requested_url: request.request().url.clone(),
            final_url,
            redirects: vec![],
            observed_at,
            parser_revision: "episode-resolution-evidence-v1".into(),
            capture_kind: kind,
            status: None,
            content_type: content_type.into(),
            content_encoding: None,
            body_sha256: format!("{:x}", Sha256::digest(bytes)),
            body_bytes: DecimalU64(bytes.len() as u64),
            browser: Some(BrowserCaptureProvenance {
                network: None,
                frame: None,
                navigation_diagnostic: snapshot.navigation_diagnostic.clone(),
                ticket: snapshot.ticket.clone(),
                document_generation: snapshot.document_generation,
                navigation_history: snapshot.navigation_history.clone(),
                engine_version: snapshot
                    .engine_version
                    .clone()
                    .ok_or_else(|| failed("Browser engine identity was not observed"))?,
                blocking_requested: snapshot.blocking_requested,
                blocking_installed: snapshot.blocking_installed,
                requested_count: snapshot.requested_count,
                blocked_count: snapshot.blocked_count,
            }),
        };
        Ok(metadata)
    }
}
fn failed(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    #[derive(Default)]
    struct Store(Mutex<Option<(Vec<u8>, CaptureMetadata)>>);
    impl CatalogArtifacts for Store {
        fn save(&self, bytes: &[u8], metadata: &CaptureMetadata) -> Result<RawArtifact, ApiError> {
            *self.0.lock().unwrap() = Some((bytes.into(), metadata.clone()));
            let artifact_id = format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(metadata).unwrap())
            );
            Ok(RawArtifact {
                artifact_id: artifact_id.clone(),
                body_sha256: metadata.body_sha256.clone(),
                body_bytes: metadata.body_bytes,
                body_path: format!("objects/{}.body", metadata.body_sha256),
                metadata_path: format!("captures/{artifact_id}.json"),
            })
        }
    }
    struct Clock;
    impl BrowserEvidenceClock for Clock {
        fn now_ms(&self) -> DecimalU64 {
            DecimalU64(42)
        }
    }
    #[test]
    fn event_trace_has_its_own_representation_and_never_fabricates_dom_http_status() {
        let (_, work) = super::super::activation_tests::episode();
        let store = Arc::new(Store::default());
        let archive = BrowserEvidenceArchive::new(store.clone(), Arc::new(Clock));
        archive
            .trace_ended(
                work.request(),
                &work.snapshot(),
                ObservationEndReason::ResponseRejected,
            )
            .unwrap();
        let saved = store.0.lock().unwrap();
        let (bytes, metadata) = saved.as_ref().unwrap();
        assert!(matches!(
            metadata.capture_kind,
            CatalogCaptureKind::BrowserEventTrace
        ));
        assert!(metadata.status.is_none());
        assert_eq!(metadata.content_type, "application/json");
        assert_eq!(metadata.body_sha256, format!("{:x}", Sha256::digest(bytes)));
        let trace: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(trace["representation"], "browser_event_trace");
        assert_eq!(trace["candidate_is_media"], false);
        assert_eq!(metadata.observed_at, "1970-01-01T00:00:00.042Z");
    }
    #[test]
    fn observed_response_uses_actual_status_and_separate_protocol_provenance() {
        let (ticket, mut work) = super::super::network_tests::prepared();
        super::super::network_tests::complete_parent(&ticket, &mut work);
        let read = work.take_network_body().unwrap();
        work.apply(
            &ticket,
            BrowserEvent::NetworkBody {
                read,
                json: r#"{"body":"fixture body","base64Encoded":false}"#.into(),
                observed_unix_ms: DecimalU64(8),
            },
        );
        let snapshot = work.snapshot();
        let body = &snapshot.network.as_ref().unwrap().bodies[0];
        let store = Arc::new(Store::default());
        BrowserEvidenceArchive::new(store.clone(), Arc::new(Clock))
            .network_body(work.request(), &snapshot, body)
            .unwrap();
        let saved = store.0.lock().unwrap();
        let (bytes, metadata) = saved.as_ref().unwrap();
        assert!(matches!(
            metadata.capture_kind,
            CatalogCaptureKind::BrowserResponseBody
        ));
        assert_eq!(metadata.status, Some(200));
        assert_eq!(bytes.as_slice(), b"fixture body");
        assert_eq!(metadata.body_sha256, format!("{:x}", Sha256::digest(bytes)));
        let encoded = serde_json::to_vec(metadata).unwrap();
        let restored: CaptureMetadata = serde_json::from_slice(&encoded).unwrap();
        let browser = restored.browser.unwrap();
        assert!(browser.frame.is_none());
        let network = browser.network.unwrap();
        assert_eq!(network.scope.frame_id, "protocol-child");
        assert_eq!(network.scope.loader_id, "child-loader");
        assert_eq!(
            network
                .owner_context_at_body_observation
                .document_generation,
            DecimalU64(2)
        );
        assert_eq!(network.finished_encoded_bytes, DecimalU64(12));
    }
}
