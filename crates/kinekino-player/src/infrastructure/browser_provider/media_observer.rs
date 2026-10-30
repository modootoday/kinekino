//! Deferred manifest-prefix observation. Not registered in the first DOM checkpoint.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        browser_provider::{BrowserMedia, BrowserTicket},
    },
    domain::{api_values::DecimalU64, media::MediaSource, navigation},
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    sync::Arc,
};
use webview2_com::{
    CoTaskMemPWSTR,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2_2, ICoreWebView2WebResourceRequest},
    WebResourceResponseReceivedEventHandler, WebResourceResponseViewGetContentCompletedHandler,
};
use windows::{
    Win32::System::Com::IStream,
    core::{Interface, PWSTR, w},
};
use wry::{WebView, WebViewExtWindows};

const MAX_PENDING: usize = 4;
const MAX_PREFIX: usize = 4096;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ObservationIdentity {
    pub ticket: BrowserTicket,
    pub document_url: String,
    pub document_generation: u64,
}
impl ObservationIdentity {
    fn valid(&self) -> bool {
        self.document_generation > 0
            && self.document_url.len() <= 8192
            && navigation::player_page(&self.document_url)
    }
}

// The caller correlates the original request to its frozen ticket/document generation.
// Looking up only the current page at response time is not sufficient for a late response.
pub(super) type RequestIdentity =
    Rc<dyn Fn(&ICoreWebView2WebResourceRequest) -> Option<ObservationIdentity>>;
pub(super) type IsCurrent = Rc<dyn Fn(&ObservationIdentity) -> bool>;

pub(super) enum MediaObservation {
    Media {
        ticket: BrowserTicket,
        media: BrowserMedia,
    },
    Failed {
        ticket: BrowserTicket,
        error: ApiError,
    },
}

struct PendingPrefix {
    identity: ObservationIdentity,
    url: String,
    referer: String,
    observed_unix_ms: DecimalU64,
    stream: IStream,
}
#[derive(Default)]
struct Pending {
    waiting: usize,
    ready: VecDeque<PendingPrefix>,
    failure: Option<(ObservationIdentity, ApiError)>,
    closed: bool,
}
impl Pending {
    fn fail(&mut self, identity: ObservationIdentity, message: &str) {
        if !self.closed && self.failure.is_none() {
            self.failure = Some((identity, ApiError::new(ErrorCode::EngineFailed, message)));
        }
    }
}

/// All COM handles, streams and closures remain on the owning STA.
pub(super) struct MediaObserver {
    core: ICoreWebView2_2,
    token: i64,
    pending: Rc<RefCell<Pending>>,
    current: IsCurrent,
}

impl MediaObserver {
    pub fn install(
        view: &WebView,
        identity: RequestIdentity,
        current: IsCurrent,
    ) -> Result<Self, ApiError> {
        let pending = Rc::new(RefCell::new(Pending::default()));
        let callback_pending = pending.clone();
        let callback_current = current.clone();
        // SAFETY: registration, callbacks, deferred stream reads and removal use one STA.
        unsafe {
            let core = view
                .webview()
                .cast::<ICoreWebView2_2>()
                .map_err(|_| engine_error("Media response observation is unavailable"))?;
            let handler =
                WebResourceResponseReceivedEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else {
                        return Ok(());
                    };
                    let request = args.Request()?;
                    let Some(frozen) =
                        identity(&request).filter(|id| id.valid() && callback_current(id))
                    else {
                        return Ok(());
                    };
                    let mut raw = PWSTR::null();
                    request.Uri(&mut raw)?;
                    let url = CoTaskMemPWSTR::from(raw).to_string();
                    let response = args.Response()?;
                    let mut mime = PWSTR::null();
                    let _ = response.Headers()?.GetHeader(w!("Content-Type"), &mut mime);
                    let mime = CoTaskMemPWSTR::from(mime).to_string();
                    if !should_inspect(&url, &mime) {
                        return Ok(());
                    }
                    let mut status = 0;
                    response.StatusCode(&mut status)?;
                    if !(200..300).contains(&status) {
                        return Ok(());
                    }
                    let mut referer = PWSTR::null();
                    let _ = request.Headers()?.GetHeader(w!("Referer"), &mut referer);
                    let mut referer = CoTaskMemPWSTR::from(referer).to_string();
                    if referer.is_empty() {
                        referer.clone_from(&frozen.document_url);
                    }
                    if referer.len() > 8192 || !navigation::navigation_allowed(&referer) {
                        return Ok(());
                    }
                    {
                        let mut queue = callback_pending.borrow_mut();
                        if queue.closed {
                            return Ok(());
                        }
                        if queue.waiting + queue.ready.len() >= MAX_PENDING {
                            queue.fail(frozen, "Media response capture queue limit reached");
                            return Ok(());
                        }
                        queue.waiting += 1;
                    }
                    let completion_pending = callback_pending.clone();
                    let completion_identity = frozen.clone();
                    let completion_current = callback_current.clone();
                    let completed = Rc::new(Cell::new(false));
                    let callback_completed = completed.clone();
                    let observed_unix_ms = observed_ms();
                    let completion = WebResourceResponseViewGetContentCompletedHandler::create(
                        Box::new(move |result, stream| {
                            if callback_completed.replace(true) {
                                return Ok(());
                            }
                            let mut queue = completion_pending.borrow_mut();
                            queue.waiting = queue.waiting.saturating_sub(1);
                            if queue.closed || !completion_current(&completion_identity) {
                                return Ok(());
                            }
                            if result.is_err() {
                                queue.fail(
                                    completion_identity.clone(),
                                    "Media response body acquisition failed",
                                );
                                return Ok(());
                            }
                            let Some(stream) = stream else {
                                queue.fail(
                                    completion_identity.clone(),
                                    "Media response body was unavailable",
                                );
                                return Ok(());
                            };
                            // No COM Read, HTML parse, disk write or network wait in this callback.
                            queue.ready.push_back(PendingPrefix {
                                identity: completion_identity.clone(),
                                url: url.clone(),
                                referer: referer.clone(),
                                observed_unix_ms,
                                stream,
                            });
                            Ok(())
                        }),
                    );
                    if response.GetContent(&completion).is_err() && !completed.replace(true) {
                        let mut queue = callback_pending.borrow_mut();
                        queue.waiting = queue.waiting.saturating_sub(1);
                        queue.fail(frozen, "Media response body acquisition could not start");
                    }
                    Ok(())
                }));
            let mut token = 0;
            core.add_WebResourceResponseReceived(&handler, &mut token)
                .map_err(|_| engine_error("Media response handler registration failed"))?;
            Ok(Self {
                core,
                token,
                pending,
                current,
            })
        }
    }

    /// Called from the STA worker outside callbacks; consumes at most one 4096-byte prefix.
    pub fn poll_one(&mut self) -> Option<MediaObservation> {
        let failure = self.pending.borrow_mut().failure.take();
        if let Some((identity, error)) = failure
            && (self.current)(&identity)
        {
            return Some(MediaObservation::Failed {
                ticket: identity.ticket,
                error,
            });
        }
        let pending = self.pending.borrow_mut().ready.pop_front()?;
        if !(self.current)(&pending.identity) {
            return None;
        }
        let mut prefix = [0u8; MAX_PREFIX];
        let mut count = 0;
        // SAFETY: stream never leaves this STA, and the buffer length matches the requested read.
        let result = unsafe {
            pending.stream.Read(
                prefix.as_mut_ptr().cast(),
                MAX_PREFIX as u32,
                Some(&mut count),
            )
        };
        if result.is_err() || count as usize > MAX_PREFIX {
            return Some(MediaObservation::Failed {
                ticket: pending.identity.ticket,
                error: engine_error("Manifest prefix read failed"),
            });
        }
        if !(self.current)(&pending.identity) {
            return None;
        }
        observed_media(
            pending.identity,
            pending.url,
            pending.referer,
            pending.observed_unix_ms,
            &prefix[..count as usize],
        )
    }
}

