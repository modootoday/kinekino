use super::{
    Command, Shared,
    capture::{Callback, Capture, now_ms},
    error,
    window::{Apartment, HiddenWindow},
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        browser_provider::*,
    },
    domain::api_values::DecimalU64,
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    time::{Duration, Instant},
};

struct Active {
    ticket: BrowserTicket,
    capture: Capture,
    deadline: Instant,
}

pub(super) fn run(
    profile: PathBuf,
    shared: Arc<Mutex<Shared>>,
    shutdown: Arc<AtomicBool>,
    commands: Receiver<Command>,
) {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_inner(profile, &shared, &shutdown, commands)
    }));
    let normal_return = matches!(&outcome, Ok(Ok(())));
    let failure = match outcome {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error),
        Err(_) => Some(error(
            ErrorCode::EngineFailed,
            "Browser worker terminated unexpectedly",
        )),
    };
    if let Ok(mut state) = shared.lock() {
        for work in &mut state.works {
            if !work.snapshot().state.terminal() {
                let ticket = work.snapshot().ticket;
                work.apply(
                    &ticket,
                    BrowserEvent::Failed(error(
                        ErrorCode::EngineFailed,
                        "Browser host stopped before work completed",
                    )),
                );
            }
            // Normal return is after run_inner dropped its original Capture.
            // Error/unwind is not promoted to a native release observation.
            if normal_return {
                work.record_native_capture_released().ok();
            }
        }
        state.host.state = if failure.is_some() {
            BrowserHostState::Failed
        } else {
            BrowserHostState::Stopped
        };
        state.host.worker_exited = true;
        state.host.failure = failure.map(Arc::new);
    }
}

