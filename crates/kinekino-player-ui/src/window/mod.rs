//! The OS window: Meari's prepared native driver (winit + softbuffer) hosting the player
//! document, with engine frames uploaded as the `<video>` texture each tick. One code path
//! for every OS; window actions (minimize/maximize/fullscreen) use Meari's action port, with
//! an X11 fallback (`window::actions`); a failed action is logged and the player keeps running.
use crate::controller::{Controller, Effect, Engine};
use crate::session::Session;
use crate::texture::VideoTexture;
pub mod actions;
mod caption;
#[cfg(all(unix, not(target_os = "macos")))]
mod edges;
use actions::{PortRefusal, WindowOps};
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
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
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
    fn run(&mut self, command: crate::session::Command) {
        if let Some(effect) = self.controller.apply(command) {
            self.effects.push(effect);
        }
        self.dirty = true;
    }

    fn collect(&mut self) {
        let handle = self.handle.clone();
        match handle.try_with(|doc| self.session.commands(doc)) {
            Ok(Ok(commands)) => {
                for command in commands {
                    self.run(command);
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
    modifiers: ModifiersState,
    frames: u64,
    remount: bool,
    closing: bool,
    capturing: bool,
    /// The player's window (first window event); on X11 its raw id is the X window.
    window_id: Option<WindowId>,
    #[cfg(all(unix, not(target_os = "macos")))]
    x11: Option<crate::window::actions::x11::X11>,
    #[cfg(all(unix, not(target_os = "macos")))]
    shown_title: String,
    /// The OS frame was turned off; the app caption row is the title bar.
    undecorated: bool,
    /// An edge drag in progress: direction, start rectangle and start pointer (screen).
    #[cfg(all(unix, not(target_os = "macos")))]
    resizing: Option<(u32, edges::Rect, (i32, i32))>,
    /// The last rectangle sent during the drag (no repeat requests for an unmoved pointer).
    #[cfg(all(unix, not(target_os = "macos")))]
    resized_to: Option<edges::Rect>,
    /// Last press on the caption row, for double-click maximize.
    caption_press: Option<Instant>,
}

impl<E: Engine> WindowOps for App<E> {
    fn port(&mut self, effect: &Effect) -> Result<(), PortRefusal> {
        let action = match *effect {
            Effect::Minimize => WindowAction::SetMinimized(true),
            Effect::Maximize(on) => WindowAction::SetMaximized(on),
            Effect::Fullscreen(on) => WindowAction::SetFullscreen(on),
            _ => return Ok(()),
        };
        self.window(action)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    fn fallback(&mut self, effect: &Effect) -> Option<Result<(), String>> {
        let (x11, id) = (self.x11.as_ref()?, self.window_id?);
        let window = u32::try_from(id.into_raw()).ok()?;
        Some(match *effect {
            Effect::Minimize => x11.minimize(window),
            Effect::Maximize(on) => x11.set_maximized(window, on),
            Effect::Fullscreen(on) => x11.set_fullscreen(window, on),
            _ => Ok(()),
        })
    }
    #[cfg(not(all(unix, not(target_os = "macos"))))]
    fn fallback(&mut self, _: &Effect) -> Option<Result<(), String>> {
        None
    }
}

impl<E: Engine> App<E> {
    #[cfg(not(all(unix, not(target_os = "macos"))))]
    fn begin_resize(&mut self, _: (f64, f64)) -> bool {
        false
    }
    #[cfg(not(all(unix, not(target_os = "macos"))))]
    fn follow_resize(&mut self, _: &WindowEvent) -> bool {
        false
    }

    fn window(&mut self, action: WindowAction) -> Result<(), PortRefusal> {
        let refusal = |e: ActionError| match e {
            ActionError::Unsupported => PortRefusal::Unsupported,
            e => PortRefusal::Other(e.to_string()),
        };
        let Some(port) = self.driver.window_actions() else {
            return Err(PortRefusal::Unsupported);
        };
        let snapshot = match port.snapshot() {
            Ok(s) => s,
            Err(ActionError::NotReady) => return Ok(()),
            Err(e) => return Err(refusal(e)),
        };
        if !snapshot.capabilities.supports(action) {
            return Err(PortRefusal::Unsupported);
        }
        self.request += 1;
        let id = NonZeroU64::new(self.request)
            .ok_or_else(|| PortRefusal::Other("window request counter".into()))?;
        port.request(snapshot.scope, id, action)
            .map(|_| ())
            .map_err(refusal)
    }

    /// The window manager's view of maximize/fullscreen where Meari's port has none (X11).
    #[cfg(all(unix, not(target_os = "macos")))]
    fn observe_fallback(&mut self) -> Option<(bool, bool)> {
        let (x11, id) = (self.x11.as_ref()?, self.window_id?);
        let window = u32::try_from(id.into_raw()).ok()?;
        let title = &self.shared.borrow().controller.view.title;
        if *title != self.shown_title && x11.set_title(window, &window_title(title)).is_ok() {
            self.shown_title = title.clone();
        }
        x11.observe(window)
    }
    #[cfg(not(all(unix, not(target_os = "macos"))))]
    fn observe_fallback(&mut self) -> Option<(bool, bool)> {
        None
    }

    /// Mirrors the OS-observed maximize/fullscreen state into the view.
    fn observe(&mut self, poll: bool) {
        let snapshot = self.driver.window_actions().and_then(|p| p.snapshot().ok());
        let ported = snapshot.and_then(|s| {
            Some((
                s.observed.maximized?,
                s.observed.borderless_monitor_coverage?,
            ))
        });
        let Some((maximized, fullscreen)) =
            ported.or_else(|| poll.then(|| self.observe_fallback()).flatten())
        else {
            return;
        };
        let mut shared = self.shared.borrow_mut();
        let view = &mut shared.controller.view;
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
                Effect::Minimize | Effect::Maximize(_) | Effect::Fullscreen(_) => {
                    if let Err(e) = crate::window::actions::perform(self, &effect) {
                        eprintln!("{{\"warning\":\"window action failed\",\"detail\":{e:?}}}");
                    }
                    // Re-read the window manager state soon after a request.
                    self.next_poll = Instant::now() + TICK * 4;
                }
                Effect::Close => self.closing = true,
                Effect::PickFile => {
                    let (picked, took) = crate::window::actions::pick_video();
                    let mut shared = self.shared.borrow_mut();
                    if shared.controller.picked(picked.as_deref(), took) {
                        shared.dirty = true;
                        if !shared.session.fits(&shared.controller.view) {
                            self.remount = true;
                            self.closing = true;
                        }
                    }
                }
            }
        }
        let poll = Instant::now() >= self.next_poll;
        self.observe(poll);
        {
            let mut shared = self.shared.borrow_mut();
            if shared.controller.idle(Instant::now()) {
                shared.dirty = true;
            }
        }
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
        self.window_id.get_or_insert(id);
        if !self.undecorated {
            self.undecorate();
        }
        if self.follow_resize(&event) {
            self.tick(event_loop);
            return;
        }
        if let WindowEvent::PointerButton {
            state: ElementState::Pressed,
            position,
            primary: true,
            button,
            ..
        } = &event
            && button.clone().mouse_button() == Some(winit::event::MouseButton::Left)
            && self.press(position.x, position.y)
        {
            self.tick(event_loop);
            return;
        }
        if matches!(event, WindowEvent::SurfaceResized(_)) {
            // A new box size re-renders the frame at that size on the next commit.
            self.shared.borrow_mut().dirty = true;
        }
        if matches!(
            event,
            WindowEvent::PointerMoved { .. }
                | WindowEvent::PointerButton { .. }
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseWheel { .. }
        ) {
            let mut shared = self.shared.borrow_mut();
            if shared.controller.input(Instant::now()) {
                shared.dirty = true;
            }
        }
        match &event {
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            WindowEvent::KeyboardInput { event: key, .. } if key.state == ElementState::Pressed => {
                if let Some(command) =
                    crate::keys::shortcut(self.modifiers, key.physical_key, key.repeat)
                {
                    self.shared.borrow_mut().run(command);
                }
            }
            _ => {}
        }
        // Space is the player's play/pause key: it must not also press a focused button.
        if let WindowEvent::KeyboardInput { event: key, .. } = &event
            && key.physical_key == PhysicalKey::Code(KeyCode::Space)
        {
            self.tick(event_loop);
            return;
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
        #[cfg(all(unix, not(target_os = "macos")))]
        self.step_resize();
        self.tick(event_loop);
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + TICK));
    }
}

/// The OS title bar text: the open file, then the product.
fn window_title(title: &str) -> String {
    if title.is_empty() {
        crate::PRODUCT.to_owned()
    } else {
        format!("{title} — {}", crate::PRODUCT)
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
                title: window_title(&current.view.title),
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
            modifiers: ModifiersState::default(),
            frames: 0,
            remount: false,
            closing: false,
            capturing: false,
            window_id: None,
            #[cfg(all(unix, not(target_os = "macos")))]
            x11: crate::window::actions::x11::X11::connect(),
            #[cfg(all(unix, not(target_os = "macos")))]
            shown_title: String::new(),
            undecorated: false,
            #[cfg(all(unix, not(target_os = "macos")))]
            resizing: None,
            #[cfg(all(unix, not(target_os = "macos")))]
            resized_to: None,
            caption_press: None,
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
