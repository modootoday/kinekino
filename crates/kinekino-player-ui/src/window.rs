//! The OS window: Meari's prepared native driver (winit + softbuffer) hosting the player
//! document, with engine frames uploaded as the `<video>` texture each tick. One code path
//! for every OS; window actions (minimize/maximize/fullscreen) use Meari's action port.
use crate::controller::{Controller, Effect, Engine};
use crate::session::Session;
use crate::texture::VideoTexture;
use kinekino_core::playback::{RgbaFrame, VideoSurface};
use kinekino_core::port::error::PortResult;
use meari::application::host_callbacks::PendingCallback;
use meari_native::application::window_actions::{ActionError, WindowAction};
use meari_native::application::window_lifecycle::{
    CleanupProgress, CleanupRequest, CompletionSignal, NativeChild,
};
use meari_native::application::{Renderer, WindowOptions};
use meari_native::domain::window_lifecycle::LifecycleLimits;
use meari_native::infrastructure::native_host::{
    ChildWindowHost, CloseDecision, CloseRequest, HostEffects, HostLifecycle, HostStamp, HostTurn,
    IntentAdmission, NativeDocumentHandle, NativeHostRuntime, NativeWindowHost,
};
use meari_native::infrastructure::window::{NativeWindowDriver, NativeWindowPreparation};
use std::cell::RefCell;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

const TICK: Duration = Duration::from_millis(16);
const POLL: Duration = Duration::from_millis(250);
const SMOKE_LIMIT: Duration = Duration::from_secs(20);

/// `smoke`: close after the first frame (and, with media, after video frames arrived).
#[derive(Debug, Default, Clone)]
pub struct Options {
    pub smoke: bool,
    pub capture: Option<PathBuf>,
}

/// What a run showed, for `--smoke` JSON and tests.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    pub frames: u64,
    pub uploads: u64,
    pub captured: bool,
    pub remounts: u32,
}

#[derive(Debug)]
pub struct HostError(String);
impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for HostError {}

/// The player window registers no child windows.
struct NoChild;
impl NativeChild for NoChild {
    type Error = std::io::Error;
    fn parent_lost(&mut self) {}
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        _: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        Err(std::io::Error::other("player window has no child windows"))
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        Err(std::io::Error::other("player window has no child windows"))
    }
}

struct Shared<E: Engine> {
    controller: Controller<E>,
    session: Session,
    handle: NativeDocumentHandle,
    texture: VideoTexture,
    effects: Vec<Effect>,
    dirty: bool,
    ready: bool,
    capture: Option<PathBuf>,
    captured: bool,
    failure: Option<String>,
}

impl<E: Engine> Shared<E> {
    fn collect(&mut self) {
        let handle = self.handle.clone();
        match handle.try_with(|doc| self.session.commands(doc)) {
            Ok(Ok(commands)) => {
                for command in commands {
                    if let Some(effect) = self.controller.apply(command) {
                        self.effects.push(effect);
                    }
                    self.dirty = true;
                }
            }
            Ok(Err(e)) => self.failure = Some(e),
            Err(e) => self.failure = Some(e.to_string()),
        }
    }

    /// Readiness is consumed when read, so it is latched until the next commit uses it.
    fn due(&mut self) -> bool {
        if self.controller.view.has_media && !self.ready {
            self.ready = self.controller.engine.frame_ready().unwrap_or(false);
        }
        self.dirty
            || self.capture.is_some()
            || (self.controller.view.has_media && (self.ready || self.texture.uploads() == 0))
    }

    fn commit(&mut self, stamp: &HostStamp) -> Result<HostEffects, HostError> {
        if !self.due() {
            return Ok(HostEffects::default());
        }
        let props = self
            .dirty
            .then(|| crate::session::meari_props(&self.controller.view));
        self.dirty = false;
        let capture = self.capture.take();
        let has_media = self.controller.view.has_media;
        let ready = std::mem::take(&mut self.ready);
        let (texture, engine) = (&mut self.texture, &mut self.controller.engine);
        let (captured, receipt) = self
            .handle
            .try_reproject(stamp, |doc| -> Result<bool, String> {
                if let Some(props) = props {
                    doc.update_module_props(props)?;
                }
                if has_media {
                    let mut surface = Latched {
                        inner: engine,
                        ready,
                    };
                    texture.upload(doc, &mut surface).map_err(|e| e.message)?;
                }
                match capture {
                    Some(path) => {
                        let viewport = doc.viewport.clone();
                        let rgba = doc.renderer.rasterize(&viewport)?;
                        kinekino_settings_ui::write_png(
                            &path,
                            viewport.width,
                            viewport.height,
                            &rgba,
                        )?;
                        Ok(true)
                    }
                    None => Ok(false),
                }
            })
            .map_err(|e| HostError(format!("{e:?}")))?;
        self.captured |= captured;
        Ok(HostEffects {
            committed: Some(receipt),
            redraw: true,
        })
    }
}