fn run_inner(
    profile: PathBuf,
    shared: &Arc<Mutex<Shared>>,
    shutdown: &AtomicBool,
    commands: Receiver<Command>,
) -> Result<(), ApiError> {
    let _apartment = Apartment::new()?;
    let window = HiddenWindow::new()?;
    let mut context = wry::WebContext::new(Some(profile));
    let engine_version = wry::webview_version()
        .map_err(|_| error(ErrorCode::EngineFailed, "WebView2 runtime is unavailable"))?;
    shared
        .lock()
        .map_err(|_| error(ErrorCode::Internal, "Browser state unavailable"))?
        .host
        .state = BrowserHostState::Ready;
    let mut active: Option<Active> = None;
    while !shutdown.load(Ordering::Acquire) {
        for _ in 0..MAX_COMMANDS {
            let command = match commands.try_recv() {
                Ok(command) => command,
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    shutdown.store(true, Ordering::Release);
                    break;
                }
            };
            match command {
                Command::Start(ticket) => {
                    release_active(&mut active, shared)?;
                    let request = {
                        let mut state = shared
                            .lock()
                            .map_err(|_| error(ErrorCode::Internal, "Browser state unavailable"))?;
                        let work = super::work(&mut state, &ticket)?;
                        if work.snapshot().state.terminal() {
                            work.record_native_capture_released()?;
                            continue;
                        }
                        if work.snapshot().state == BrowserWorkState::Cancelling {
                            work.apply(&ticket, BrowserEvent::Cancelled);
                            work.record_native_capture_released()?;
                            continue;
                        }
                        work.request().clone()
                    };
                    let deadline =
                        Instant::now() + Duration::from_millis(request.request().timeout_ms.0);
                    let result = Capture::new(&window, &mut context, &request);
                    match result {
                        Ok(mut capture) => {
                            apply(
                                shared,
                                &ticket,
                                BrowserEvent::Started {
                                    engine_version: engine_version.clone(),
                                    blocking_installed: true,
                                },
                            )?;
                            if let Err(error) = capture.navigate(&request.request().url) {
                                apply(shared, &ticket, BrowserEvent::Failed(error))?;
                            }
                            active = Some(Active {
                                ticket,
                                capture,
                                deadline,
                            });
                        }
                        Err(error) => {
                            apply(shared, &ticket, BrowserEvent::Failed(error))?;
                            record_capture_released(shared, &ticket)?;
                        }
                    }
                }
                Command::Navigate(ticket, url) => {
                    if let Some(current) = &mut active
                        && current.ticket == ticket
                        && let Err(error) = current.capture.navigate(&url)
                    {
                        apply(shared, &ticket, BrowserEvent::Failed(error))?;
                    }
                }
                Command::ActivateEpisode(ticket, generation) => {
                    let current_generation = {
                        let mut state = shared
                            .lock()
                            .map_err(|_| error(ErrorCode::Internal, "Browser state unavailable"))?;
                        super::work(&mut state, &ticket)
                            .ok()
                            .map(|work| work.snapshot())
                            .filter(|snapshot| {
                                snapshot.state == BrowserWorkState::Running
                                    && snapshot.activation.as_ref().is_some_and(|activation| {
                                        activation.status == EpisodeActivationStatus::Enqueued
                                    })
                            })
                            .map(|snapshot| snapshot.document_generation.0)
                    };
                    if current_generation != Some(generation) {
                        continue;
                    }
                    if let Some(current) = &mut active
                        && current.ticket == ticket
                        && let Err(failure) = current.capture.activate_episode(generation)
                        && failure.code != ErrorCode::RevisionConflict
                    {
                        apply(shared, &ticket, BrowserEvent::Failed(failure))?;
                    }
                }
                Command::Cancel(ticket) => {
                    if active
                        .as_ref()
                        .is_some_and(|current| current.ticket == ticket)
                    {
                        active = None;
                    }
                    apply(shared, &ticket, BrowserEvent::Cancelled)?;
                    record_capture_released(shared, &ticket)?;
                }
            }
        }
        window.pump();
        if let Some(current) = &mut active {
            let ticket = &current.ticket;
            if Instant::now() >= current.deadline {
                let mut state = shared
                    .lock()
                    .map_err(|_| error(ErrorCode::Internal, "Browser state unavailable"))?;
                if let Ok(work) = super::work(&mut state, ticket) {
                    work.expire();
                }
            } else if current.capture.overflowed() {
                apply(
                    shared,
                    ticket,
                    BrowserEvent::Failed(error(
                        ErrorCode::ResourceExhausted,
                        "Browser callback queue overflow",
                    )),
                )?;
            } else {
                let (requested, blocked) = current.capture.counts();
                apply(shared, ticket, BrowserEvent::Counts { requested, blocked })?;
                let mut native_drained = false;
                for _ in 0..MAX_COMMANDS {
                    let Some(event) = current.capture.next() else {
                        native_drained = true;
                        break;
                    };
                    match event {
                        Callback::Navigation(url, document_generation, native_navigation_id) => {
                            apply(
                                shared,
                                ticket,
                                BrowserEvent::Navigated {
                                    url,
                                    document_generation,
                                },
                            )?;
                            apply(
                                shared,
                                ticket,
                                BrowserEvent::NavigationIdentity {
                                    document_generation,
                                    native_navigation_id,
                                },
                            )?;
                        }
                        Callback::NavigationCompleted(completion) => apply(
                            shared,
                            ticket,
                            BrowserEvent::NavigationCompleted(completion),
                        )?,
                        Callback::ActivationResult(
                            document_generation,
                            native_navigation_id,
                            invoked,
                        ) => apply(
                            shared,
                            ticket,
                            if invoked {
                                BrowserEvent::ActivationInvoked {
                                    document_generation,
                                    native_navigation_id,
                                }
                            } else {
                                BrowserEvent::ActivationRejected {
                                    document_generation,
                                    native_navigation_id,
                                }
                            },
                        )?,
                        Callback::PlayerFrame(frame) => {
                            apply(shared, ticket, BrowserEvent::PlayerFrame(frame))?
                        }
                        Callback::FrameDocument(event) => {
                            apply(shared, ticket, BrowserEvent::FrameDocument(event))?;
                        }
                        Callback::FrameRuntime(request, json, observed_unix_ms) => {
                            apply(
                                shared,
                                ticket,
                                BrowserEvent::FrameRuntime {
                                    request,
                                    json,
                                    observed_unix_ms,
                                },
                            )?;
                        }
                        Callback::FrameDom(identity, json) => {
                            match decode_dom(
                                &json,
                                identity.url.clone(),
                                identity.document_generation.0,
                            ) {
                                Ok(Some(document)) => apply(
                                    shared,
                                    ticket,
                                    BrowserEvent::FrameDocument(FrameDocumentEvent::Document(
                                        identity, document,
                                    )),
                                )?,
                                Ok(None) => {}
                                Err(error) => apply(shared, ticket, BrowserEvent::Failed(error))?,
                            }
                        }
                        Callback::Loaded(url) => {
                            if let Err(error) = current.capture.request_dom(&url) {
                                apply(shared, ticket, BrowserEvent::Failed(error))?;
                            }
                        }
                        Callback::Dom {
                            url,
                            generation,
                            json,
                        } => {
                            // JSON string decoding only. HTML parse/publication belongs to the SDK executor.
                            match decode_dom(&json, url, generation) {
                                Ok(Some(document)) => {
                                    apply(shared, ticket, BrowserEvent::DocumentReady(document))?;
                                }
                                Ok(None) => {}
                                Err(error) => {
                                    apply(shared, ticket, BrowserEvent::Failed(error))?;
                                }
                            }
                        }
                        Callback::MediaRequested(candidate) => {
                            apply(shared, ticket, BrowserEvent::MediaRequested(candidate))?;
                        }
                        Callback::Failed(error) => {
                            apply(shared, ticket, BrowserEvent::Failed(error))?;
                        }
                    }
                }
                super::network_worker::drive(&mut current.capture, shared, ticket, native_drained)?;
                super::runtime_worker::drive(&current.capture, shared, ticket, native_drained)?;
            }
            let terminal = {
                let mut state = shared
                    .lock()
                    .map_err(|_| error(ErrorCode::Internal, "Browser state unavailable"))?;
                super::work(&mut state, ticket)
                    .map_or(true, |work| work.snapshot().state.terminal())
            };
            if terminal {
                release_active(&mut active, shared)?;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if let Ok(mut state) = shared.lock() {
        state.host.state = BrowserHostState::Stopping;
    }
    drop(active);
    Ok(())
}

fn release_active(
    active: &mut Option<Active>,
    shared: &Arc<Mutex<Shared>>,
) -> Result<(), ApiError> {
    if let Some(original) = active.take() {
        let ticket = original.ticket.clone();
        // Same STA, same original native resource; record only after Drop returns.
        drop(original);
        record_capture_released(shared, &ticket)?;
    }
    Ok(())
}
fn record_capture_released(
    shared: &Arc<Mutex<Shared>>,
    ticket: &BrowserTicket,
) -> Result<(), ApiError> {
    let mut state = shared
        .lock()
        .map_err(|_| error(ErrorCode::Internal, "Browser state unavailable"))?;
    if let Ok(original) = super::work(&mut state, ticket) {
        original.record_native_capture_released()?;
    }
    Ok(())
}

pub(super) fn apply(
    shared: &Arc<Mutex<Shared>>,
    ticket: &BrowserTicket,
    event: BrowserEvent,
) -> Result<(), ApiError> {
    let mut state = shared
        .lock()
        .map_err(|_| error(ErrorCode::Internal, "Browser state unavailable"))?;
    // Explicit release can race already queued callback delivery after terminal completion.
    if let Ok(work) = super::work(&mut state, ticket) {
        work.apply(ticket, event);
    }
    Ok(())
}

pub(super) fn decode_dom(
    json: &str,
    url: String,
    generation: u64,
) -> Result<Option<BrowserDocument>, ApiError> {
    match serde_json::from_str::<Option<(String, String, bool)>>(json) {
        Ok(Some((observed_url, _, _))) if observed_url != url => Ok(None),
        Ok(Some((_, html, truncated))) => {
            if truncated || html.len() > MAX_CAPTURE_BYTES {
                return Err(error(
                    ErrorCode::ResourceExhausted,
                    "Browser DOM capture exceeded its limit; truncated content was discarded",
                ));
            }
            Ok(Some(BrowserDocument {
                kind: CaptureKind::DomSnapshot,
                final_url: url,
                document_generation: DecimalU64(generation),
                observed_unix_ms: now_ms(),
                body: Arc::from(html.into_bytes()),
                response: None,
            }))
        }
        _ => Err(error(
            ErrorCode::EngineFailed,
            "Browser DOM capture was invalid",
        )),
    }
}
