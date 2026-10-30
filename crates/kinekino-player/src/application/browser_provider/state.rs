use super::*;
use crate::application::browser_network::{NetworkCaptureState, NetworkEvent, ProtocolSession};
use crate::{application::api::ErrorCode, domain::navigation};

pub enum BrowserEvent {
    Started {
        engine_version: String,
        blocking_installed: bool,
    },
    Navigated {
        url: String,
        document_generation: u64,
    },
    Counts {
        requested: u64,
        blocked: u64,
    },
    DocumentReady(BrowserDocument),
    NavigationIdentity {
        document_generation: u64,
        native_navigation_id: u64,
    },
    NavigationCompleted(MainNavigationDiagnostic),
    ActivationInvoked {
        document_generation: u64,
        native_navigation_id: u64,
    },
    ActivationRejected {
        document_generation: u64,
        native_navigation_id: u64,
    },
    PlayerFrame(PlayerFrameObservation),
    MediaRequested(MediaRequestObservation),
    FrameDocument(FrameDocumentEvent),
    FrameRuntime {
        request: FrameRuntimeRequest,
        json: String,
        observed_unix_ms: DecimalU64,
    },
    NetworkObserved {
        session: ProtocolSession,
        event: NetworkEvent,
    },
    NetworkObservedAt {
        session: ProtocolSession,
        event: NetworkEvent,
        received_unix_ms: DecimalU64,
    },
    NetworkBody {
        read: OwnedResponseCapture,
        json: String,
        observed_unix_ms: DecimalU64,
    },
    NetworkBodyUnavailable(OwnedResponseCapture),
    MediaReady(BrowserMedia),
    Failed(ApiError),
    Cancelled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDisposition {
    Applied,
    Stale,
    AlreadyTerminal,
}

/// Pure transition policy; the adapter serializes access without holding COM objects here.
#[derive(Clone)]
pub struct BrowserWork {
    request: ValidatedBrowserRequest,
    snapshot: BrowserSnapshot,
    network: Option<NetworkCaptureState>,
    native_capture_released: bool,
}
impl BrowserWork {
    pub(crate) fn admit_media_stream(
        &mut self,
        generation: u64,
        session: &ProtocolSession,
        request_id: &str,
    ) -> Result<OwnedMediaStream, ApiError> {
        if self.snapshot.state != BrowserWorkState::Running
            || self.snapshot.navigation_pending
            || self.snapshot.document_generation.0 != generation
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original browser work changed",
            ));
        }
        let native_navigation_id = self.snapshot.native_navigation_id.ok_or_else(|| {
            ApiError::new(
                ErrorCode::SourceExpired,
                "Original navigation identity is unavailable",
            )
        })?;
        let admission = self
            .network
            .as_mut()
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::SourceExpired,
                    "Original network owner is unavailable",
                )
            })?
            .admit_stream(session, request_id)?;
        Ok(OwnedMediaStream {
            admission,
            ticket: self.snapshot.ticket.clone(),
            document_generation: self.snapshot.document_generation,
            native_navigation_id,
        })
    }
    pub(crate) fn check_media_stream(&self, stream: &OwnedMediaStream) -> Result<(), ApiError> {
        if self.snapshot.state != BrowserWorkState::Running
            || self.snapshot.navigation_pending
            || self.snapshot.ticket != stream.ticket
            || self.snapshot.document_generation != stream.document_generation
            || self.snapshot.native_navigation_id != Some(stream.native_navigation_id)
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original browser stream owner changed",
            ));
        }
        self.network
            .as_ref()
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::SourceExpired,
                    "Original network owner is unavailable",
                )
            })?
            .check_stream(&stream.admission)
    }
    pub fn new(ticket: BrowserTicket, request: ValidatedBrowserRequest) -> Result<Self, ApiError> {
        if ticket.request_id() != request.request().request_id {
            return Err(invalid("Browser ticket does not match request identity"));
        }
        let snapshot = BrowserSnapshot {
            ticket,
            state: BrowserWorkState::Queued,
            current_url: None,
            document_generation: DecimalU64(0),
            navigation_pending: true,
            navigation_history: Vec::new(),
            native_navigation_id: None,
            navigation_diagnostic: None,
            activation: None,
            activation_history: Vec::new(),
            player_frames: Vec::new(),
            media_requests: Vec::new(),
            frame_documents: Vec::new(),
            network: None,
            engine_version: None,
            blocking_requested: request.request().blocking_enabled,
            blocking_installed: false,
            requested_count: DecimalU64(0),
            blocked_count: DecimalU64(0),
            result: None,
            document: None,
            failure: None,
        };
        let network = if request.request().operation == BrowserOperation::ResolveEpisode {
            Some(NetworkCaptureState::new(&request.request().url)?)
        } else {
            None
        };
        Ok(Self {
            request,
            snapshot,
            network,
            native_capture_released: false,
        })
    }
    pub fn request(&self) -> &ValidatedBrowserRequest {
        &self.request
    }
    /// Native worker observation after dropping its original Capture, not process exit.
    pub(crate) fn record_native_capture_released(&mut self) -> Result<(), ApiError> {
        if !self.snapshot.state.terminal() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Active browser capture cannot be released",
            ));
        }
        self.native_capture_released = true;
        Ok(())
    }
    pub(crate) fn native_capture_released(&self) -> bool {
        self.native_capture_released
    }
    pub fn snapshot(&self) -> BrowserSnapshot {
        let mut snapshot = self.snapshot.clone();
        snapshot.network = self.network.as_ref().map(NetworkCaptureState::snapshot);
        snapshot
    }
    pub fn take_frame_runtime_probe(
        &mut self,
        now: std::time::Instant,
    ) -> Option<FrameRuntimeRequest> {
        let asset_observed = self.snapshot.native_navigation_id.is_some_and(|native_id| {
            self.network.as_ref().is_some_and(|network| {
                network.has_current_player_script_body(self.snapshot.document_generation, native_id)
            })
        });
        frame_runtime::reserve_current(&mut self.snapshot, asset_observed, now)
    }
    pub fn take_network_body(&mut self) -> Option<OwnedResponseCapture> {
        if self.snapshot.state != BrowserWorkState::Running
            || self.snapshot.navigation_pending
            || self
                .snapshot
                .current_url
                .as_deref()
                .is_none_or(|url| !navigation::player_page(url))
            || self
                .snapshot
                .navigation_diagnostic
                .as_ref()
                .is_none_or(|completion| {
                    completion.document_generation != self.snapshot.document_generation
                        || Some(completion.native_navigation_id)
                            != self.snapshot.native_navigation_id
                })
        {
            return None;
        }
        let native_id = self.snapshot.native_navigation_id?;
        self.network
            .as_mut()?
            .take_body()
            .map(|capture| OwnedResponseCapture {
                session: capture.scope().session.clone(),
                capture,
                owner_document_generation: self.snapshot.document_generation,
                owner_native_navigation_id: native_id,
            })
    }
    pub fn cancel(&mut self) -> CancelDisposition {
        if self.snapshot.state.terminal() {
            return CancelDisposition::AlreadyTerminal;
        }
        if self.snapshot.state == BrowserWorkState::Cancelling {
            return CancelDisposition::AlreadyRequested;
        }
        self.snapshot.state = BrowserWorkState::Cancelling;
        if let Some(network) = &mut self.network {
            network.close();
        }
        CancelDisposition::Requested
    }
    pub fn reserve_navigation(
        &mut self,
        expected_document_generation: u64,
        url: &str,
    ) -> Result<(), ApiError> {
        if self.request.request().operation != BrowserOperation::ResolveEpisode {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Explicit staged navigation is for episode resolution",
            ));
        }
        if self.snapshot.state != BrowserWorkState::Running
            || self.snapshot.navigation_pending
            || expected_document_generation != self.snapshot.document_generation.0
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Browser document changed or navigation is pending",
            ));
        }
        if !self.request.navigation_allowed(url) {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Browser navigation target is not allowed",
            ));
        }
        self.snapshot.navigation_pending = true;
        self.snapshot.document = None;
        Ok(())
    }
    pub fn reserve_episode_activation(&mut self, expected_generation: u64) -> Result<(), ApiError> {
        if self.request.request().operation != BrowserOperation::ResolveEpisode {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Only an episode request can activate watch",
            ));
        }
        if !self.current_document(expected_generation, &self.request.request().url)
            || self.snapshot.document.is_none()
            || self.snapshot.activation.is_some()
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Episode document changed or watch was already admitted",
            ));
        }
        let native_navigation_id = self.snapshot.native_navigation_id.ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "Native episode navigation identity was not observed",
            )
        })?;
        let completion = self
            .snapshot
            .navigation_diagnostic
            .as_ref()
            .filter(|completion| {
                completion.document_generation.0 == expected_generation
                    && completion.native_navigation_id == native_navigation_id
            })
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Episode navigation completion was not observed",
                )
            })?;
        if !completion.is_success
            || completion
                .http_status_code
                .is_some_and(|status| status >= 400)
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Episode navigation did not complete successfully",
            ));
        }
        if !self.snapshot.activation_history.is_empty() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Episode watch was already admitted for this work",
            ));
        }
        let activation = EpisodeActivationObservation {
            document_generation: DecimalU64(expected_generation),
            native_navigation_id,
            status: EpisodeActivationStatus::Enqueued,
        };
        self.snapshot.activation_history.push(activation.clone());
        self.snapshot.activation = Some(activation);
        Ok(())
    }
    pub fn expire(&mut self) {
        if !self.snapshot.state.terminal() {
            self.fail(ApiError::new(
                ErrorCode::Timeout,
                "Browser operation deadline expired",
            ));
        }
    }
    pub fn apply(&mut self, ticket: &BrowserTicket, event: BrowserEvent) -> EventDisposition {
        if ticket != &self.snapshot.ticket {
            return EventDisposition::Stale;
        }
        if self.snapshot.state.terminal() {
            return EventDisposition::AlreadyTerminal;
        }
        if self.snapshot.state == BrowserWorkState::Cancelling {
            return match event {
                BrowserEvent::Cancelled => {
                    self.snapshot.state = BrowserWorkState::Cancelled;
                    EventDisposition::Applied
                }
                BrowserEvent::Failed(error) => {
                    self.fail(error);
                    EventDisposition::Applied
                }
                _ => EventDisposition::Stale,
            };
        }
        match event {
            BrowserEvent::Started {
                engine_version,
                blocking_installed,
            } => {
                if self.snapshot.state != BrowserWorkState::Queued {
                    return EventDisposition::Stale;
                }
                if engine_version.is_empty()
                    || engine_version.len() > 128
                    || engine_version.chars().any(char::is_control)
                {
                    self.fail(invalid("Invalid browser engine identity"));
                } else if self.snapshot.blocking_requested && !blocking_installed {
                    self.fail(ApiError::new(
                        ErrorCode::EngineFailed,
                        "Requested browser blocking filter was not installed",
                    ));
                } else {
                    self.snapshot.engine_version = Some(engine_version);
                    self.snapshot.blocking_installed = blocking_installed;
                    self.snapshot.state = BrowserWorkState::Running;
                }
            }
            BrowserEvent::Navigated {
                url,
                document_generation,
            } => {
                if self.snapshot.state != BrowserWorkState::Running
                    || document_generation <= self.snapshot.document_generation.0
                {
                    return EventDisposition::Stale;
                }
                if !self.request.navigation_allowed(&url) {
                    self.fail(ApiError::new(
                        ErrorCode::PermissionDenied,
                        "Browser left the permitted navigation scope",
                    ));
                } else if self.snapshot.navigation_history.len() >= MAX_NAVIGATIONS {
                    self.fail(ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Browser navigation observation limit exceeded",
                    ));
                } else {
                    self.snapshot
                        .navigation_history
                        .push(BrowserNavigationObservation {
                            url: url.clone(),
                            document_generation: DecimalU64(document_generation),
                        });
                    self.snapshot.current_url = Some(url);
                    self.snapshot.document_generation = DecimalU64(document_generation);
                    self.snapshot.navigation_pending = false;
                    self.snapshot.document = None;
                    self.snapshot.native_navigation_id = None;
                    self.snapshot.navigation_diagnostic = None;
                    self.snapshot.activation = None;
                    self.snapshot.player_frames.clear();
                    self.snapshot.frame_documents.clear();
                }
            }
            BrowserEvent::Counts { requested, blocked } => {
                if requested < self.snapshot.requested_count.0
                    || blocked < self.snapshot.blocked_count.0
                    || blocked > requested
                {
                    self.fail(invalid("Browser counters are not monotonic"));
                } else {
                    self.snapshot.requested_count = DecimalU64(requested);
                    self.snapshot.blocked_count = DecimalU64(blocked);
                }
            }
            BrowserEvent::DocumentReady(document) => {
                if !self.current_document(document.document_generation.0, &document.final_url) {
                    return EventDisposition::Stale;
                }
                let response_valid = match (&document.kind, &document.response) {
                    (CaptureKind::NetworkResponse, Some(response)) => {
                        (100..=599).contains(&response.status)
                            && response.content_type.len() <= 1024
                            && response
                                .content_encoding
                                .as_ref()
                                .is_none_or(|v| v.len() <= 128)
                    }
                    (CaptureKind::DomSnapshot, None) => true,
                    _ => false,
                };
                if document.body.len() > MAX_CAPTURE_BYTES || !response_valid {
                    self.fail(invalid("Invalid browser document evidence"));
                } else {
                    let document = Arc::new(document);
                    self.snapshot.document = Some(document.clone());
                    if self.request.request().operation != BrowserOperation::ResolveEpisode {
                        self.snapshot.result =
                            Some(Arc::new(BrowserResult::Document((*document).clone())));
                        self.snapshot.state = BrowserWorkState::Succeeded;
                    }
                }
            }
            BrowserEvent::NavigationIdentity {
                document_generation,
                native_navigation_id,
            } => {
                if self.snapshot.document_generation.0 != document_generation
                    || self.snapshot.state != BrowserWorkState::Running
                {
                    return EventDisposition::Stale;
                }
                if self
                    .snapshot
                    .native_navigation_id
                    .is_some_and(|id| id.0 != native_navigation_id)
                {
                    self.fail(invalid(
                        "Native navigation identity changed without a new document",
                    ));
                } else {
                    self.snapshot.native_navigation_id = Some(DecimalU64(native_navigation_id));
                }
            }
            BrowserEvent::NavigationCompleted(completion) => {
                if self.snapshot.state != BrowserWorkState::Running
                    || self.snapshot.document_generation != completion.document_generation
                    || self.snapshot.native_navigation_id != Some(completion.native_navigation_id)
                {
                    return EventDisposition::Stale;
                }
                if completion
                    .http_status_code
                    .is_some_and(|status| status != 0 && !(100..=599).contains(&status))
                {
                    self.fail(invalid("Invalid native navigation status"));
                } else {
                    self.snapshot.navigation_diagnostic = Some(completion);
                }
            }
            BrowserEvent::ActivationInvoked {
                document_generation,
                native_navigation_id,
            } => {
                if self.snapshot.document_generation.0 != document_generation
                    || self.snapshot.native_navigation_id != Some(DecimalU64(native_navigation_id))
                {
                    return EventDisposition::Stale;
                }
                let Some(activation) = self.snapshot.activation.as_mut().filter(|activation| {
                    activation.document_generation.0 == document_generation
                        && activation.native_navigation_id.0 == native_navigation_id
                }) else {
                    return EventDisposition::Stale;
                };
                activation.status = EpisodeActivationStatus::InvocationObserved;
                if self.snapshot.activation_history.len() == 1 {
                    self.snapshot.activation_history.push(activation.clone());
                }
            }
            BrowserEvent::ActivationRejected {
                document_generation,
                native_navigation_id,
            } => {
                if self.snapshot.document_generation.0 != document_generation
                    || self.snapshot.native_navigation_id != Some(DecimalU64(native_navigation_id))
                    || self.snapshot.activation.as_ref().is_none_or(|activation| {
                        activation.document_generation.0 != document_generation
                            || activation.native_navigation_id.0 != native_navigation_id
                    })
                {
                    return EventDisposition::Stale;
                }
                self.fail(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Episode watch actuator did not observe its fixed target",
                ));
            }
            BrowserEvent::PlayerFrame(frame) => {
                if self.snapshot.state != BrowserWorkState::Running
                    || self.snapshot.navigation_pending
                    || self.snapshot.document_generation
                        != frame.owner_document_generation_at_observation
                    || self.snapshot.native_navigation_id
                        != Some(frame.owner_navigation_id_at_observation)
                    || self.snapshot.activation.as_ref().is_none_or(|activation| {
                        activation.document_generation
                            != frame.owner_document_generation_at_observation
                            || activation.native_navigation_id
                                != frame.owner_navigation_id_at_observation
                    })
                {
                    return EventDisposition::Stale;
                }
                if frame.url.len() > 8192
                    || frame.lineage_verified
                    || !navigation::player_page(&frame.url)
                    || frame
                        .request_method
                        .as_ref()
                        .is_some_and(|method| method.len() > 16)
                    || frame
                        .referer
                        .as_ref()
                        .is_some_and(|referer| referer.len() > 8192)
                {
                    self.fail(invalid("Invalid player frame observation"));
                } else if self.snapshot.player_frames.len() >= 16 {
                    self.fail(ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Player frame observation limit exceeded",
                    ));
                } else {
                    self.snapshot.player_frames.push(frame);
                }
            }
            BrowserEvent::MediaRequested(candidate) => {
                if !self.current_document(candidate.document_generation.0, &candidate.document_url)
                    || self.snapshot.native_navigation_id != Some(candidate.native_navigation_id)
                    || self.snapshot.activation_history.is_empty()
                {
                    return EventDisposition::Stale;
                }
                if !navigation::player_page(&candidate.document_url)
                    || !navigation::media_candidate(&candidate.url)
                    || candidate.url.len() > 8192
                    || candidate
                        .referer
                        .as_ref()
                        .is_some_and(|value| value.len() > 8192)
                    || candidate.request_method.is_empty()
                    || candidate.request_method.len() > 16
                {
                    self.fail(invalid("Invalid media request observation"));
                } else if self.snapshot.media_requests.len() >= 16 {
                    self.fail(ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Media request observation limit exceeded",
                    ));
                } else {
                    self.snapshot.media_requests.push(candidate);
                }
            }
            BrowserEvent::FrameDocument(event) => {
                let identity = event.identity();
                if self.snapshot.state != BrowserWorkState::Running
                    || self.snapshot.navigation_pending
                    || self.request.request().operation != BrowserOperation::ResolveEpisode
                    || self.snapshot.activation_history.is_empty()
                    || self.snapshot.document_generation != identity.owner_document_generation
                    || self.snapshot.native_navigation_id
                        != Some(identity.owner_native_navigation_id)
                    || self
                        .snapshot
                        .current_url
                        .as_deref()
                        .is_none_or(|url| !navigation::player_page(url))
                {
                    return EventDisposition::Stale;
                }
                match frame_documents::apply(&mut self.snapshot.frame_documents, event) {
                    Ok(disposition) => return disposition,
                    Err(error) => self.fail(error),
                }
            }
            BrowserEvent::FrameRuntime {
                request,
                json,
                observed_unix_ms,
            } => {
                match frame_runtime::apply_current(
                    &mut self.snapshot,
                    &request,
                    &json,
                    observed_unix_ms,
                    std::time::Instant::now(),
                ) {
                    Ok(disposition) => return disposition,
                    Err(error) => self.fail(error),
                }
            }
            BrowserEvent::NetworkObserved { session, event } => {
                if self.snapshot.state != BrowserWorkState::Running {
                    return EventDisposition::Stale;
                }
                let Some(network) = &mut self.network else {
                    return EventDisposition::Stale;
                };
                if let Err(error) = network.observe(&session, event) {
                    self.fail(error);
                }
            }
            BrowserEvent::NetworkObservedAt {
                session,
                event,
                received_unix_ms,
            } => {
                if self.snapshot.state != BrowserWorkState::Running {
                    return EventDisposition::Stale;
                }
                let Some(network) = &mut self.network else {
                    return EventDisposition::Stale;
                };
                if let Err(error) = network.observe_at(&session, event, Some(received_unix_ms)) {
                    self.fail(error);
                }
            }
            BrowserEvent::NetworkBody {
                read,
                json,
                observed_unix_ms,
            } => {
                if !self.network_owner_current(&read) {
                    return EventDisposition::Stale;
                }
                let Some(network) = &mut self.network else {
                    return EventDisposition::Stale;
                };
                if let Err(error) = network.body(
                    &read.session,
                    &read.capture,
                    &json,
                    observed_unix_ms,
                    crate::application::browser_network::BodyOwnerContext {
                        document_generation: read.owner_document_generation,
                        native_navigation_id: read.owner_native_navigation_id,
                    },
                ) {
                    self.fail(error);
                }
            }
            BrowserEvent::NetworkBodyUnavailable(read) => {
                if !self.network_owner_current(&read) {
                    return EventDisposition::Stale;
                }
                if let Some(network) = &mut self.network {
                    network.unavailable(
                        &read.session,
                        &read.capture,
                        crate::application::browser_network::BodyOwnerContext {
                            document_generation: read.owner_document_generation,
                            native_navigation_id: read.owner_native_navigation_id,
                        },
                    );
                }
            }
            BrowserEvent::MediaReady(media) => {
                if !self.current_document(media.document_generation.0, &media.document_url) {
                    return EventDisposition::Stale;
                }
                let reconstructed = if let Some(prefix) = &media.manifest_prefix {
                    (prefix.len() <= 4096)
                        .then(|| {
                            MediaSource::from_manifest(
                                media.source.url.clone(),
                                media.source.referer.clone(),
                                prefix,
                            )
                        })
                        .flatten()
                } else {
                    MediaSource::from_request(
                        media.source.url.clone(),
                        media.source.referer.clone(),
                    )
                };
                if self.request.request().operation != BrowserOperation::ResolveEpisode
                    || !navigation::player_page(&media.document_url)
                    || reconstructed.is_none_or(|source| source.hls != media.source.hls)
                {
                    self.fail(invalid("Invalid browser media evidence"));
                } else {
                    self.snapshot.result = Some(Arc::new(BrowserResult::Media(media)));
                    self.snapshot.state = BrowserWorkState::Succeeded;
                }
            }
            BrowserEvent::Failed(error) => self.fail(error),
            BrowserEvent::Cancelled => return EventDisposition::Stale,
        }
        EventDisposition::Applied
    }
    fn current_document(&self, generation: u64, url: &str) -> bool {
        self.snapshot.state == BrowserWorkState::Running
            && !self.snapshot.navigation_pending
            && generation > 0
            && generation == self.snapshot.document_generation.0
            && self.snapshot.current_url.as_deref() == Some(url)
    }
    fn fail(&mut self, error: ApiError) {
        if let Some(network) = &mut self.network {
            network.close();
        }
        self.snapshot.state = BrowserWorkState::Failed;
        self.snapshot.failure = Some(Arc::new(error));
        self.snapshot.result = None;
    }
    fn network_owner_current(&self, read: &OwnedResponseCapture) -> bool {
        self.snapshot.state == BrowserWorkState::Running
            && self.snapshot.document_generation == read.owner_document_generation
            && self.snapshot.native_navigation_id == Some(read.owner_native_navigation_id)
    }
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