/// A surface whose readiness was already read (and latched) by the host.
struct Latched<'a, E: Engine> {
    inner: &'a mut E,
    ready: bool,
}
impl<E: Engine> VideoSurface for Latched<'_, E> {
    fn frame_ready(&mut self) -> PortResult<bool> {
        Ok(self.ready)
    }
    fn render(&mut self, frame: &mut RgbaFrame) -> PortResult<()> {
        self.inner.render(frame)
    }
}

struct Host<E: Engine>(Rc<RefCell<Shared<E>>>);

impl<E: Engine> NativeWindowHost for Host<E> {
    type Error = HostError;
    type CloseCheckpoint = ();
    fn admit_callback(&mut self, _: &HostStamp, _: &PendingCallback) -> IntentAdmission<HostError> {
        IntentAdmission::Rejected(HostError("player window has no host callbacks".into()))
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, HostError> {
        let mut shared = self.0.borrow_mut();
        shared.collect();
        shared.commit(&turn.stamp)
    }
    fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), HostError> {
        CloseDecision::HostCleanupComplete { checkpoint: () }
    }
}

struct App<E: Engine> {
    driver: NativeWindowDriver,
    shared: Rc<RefCell<Shared<E>>>,
    smoke: Option<Instant>,
    capture: Option<PathBuf>,
    next_poll: Instant,
    request: u64,
    frames: u64,
    remount: bool,
    closing: bool,
    capturing: bool,
}

impl<E: Engine> App<E> {
    fn window(&mut self, action: WindowAction) -> Result<(), String> {
        let Some(port) = self.driver.window_actions() else {
            return Err("window actions unavailable".into());
        };
        let scope = match port.snapshot() {
            Ok(s) => s.scope,
            Err(ActionError::NotReady) => return Ok(()),
            Err(e) => return Err(e.to_string()),
        };
        self.request += 1;
        let id = NonZeroU64::new(self.request).ok_or("window request counter")?;
        port.request(scope, id, action)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Mirrors the OS-observed maximize/fullscreen state into the view.
    fn observe(&mut self) {
        let Some(snapshot) = self.driver.window_actions().and_then(|p| p.snapshot().ok()) else {
            return;
        };
        let mut shared = self.shared.borrow_mut();
        let view = &mut shared.controller.view;
        let maximized = snapshot.observed.maximized.unwrap_or(view.maximized);
        let fullscreen = snapshot
            .observed
            .borderless_monitor_coverage
            .unwrap_or(view.fullscreen);
        if (maximized, fullscreen) != (view.maximized, view.fullscreen) {
            view.maximized = maximized;
            view.fullscreen = fullscreen;
            shared.dirty = true;
        }
    }

    fn advance(&mut self) -> Result<(), String> {
        let wake = self.driver.wake_handle();
        if let Some(f) = self.driver.last_submission() {
            self.frames = self.frames.max(f.ticket.scene_generation);
        }
        let effects = std::mem::take(&mut self.shared.borrow_mut().effects);
        for effect in effects {
            match effect {
                Effect::Minimize => self.window(WindowAction::SetMinimized(true))?,
                Effect::Maximize(on) => self.window(WindowAction::SetMaximized(on))?,
                Effect::Fullscreen(on) => self.window(WindowAction::SetFullscreen(on))?,
                Effect::Close => self.closing = true,
                Effect::PickFile => {
                    let picked = rfd::FileDialog::new()
                        .add_filter("동영상", &["mkv", "mp4", "webm", "avi", "mov", "m4v", "ts"])
                        .pick_file();
                    if let Some(path) = picked {
                        let mut shared = self.shared.borrow_mut();
                        shared.controller.open(&path);
                        shared.dirty = true;
                        if !shared.session.fits(&shared.controller.view) {
                            self.remount = true;
                            self.closing = true;
                        }
                    }
                }
            }
        }
        self.observe();
        if Instant::now() >= self.next_poll {
            self.next_poll = Instant::now() + POLL;
            let mut shared = self.shared.borrow_mut();
            if shared.controller.poll() {
                shared.dirty = true;
            }
        }
        if let Some(started) = self.smoke {
            let shared = self.shared.borrow();
            let shown = self.frames > 0
                && (!shared.controller.view.has_media || shared.texture.uploads() >= 2);
            drop(shared);
            if shown && self.capture.is_some() {
                self.shared.borrow_mut().capture = self.capture.take();
                self.capturing = true;
            } else if shown && (!self.capturing || self.shared.borrow().captured) {
                self.closing = true;
            } else if started.elapsed() > SMOKE_LIMIT {
                return Err("player smoke: no video frame within the time limit".into());
            }
        }
        if self.closing {
            wake.request_close().map_err(|e| e.to_string())?;
        } else if self.shared.borrow_mut().due() {
            wake.wake().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn tick(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.shared.borrow().handle.lifecycle() == HostLifecycle::Closed {
            event_loop.exit();
            return;
        }
        if let Err(e) = self.advance() {
            self.shared.borrow_mut().failure = Some(e);
        }
        if self.shared.borrow().failure.is_some() {
            event_loop.exit();
        }
    }
}

impl<E: Engine> ApplicationHandler for App<E> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.driver.can_create_surfaces(event_loop);
        self.tick(event_loop);
    }
    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.driver.destroy_surfaces(event_loop);
    }
    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::SurfaceResized(_)) {
            // A new box size re-renders the frame at that size on the next commit.
            self.shared.borrow_mut().dirty = true;
        }
        self.driver.window_event(event_loop, id, event);
        self.tick(event_loop);
    }
    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.driver.proxy_wake_up(event_loop);
        self.tick(event_loop);
    }
    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.driver.about_to_wait(event_loop);
        self.tick(event_loop);
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + TICK));
    }
}