fn observed_media(
    identity: ObservationIdentity,
    url: String,
    referer: String,
    observed_unix_ms: DecimalU64,
    prefix: &[u8],
) -> Option<MediaObservation> {
    if !identity.valid() || prefix.len() > MAX_PREFIX {
        return None;
    }
    let source = MediaSource::from_manifest(url, referer, prefix)?;
    Some(MediaObservation::Media {
        ticket: identity.ticket,
        media: BrowserMedia {
            document_url: identity.document_url,
            document_generation: DecimalU64(identity.document_generation),
            observed_unix_ms,
            source,
            manifest_prefix: Some(Arc::from(prefix)),
        },
    })
}

fn should_inspect(url: &str, mime: &str) -> bool {
    if url.len() > 8192 || mime.len() > 1024 {
        return false;
    }
    url::Url::parse(url).is_ok_and(|url| {
        url.scheme() == "https"
            && (mime.contains("mpegurl")
                || (url.host_str() == Some("media.example.test") && url.path().ends_with(".txt")))
    })
}
fn engine_error(message: &str) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, message)
}
fn observed_ms() -> DecimalU64 {
    DecimalU64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u64::MAX as u128) as u64,
    )
}
impl Drop for MediaObserver {
    fn drop(&mut self) {
        let mut pending = self.pending.borrow_mut();
        pending.closed = true;
        pending.ready.clear();
        pending.failure = None;
        // SAFETY: observer is dropped on its creating STA before the owning WebView/context.
        unsafe {
            let _ = self.core.remove_WebResourceResponseReceived(self.token);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifest_prefix_keeps_frozen_identity_and_is_not_a_complete_document() {
        let identity = ObservationIdentity {
            ticket: BrowserTicket::new("fixture-session".into(), "fixture-request".into(), 2)
                .unwrap(),
            document_url: "https://media.example.test/video/one".into(),
            document_generation: 7,
        };
        let prefix = b"#EXTM3U\n#EXT-X-VERSION:3\n";
        let Some(MediaObservation::Media { ticket, media }) = observed_media(
            identity.clone(),
            "https://media.example.test/a.txt".into(),
            identity.document_url.clone(),
            DecimalU64(10),
            prefix,
        ) else {
            panic!("expected manifest evidence");
        };
        assert_eq!(ticket, identity.ticket);
        assert_eq!(media.document_generation.0, 7);
        assert_eq!(media.manifest_prefix.as_deref(), Some(prefix.as_slice()));
        assert!(media.source.hls);
        assert!(
            observed_media(
                identity.clone(),
                "https://media.example.test/a.txt".into(),
                identity.document_url,
                DecimalU64(10),
                &vec![b'x'; MAX_PREFIX + 1]
            )
            .is_none()
        );
    }
    #[test]
    fn mime_and_url_scope_preserve_existing_manifest_probe_policy() {
        assert!(should_inspect("https://media.example.test/a.txt", "text/plain"));
        assert!(should_inspect(
            "https://cdn.example/a",
            "application/vnd.apple.mpegurl"
        ));
        assert!(!should_inspect("http://media.example.test/a.txt", "text/plain"));
        assert!(!should_inspect("https://media.example.test/a.png", "image/png"));
    }
}
