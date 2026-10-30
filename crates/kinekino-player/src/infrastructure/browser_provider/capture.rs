use super::{
    error,
    frame_events::FrameEvents,
    navigation_events::NavigationEvents,
    protocol_observer::{ProtocolMessage, ProtocolObserver},
    ui_policy::HiddenUiPolicy,
    window::HiddenWindow,
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        browser_provider::*,
    },
    domain::{api_values::DecimalU64, media::MediaSource},
    infrastructure::adblock,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender},
    },
};
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use webview2_com::{CoTaskMemPWSTR, WebResourceRequestedEventHandler};
use windows::{
    Win32::System::Com::IStream,
    core::{Interface, PWSTR, w},
};
use wry::{NewWindowResponse, WebContext, WebView, WebViewBuilder, WebViewExtWindows};

pub(super) enum Callback {
    Navigation(String, u64, u64),
    NavigationCompleted(MainNavigationDiagnostic),
    ActivationResult(u64, u64, bool),
    PlayerFrame(PlayerFrameObservation),
    FrameDocument(FrameDocumentEvent),
    FrameDom(FrameDocumentIdentity, String),
    FrameRuntime(FrameRuntimeRequest, String, DecimalU64),
    Loaded(String),
    Dom {
        url: String,
        generation: u64,
        json: String,
    },
    MediaRequested(MediaRequestObservation),
    Failed(ApiError),
}
pub(super) struct Page {
    pub url: String,
    pub generation: u64,
    pub native_navigation_id: Option<u64>,
    pub activation: Option<(u64, u64)>,
}
pub(super) struct Capture {
    // Struct fields drop in declaration order: unregister before releasing the WebView.
    _ui_policy: HiddenUiPolicy,
    _navigation: NavigationEvents,
    frames: Option<FrameEvents>,
    protocol: Option<ProtocolObserver>,
    view: WebView,
    page: Rc<RefCell<Page>>,
    receiver: Receiver<Callback>,
    sender: SyncSender<Callback>,
    overflow: Arc<std::sync::atomic::AtomicBool>,
    requested: Rc<Cell<u64>>,
    blocked: Rc<Cell<u64>>,
    dom_requested: Option<u64>,
    pending_navigation: Option<String>,
}
fn emit(sender: &SyncSender<Callback>, overflow: &std::sync::atomic::AtomicBool, event: Callback) {
    if sender.try_send(event).is_err() {
        overflow.store(true, std::sync::atomic::Ordering::Release);
    }
}
pub(super) fn now_ms() -> DecimalU64 {
    DecimalU64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u64::MAX as u128) as u64,
    )
}
impl Capture {
    pub fn new(
        parent: &HiddenWindow,
        context: &mut WebContext,
        request: &ValidatedBrowserRequest,
    ) -> Result<Self, ApiError> {
        let (sender, receiver) = mpsc::sync_channel(MAX_COMMANDS);
        let overflow = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let page = Rc::new(RefCell::new(Page {
            url: String::new(),
            generation: 0,
            native_navigation_id: None,
            activation: None,
        }));
        let popup_sender = sender.clone();
        let popup_overflow = overflow.clone();
        let popup_policy = request.clone();
        let popup_page = page.clone();
        let view = WebViewBuilder::new_with_web_context(context)
            .with_visible(false)
            .with_focused(false)
            .with_devtools(false)
            .with_new_window_req_handler(move |url, _| {
                let current = popup_page.borrow();
                if crate::domain::navigation::player_page(&url)
                    && let Some(native_id) = current.native_navigation_id
                    && current.activation == Some((current.generation, native_id))
                {
                    emit(
                        &popup_sender,
                        &popup_overflow,
                        Callback::PlayerFrame(PlayerFrameObservation {
                            kind: PlayerDocumentObservationKind::PopupIntent,
                            owner_document_generation_at_observation: DecimalU64(
                                current.generation,
                            ),
                            owner_navigation_id_at_observation: DecimalU64(native_id),
                            frame_navigation_id: None,
                            frame_id: None,
                            parent_frame_id: None,
                            request_source_kind: None,
                            lineage_verified: false,
                            url,
                            observed_unix_ms: now_ms(),
                            request_method: None,
                            referer: None,
                        }),
                    );
                    return NewWindowResponse::Deny;
                }
                if popup_policy.navigation_allowed(&url) {
                    // Popup is evidence for SDK staged extraction, never an implicit second window.
                    emit(
                        &popup_sender,
                        &popup_overflow,
                        Callback::Failed(error(
                            ErrorCode::NeedsInput,
                            "Browser requested a popup; explicit staged navigation is required",
                        )),
                    );
                }
                NewWindowResponse::Deny
            })
            .build_as_child(parent)
            .map_err(|_| error(ErrorCode::EngineFailed, "Browser engine creation failed"))?;
        let failure_sender = sender.clone();
        let failure_overflow = overflow.clone();
        let ui_policy = HiddenUiPolicy::install(
            &view,
            Rc::new(move |error| {
                emit(&failure_sender, &failure_overflow, Callback::Failed(error));
            }),
        )?;
        let requested = Rc::new(Cell::new(0u64));
        let navigation_sender = sender.clone();
        let navigation_overflow = overflow.clone();
        let navigation = NavigationEvents::install(
            &view,
            page.clone(),
            request.clone(),
            Rc::new(move |event| emit(&navigation_sender, &navigation_overflow, event)),
        )?;
        let blocked = Rc::new(Cell::new(0u64));
        let frame_sender = sender.clone();
        let frame_overflow = overflow.clone();
        let frames = if request.request().operation == BrowserOperation::ResolveEpisode {
            Some(FrameEvents::install(
                &view,
                page.clone(),
                Rc::new(move |event| emit(&frame_sender, &frame_overflow, event)),
            )?)
        } else {
            None
        };
        let protocol = if request.request().operation == BrowserOperation::ResolveEpisode {
            Some(ProtocolObserver::install(&view)?)
        } else {
            None
        };
        install_filter(
            &view,
            request,
            page.clone(),
            sender.clone(),
            overflow.clone(),
            requested.clone(),
            blocked.clone(),
        )?;
        Ok(Self {
            _ui_policy: ui_policy,
            _navigation: navigation,
            frames,
            protocol,
            view,
            page,
            receiver,
            sender,
            overflow,
            requested,
            blocked,
            dom_requested: None,
            pending_navigation: None,
        })
    }
    pub fn navigate(&mut self, url: &str) -> Result<(), ApiError> {
        self.dom_requested = None;
        if self
            .protocol
            .as_ref()
            .is_some_and(|protocol| !protocol.ready())
        {
            self.pending_navigation = Some(url.into());
            return Ok(());
        }
        self.view.load_url(url).map_err(|_| {
            error(
                ErrorCode::EngineFailed,
                "Browser navigation could not start",
            )
        })
    }
    pub fn drive_protocol(&mut self) -> Result<(), ApiError> {
        if let Some(protocol) = &self.protocol {
            protocol.drive_targets()?;
            if let Some(failure) = protocol.take_failure() {
                return Err(failure);
            }
            if protocol.ready()
                && let Some(url) = self.pending_navigation.take()
            {
                self.navigate(&url)?;
            }
        }
        Ok(())
    }
    pub fn next_protocol(&self) -> Option<ProtocolMessage> {
        self.protocol.as_ref()?.next()
    }
    pub fn observe_frame_runtime(&self, request: FrameRuntimeRequest) -> Result<(), ApiError> {
        self.frames
            .as_ref()
            .ok_or_else(|| {
                error(
                    ErrorCode::Unsupported,
                    "Child runtime observation unavailable",
                )
            })?
            .observe_runtime(request)
    }
    pub fn observe_protocol_target(
        &self,
        session: &crate::application::browser_network::ProtocolSession,
        event: &crate::application::browser_network::NetworkEvent,
    ) -> Result<(), ApiError> {
        if let Some(protocol) = &self.protocol {
            protocol.observe_target_event(session, event)?;
        }
        Ok(())
    }
    pub fn protocol_body_capacity(&self) -> bool {
        self.protocol
            .as_ref()
            .is_some_and(ProtocolObserver::body_capacity)
    }
    pub fn read_protocol_body(&self, read: OwnedResponseCapture) -> Result<(), ApiError> {
        self.protocol
            .as_ref()
            .ok_or_else(|| {
                error(
                    ErrorCode::Unsupported,
                    "Owned protocol observer unavailable",
                )
            })?
            .capture_body(read)
    }
    pub fn activate_episode(&mut self, generation: u64) -> Result<(), ApiError> {
        let mut page = self.page.borrow_mut();
        if page.activation.is_some() {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Episode watch was already invoked for this document",
            ));
        }
        let native_id = page
            .native_navigation_id
            .filter(|_| page.generation == generation)
            .ok_or_else(|| {
                error(
                    ErrorCode::RevisionConflict,
                    "Episode activation document changed",
                )
            })?;
        let expected_url = serde_json::to_string(&page.url).map_err(|_| {
            error(
                ErrorCode::Internal,
                "Episode activation identity serialization failed",
            )
        })?;
        page.activation = Some((generation, native_id));
        drop(page);
        let sender = self.sender.clone();
        let overflow = self.overflow.clone();
        // Fixed actuator only: Rust owns admission, identities and all navigation policy.
        let script = format!(
            "(()=>{{if(location.href!=={expected_url})return false;const button=document.querySelector('button.watch-now');if(!button)return false;button.click();return true;}})()"
        );
        self.view
            .evaluate_script_with_callback(&script, move |json| {
                emit(
                    &sender,
                    &overflow,
                    Callback::ActivationResult(generation, native_id, json == "true"),
                );
            })
            .map_err(|_| {
                error(
                    ErrorCode::EngineFailed,
                    "Episode watch actuator could not be enqueued",
                )
            })
    }
    pub fn counts(&self) -> (u64, u64) {
        (self.requested.get(), self.blocked.get())
    }
    pub fn overflowed(&self) -> bool {
        self.overflow.load(std::sync::atomic::Ordering::Acquire)
    }
    pub fn next(&self) -> Option<Callback> {
        self.receiver.try_recv().ok()
    }
    pub fn request_dom(&mut self, expected_url: &str) -> Result<(), ApiError> {
        let page = self.page.borrow();
        if page.url != expected_url
            || page.generation == 0
            || self.dom_requested == Some(page.generation)
        {
            return Ok(());
        }
        let url = page.url.clone();
        let generation = page.generation;
        drop(page);
        let sender = self.sender.clone();
        let overflow = self.overflow.clone();
        self.dom_requested = Some(generation);
        // Minimal serialization bridge only; Rust owns validation, parsing and policy.
        let script = format!(
            "(()=>{{const root=document.documentElement;if(!root)return null;const limit={MAX_CAPTURE_BYTES};const html=root.outerHTML;return [location.href,html.slice(0,limit+1),html.length>limit];}})()"
        );
        // Worst-case JSON escaping of the bounded UTF-16 HTML, URL and envelope.
        let max_json_bytes = 6 * (MAX_CAPTURE_BYTES + 1 + 8192) + 64;
        self.view
            .evaluate_script_with_callback(&script, move |json| {
                if json.len() > max_json_bytes {
                    emit(
                        &sender,
                        &overflow,
                        Callback::Failed(error(
                            ErrorCode::ResourceExhausted,
                            "DOM response exceeds capture limit",
                        )),
                    );
                } else {
                    emit(
                        &sender,
                        &overflow,
                        Callback::Dom {
                            url: url.clone(),
                            generation,
                            json,
                        },
                    );
                }
            })
            .map_err(|_| error(ErrorCode::EngineFailed, "DOM capture could not start"))
    }
}