/// Runs the player window until closed. Opening a folder with a different episode count
/// remounts the document (the window is rebuilt with the new rows).
pub fn run<E: Engine>(controller: Controller<E>, options: Options) -> Result<Report, String> {
    let mut controller = Some(controller);
    let mut report = Report::default();
    let mut event_loop = blitz_shell::create_default_event_loop();
    let mut capture = options.capture.clone();
    loop {
        let mut current = controller.take().ok_or("player controller lost")?;
        let (session, document) = Session::mount(&current.view)?;
        current.view.status.clear();
        let handle = NativeDocumentHandle::new(document, 1).map_err(|e| e.to_string())?;
        let preparation = NativeWindowPreparation::<NoChild>::new(
            &event_loop,
            handle.clone(),
            LifecycleLimits::new(1, Duration::from_secs(2)).map_err(|e| format!("{e:?}"))?,
            WindowOptions {
                title: crate::PRODUCT.into(),
            },
            None,
        )
        .map_err(|e| format!("{e:?}"))?;
        let shared = Rc::new(RefCell::new(Shared {
            controller: current,
            session,
            handle: handle.clone(),
            texture: VideoTexture::default(),
            effects: Vec::new(),
            dirty: false,
            ready: false,
            capture: None,
            captured: false,
            failure: None,
        }));
        let runtime = Rc::new(RefCell::new(NativeHostRuntime::new(
            handle,
            ChildWindowHost::new(Host(shared.clone()), preparation.children()),
        )));
        let driver = preparation
            .attach(runtime)
            .map_err(|e| format!("player window attach: {:?}", e.problem))?;
        let mut app = App {
            driver,
            shared: shared.clone(),
            smoke: options.smoke.then(Instant::now),
            capture: capture.take(),
            next_poll: Instant::now(),
            request: 0,
            frames: 0,
            remount: false,
            closing: false,
            capturing: false,
        };
        winit::event_loop::run_on_demand::EventLoopExtRunOnDemand::run_app_on_demand(
            &mut event_loop,
            &mut app,
        )
        .map_err(|e| e.to_string())?;
        let remount = app.remount;
        report.frames += app.frames;
        drop(app);
        let shared = Rc::try_unwrap(shared)
            .map_err(|_| "player window state still shared")?
            .into_inner();
        if let Some(e) = shared.failure {
            return Err(e);
        }
        report.uploads += shared.texture.uploads();
        report.captured |= shared.captured;
        if !remount {
            return Ok(report);
        }
        report.remounts += 1;
        controller = Some(shared.controller);
    }
}
