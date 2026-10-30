use super::{
    document_session::{BrowserDocumentSession, check},
    evidence_archive::{BrowserEvidenceArchive, ObservationEndReason},
    *,
};
use crate::application::{
    api::ErrorCode,
    catalog::{CatalogArtifacts, RawArtifact},
    episode_resolve::{
        EpisodeMarkupPort, EpisodeResolutionResults, ResolveEpisode, ResolvedManifestRecord,
        StoredEpisodeResolution, observed_manifest, observed_response_status, player_target,
    },
    jobs::{ExecutionControl, JobExecutor, SourceBlobStore, VerifiedJobOutput},
};
use kinekino_jobs::domain::PreparedJobInput;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Executes owned episode activation and preserves evidence. No unverified media is published.
pub struct BrowserEpisodeResolveJobs {
    port: Arc<dyn BrowserProviderPort>,
    owner: Arc<dyn BrowserProviderOwner>,
    archive: BrowserEvidenceArchive,
    markup: Arc<dyn EpisodeMarkupPort>,
    results: Arc<dyn EpisodeResolutionResults>,
}
impl BrowserEpisodeResolveJobs {
    pub fn new(
        port: Arc<dyn BrowserProviderPort>,
        owner: Arc<dyn BrowserProviderOwner>,
        artifacts: Arc<dyn CatalogArtifacts>,
        markup: Arc<dyn EpisodeMarkupPort>,
        clock: Arc<dyn BrowserEvidenceClock>,
        results: Arc<dyn EpisodeResolutionResults>,
    ) -> Self {
        Self {
            port,
            owner,
            archive: BrowserEvidenceArchive::new(artifacts, clock),
            markup,
            results,
        }
    }
    fn run(
        &self,
        input: &PreparedJobInput,
        control: &dyn ExecutionControl,
        artifacts: &mut Vec<RawArtifact>,
    ) -> Result<VerifiedJobOutput, ApiError> {
        let request_input: ResolveEpisode = serde_json::from_value(input.parameters.clone())
            .map_err(|_| invalid("Invalid episode resolution parameters"))?;
        let episode = request_input.location()?;
        check(control)?;
        let timeout = control
            .deadline()
            .saturating_duration_since(Instant::now())
            .as_millis()
            .clamp(1, 120_000) as u64;
        let request = ValidatedBrowserRequest::new(BrowserRequest {
            request_id: format!("episode-{}", &input.source.sha256[..16]),
            operation: BrowserOperation::ResolveEpisode,
            url: episode.as_str().into(),
            timeout_ms: DecimalU64(timeout),
            blocking_enabled: true,
        })
        .map_err(invalid)?;
        let session = BrowserDocumentSession::start(
            self.port.clone(),
            self.owner.as_ref(),
            request.clone(),
            control,
        )?;
        let initial = session.until(control, |snapshot| snapshot.document.is_some())?;
        artifacts.push(self.archive.document(&request, &initial)?);
        let document = initial
            .document
            .as_deref()
            .ok_or_else(|| invalid("Episode document was not retained"))?;
        // A declared iframe target is identity evidence, not a replacement for site watch behavior.
        let target = player_target(&request_input, document, self.markup.as_ref())?;
        let mut saved_parent_generation = None;
        let outcome = (|| {
            session.activate(initial.document_generation.0, control)?;
            let parent = session.until(control, |snapshot| {
                player_document_ready(snapshot, initial.document_generation)
            })?;
            artifacts.push(self.archive.document(&request, &parent)?);
            saved_parent_generation = Some(parent.document_generation);
            session.until(control, |snapshot| {
                child_document_ready(snapshot, target.document.as_str())
            })?;
            session.until(control, |snapshot| {
                let bootstrap = observed_response_status(
                    snapshot,
                    target.document.as_str(),
                    &crate::application::browser_network::ResponsePurpose::PlayerBootstrap,
                );
                if player_asset_status(snapshot, target.document.as_str())
                    .is_some_and(|status| !(200..300).contains(&status))
                    || bootstrap.is_some_and(|status| !(200..300).contains(&status))
                    || required_body_unavailable(snapshot, target.document.as_str())
                {
                    return true;
                }
                match observed_manifest(snapshot, target.document.as_str()) {
                    Err(_) => true,
                    Ok(Some(_)) => resolution_prerequisites(snapshot, target.document.as_str()),
                    Ok(None) => false,
                }
            })
        })();
        let observed = match &outcome {
            Ok(snapshot) => Some(snapshot.clone()),
            Err(_) => session.port.poll(&session.ticket).ok(),
        };
        let selected = observed.as_ref().and_then(|snapshot| {
            observed_manifest(snapshot, target.document.as_str())
                .ok()
                .flatten()
        });
        let mut manifest_artifact = None;
        if let Some(snapshot) = &observed {
            if snapshot.document.as_deref().is_some_and(|document| {
                document.document_generation != initial.document_generation
                    && Some(document.document_generation) != saved_parent_generation
            }) {
                artifacts.push(self.archive.document(&request, snapshot)?);
            }
            for frame in &snapshot.frame_documents {
                if frame.document.is_some() {
                    artifacts.push(self.archive.frame_document(&request, snapshot, frame)?);
                }
            }
            if let Some(network) = &snapshot.network {
                for body in &network.bodies {
                    let artifact = self.archive.network_body(&request, snapshot, body)?;
                    if selected
                        .as_ref()
                        .is_some_and(|selected| selected.body.body.capture == body.body.capture)
                    {
                        manifest_artifact = Some(artifact.clone());
                    }
                    artifacts.push(artifact);
                }
            }
            let end = match &outcome {
                Err(error) if error.code == ErrorCode::Timeout => {
                    ObservationEndReason::ExecutionDeadline
                }
                Err(error) if error.code == ErrorCode::Cancelled => ObservationEndReason::Cancelled,
                Err(_) => ObservationEndReason::ProviderFailed,
                Ok(_)
                    if selected.is_some()
                        && resolution_prerequisites(snapshot, target.document.as_str()) =>
                {
                    ObservationEndReason::ManifestVerified
                }
                Ok(_) => ObservationEndReason::ResponseRejected,
            };
            artifacts.push(self.archive.trace_ended(&request, snapshot, end)?);
        }
        let snapshot = outcome?;
        let selected = observed_manifest(&snapshot, target.document.as_str())?;
        if let Some(selected) =
            selected.filter(|_| resolution_prerequisites(&snapshot, target.document.as_str()))
        {
            let stored = StoredEpisodeResolution {
                schema_version: "2".into(),
                source_descriptor_sha256: input.source.sha256.clone(),
                episode_url: episode.as_str().into(),
                player_document_url: target.document.as_str().into(),
                manifest: ResolvedManifestRecord {
                    validation_profile: crate::domain::hls::HLS_VALIDATION_PROFILE.into(),
                    url: selected.source.url,
                    referer: selected.source.referer,
                    summary: selected.manifest.summary().clone(),
                    artifact: manifest_artifact
                        .ok_or_else(|| invalid("Verified manifest artifact was not retained"))?,
                },
                raw_artifacts: artifacts.clone(),
            };
            check(control)?;
            let bytes = serde_json::to_vec(&stored)
                .map_err(|_| invalid("Episode resolution serialization failed"))?;
            let metadata = self
                .archive
                .resolution_metadata(&request, &snapshot, &bytes)?;
            control.reserve_publication()?;
            control.begin_verification()?;
            return self.results.publish(&stored, &metadata);
        }
        let mut error = ApiError::new(
            ErrorCode::NeedsInput,
            "Episode watch evidence was observed; verified media extraction is not yet available",
        );
        let asset_status = player_asset_status(&snapshot, target.document.as_str());
        let asset_http_success = asset_status.is_some_and(|status| (200..300).contains(&status));
        let stage = if asset_http_success {
            if required_body_unavailable(&snapshot, target.document.as_str()) {
                "player_required_response_body_unavailable"
            } else {
                "player_bootstrap_response_failed"
            }
        } else if asset_status.is_some() {
            "player_asset_response_failed"
        } else {
            "player_asset_body_unavailable"
        };
        let runtime_observed = frame_runtime::current_finished(&snapshot, target.document.as_str());
        let runtime_sample_count = snapshot
            .frame_documents
            .iter()
            .filter(|frame| {
                !frame.destroyed
                    && !frame.superseded
                    && frame.identity.url == target.document.as_str()
                    && frame.identity.owner_document_generation == snapshot.document_generation
                    && Some(frame.identity.owner_native_navigation_id)
                        == snapshot.native_navigation_id
            })
            .map(|frame| frame.runtime.observations().len())
            .sum::<usize>();
        error.details = json!({"stage":stage,"player_document_observed":true,"frame_document_observed":true,"player_asset_observed":asset_status.is_some(),"player_asset_http_success":asset_http_success,"player_asset_status":asset_status,"runtime_observed":runtime_observed,"runtime_sample_count":runtime_sample_count,"frame_document_count":snapshot.frame_documents.iter().filter(|frame|frame.document.is_some()).count(),"candidate_count":snapshot.player_frames.len(),"media_request_count":snapshot.media_requests.len(),"activation":snapshot.activation,"activation_history":snapshot.activation_history,"navigation":snapshot.navigation_diagnostic,"media_verified":false,"native_playback":false});
        Err(error)
    }
}
fn required_body_unavailable(snapshot: &BrowserSnapshot, target: &str) -> bool {
    use crate::application::browser_network::ResponsePurpose;
    snapshot.network.as_ref().is_some_and(|network| {
        network.unavailable_captures.iter().any(|failed| {
            failed.current
                && failed.owner_context.document_generation == snapshot.document_generation
                && Some(failed.owner_context.native_navigation_id) == snapshot.native_navigation_id
                && failed.capture.scope().url == target
                && matches!(
                    failed.capture.purpose(),
                    ResponsePurpose::PlayerScript
                        | ResponsePurpose::PlayerBootstrap
                        | ResponsePurpose::ManifestCandidate
                )
        })
    })
}
fn resolution_prerequisites(snapshot: &BrowserSnapshot, target: &str) -> bool {
    use crate::application::browser_network::ResponsePurpose;
    observed_response_status(snapshot, target, &ResponsePurpose::PlayerScript)
        .is_some_and(|status| (200..300).contains(&status))
        && observed_response_status(snapshot, target, &ResponsePurpose::PlayerBootstrap)
            .is_some_and(|status| (200..300).contains(&status))
        && frame_runtime::current_finished(snapshot, target)
        && !required_body_unavailable(snapshot, target)
}
fn player_asset_status(snapshot: &BrowserSnapshot, target: &str) -> Option<u16> {
    if snapshot.state != BrowserWorkState::Running {
        return None;
    }
    snapshot
        .network
        .as_ref()?
        .bodies
        .iter()
        .find(|body| {
            body.current
                && body.owner_context.document_generation == snapshot.document_generation
                && Some(body.owner_context.native_navigation_id) == snapshot.native_navigation_id
                && body.body.capture.scope().url == target
                && body.body.capture.purpose()
                    == &crate::application::browser_network::ResponsePurpose::PlayerScript
        })
        .map(|body| body.body.capture.response().status)
}
pub(super) fn child_document_ready(snapshot: &BrowserSnapshot, expected_url: &str) -> bool {
    snapshot.state == BrowserWorkState::Running
        && !snapshot.navigation_pending
        && snapshot.frame_documents.iter().any(|frame| {
            !frame.destroyed
                && !frame.superseded
                && frame.identity.owner_document_generation == snapshot.document_generation
                && Some(frame.identity.owner_native_navigation_id) == snapshot.native_navigation_id
                && frame.identity.url == expected_url
                && frame.completion.is_some()
                && frame.document.as_deref().is_some_and(|document| {
                    document.document_generation == frame.identity.document_generation
                        && document.final_url == expected_url
                })
        })
}
/// Completion and DOM belong to the same current player document; neither alone is ready.
pub(super) fn player_document_ready(
    snapshot: &BrowserSnapshot,
    initial_generation: DecimalU64,
) -> bool {
    snapshot.document_generation > initial_generation
        && !snapshot.navigation_pending
        && snapshot
            .current_url
            .as_deref()
            .is_some_and(crate::domain::navigation::player_page)
        && snapshot
            .navigation_diagnostic
            .as_ref()
            .is_some_and(|completion| {
                completion.document_generation == snapshot.document_generation
                    && snapshot.native_navigation_id == Some(completion.native_navigation_id)
            })
        && snapshot.document.as_deref().is_some_and(|document| {
            document.document_generation == snapshot.document_generation
                && snapshot.current_url.as_deref() == Some(document.final_url.as_str())
        })
}
impl JobExecutor for BrowserEpisodeResolveJobs {
    fn operations(&self) -> Vec<String> {
        vec!["episodes.resolve".into()]
    }
    fn supports(&self, operation: &str) -> bool {
        operation == "episodes.resolve"
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.supports(operation).then_some(120_000)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        if !self.supports(operation) {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Episode executor does not support this operation",
            ));
        }
        let input: ResolveEpisode = serde_json::from_value(parameters)
            .map_err(|_| invalid("Episode resolution requires only episode_url"))?;
        Ok(json!({"episode_url":input.location()?.as_str()}))
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        if expected.is_some() {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Episode browser source is a request descriptor, not frozen server content",
            ));
        }
        let parameters = self.normalize_parameters(operation, parameters)?;
        let descriptor = descriptor(&parameters)?;
        Ok(PreparedJobInput {
            operation: operation.into(),
            parameters,
            source: blobs.put_utf8(&descriptor)?,
            execution_timeout_ms: DecimalU64(60_000),
        })
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        let parameters = self.normalize_parameters(&input.operation, input.parameters.clone())?;
        let descriptor = descriptor(&parameters)?;
        if parameters != input.parameters
            || blobs.read_utf8(&input.source)? != descriptor
            || input.source.sha256 != format!("{:x}", Sha256::digest(descriptor.as_bytes()))
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Episode request descriptor identity changed",
            ));
        }
        let mut artifacts = Vec::new();
        self.run(input,control,&mut artifacts).map_err(|mut error| {
            if !error.details.is_object() {error.details=json!({"underlying_details_omitted":true});}
            error.details["raw_artifacts"]=json!(artifacts);
            error.details["source_kind"]=json!("request_descriptor");
            error.details["media_verified"]=json!(false);
            error.remediation="Read the preserved private browser evidence; no automatic URL rewriting or unverified media publication".into();
            error
        })
    }
}
fn descriptor(parameters: &Value) -> Result<String, ApiError> {
    serde_json::to_string(&json!({"operation":"episodes.resolve","parameters":parameters,"source_kind":"request_descriptor"})).map_err(|_|invalid("Episode descriptor serialization failed"))
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