fn install_filter(
    view: &WebView,
    policy: &ValidatedBrowserRequest,
    page: Rc<RefCell<Page>>,
    sender: SyncSender<Callback>,
    overflow: Arc<std::sync::atomic::AtomicBool>,
    requested: Rc<Cell<u64>>,
    blocked: Rc<Cell<u64>>,
) -> Result<(), ApiError> {
    // No file IO inside request callbacks; use the existing embedded rule policy.
    let engine = adblock::engine("");
    let enabled = policy.request().blocking_enabled;
    let observe_media = policy.request().operation
        == crate::domain::browser_provider::BrowserOperation::ResolveEpisode;
    let environment = view.environment();
    // SAFETY: all interfaces and callbacks are created, used and released on the owning STA.
    unsafe {
        let core = view.webview();
        core.cast::<ICoreWebView2_22>()
            .and_then(|core| {
                core.AddWebResourceRequestedFilterWithRequestSourceKinds(
                    w!("*"),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                    COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
                )
            })
            .map_err(|_| {
                error(
                    ErrorCode::EngineFailed,
                    "Browser blocking filter registration failed",
                )
            })?;
        let handler = WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
            let Some(args) = args else {
                return Ok(());
            };
            requested.set(requested.get().saturating_add(1));
            let request = args.Request()?;
            let mut raw = PWSTR::null();
            request.Uri(&mut raw)?;
            let url = CoTaskMemPWSTR::from(raw).to_string();
            let mut method = PWSTR::null();
            request.Method(&mut method)?;
            let method = CoTaskMemPWSTR::from(method).to_string();
            let mut referer = PWSTR::null();
            let _ = request.Headers()?.GetHeader(w!("Referer"), &mut referer);
            let mut referer = CoTaskMemPWSTR::from(referer).to_string();
            let observed_referer = (!referer.is_empty()).then(|| referer.clone());
            let page = page.borrow();
            if referer.is_empty() {
                referer.clone_from(&page.url);
            }
            let mut context = COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
            args.ResourceContext(&mut context)?;
            if enabled
                && adblock::blocked(
                    &engine,
                    &url,
                    &referer,
                    crate::infrastructure::browser::requests::resource_kind(context),
                    &method,
                )
            {
                let response = environment.CreateWebResourceResponse(
                    None::<&IStream>,
                    403,
                    w!("Blocked"),
                    w!("Content-Type: text/plain\r\n"),
                )?;
                args.SetResponse(&response)?;
                blocked.set(blocked.get().saturating_add(1));
            } else if observe_media
                && MediaSource::from_request(url.clone(), referer).is_some()
                && crate::domain::navigation::player_page(&page.url)
                && let Some(native_navigation_id) = page.native_navigation_id
            {
                emit(
                    &sender,
                    &overflow,
                    Callback::MediaRequested(MediaRequestObservation {
                        document_url: page.url.clone(),
                        document_generation: DecimalU64(page.generation),
                        observed_unix_ms: now_ms(),
                        native_navigation_id: DecimalU64(native_navigation_id),
                        url,
                        referer: observed_referer,
                        request_method: method,
                    }),
                );
            }
            Ok(())
        }));
        let mut token = 0;
        core.add_WebResourceRequested(&handler, &mut token)
            .map_err(|_| {
                error(
                    ErrorCode::EngineFailed,
                    "Browser request callback registration failed",
                )
            })?;
    }
    Ok(())
}
