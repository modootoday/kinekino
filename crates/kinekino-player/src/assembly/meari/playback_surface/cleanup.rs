use meari_native::application::window_lifecycle::{
    CleanupProgress, CleanupRequest, CompletionSignal, NativeChild, WindowStamp,
};
use std::{error::Error, fmt, marker::PhantomData, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseStage {
    Engine,
    Surface,
}

#[derive(Clone, Debug)]
pub enum PlaybackSurfaceError<E, S> {
    Identity,
    Engine(E),
    Surface(S),
    ObservationDiagnosticMissing(ReleaseStage),
}
impl<E: Error, S: Error> fmt::Display for PlaybackSurfaceError<E, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identity => f.write_str("playback surface cleanup identity mismatch"),
            Self::Engine(error) => write!(f, "engine cleanup: {error}"),
            Self::Surface(error) => write!(f, "surface cleanup: {error}"),
            Self::ObservationDiagnosticMissing(stage) => {
                write!(f, "{stage:?} cleanup is unobservable without a diagnostic")
            }
        }
    }
}
impl<E: Error + 'static, S: Error + 'static> Error for PlaybackSurfaceError<E, S> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Engine(error) => Some(error),
            Self::Surface(error) => Some(error),
            _ => None,
        }
    }
}
type Outcome<E, S> = Result<CleanupProgress, PlaybackSurfaceError<E, S>>;
struct Attempt<E, S> {
    request: CleanupRequest,
    signal: CompletionSignal,
    progress: CleanupProgress,
    terminal: Option<Outcome<E, S>>,
}

/// The trusted registrar must supply both original components for the same child.
/// The concrete surface adapter must own actual release evidence; this type cannot mint it.
/// No component may have a second cleanup driver outside this outer coordinator child.
pub struct PlaybackSurfaceChild<E, S>
where
    E: NativeChild,
    S: NativeChild,
    E::Error: Clone,
    S::Error: Clone,
{
    window: WindowStamp,
    child: u64,
    engine: E,
    surface: S,
    engine_complete: bool,
    surface_started: bool,
    attempt: Option<Box<Attempt<E::Error, S::Error>>>,
    engine_failure: Option<E::Error>,
    surface_failure: Option<S::Error>,
    observation_failure: Option<PlaybackSurfaceError<E::Error, S::Error>>,
    unavailable: bool,
    _ui_thread: PhantomData<Rc<()>>,
}
impl<E, S> PlaybackSurfaceChild<E, S>
where
    E: NativeChild,
    S: NativeChild,
    E::Error: Clone,
    S::Error: Clone,
{
    pub fn new(window: WindowStamp, child: u64, engine: E, surface: S) -> Self {
        Self {
            window,
            child,
            engine,
            surface,
            engine_complete: false,
            surface_started: false,
            attempt: None,
            engine_failure: None,
            surface_failure: None,
            observation_failure: None,
            unavailable: false,
            _ui_thread: PhantomData,
        }
    }
    pub fn engine_failure(&self) -> Option<&E::Error> {
        self.engine_failure.as_ref()
    }
    pub fn surface_failure(&self) -> Option<&S::Error> {
        self.surface_failure.as_ref()
    }
    /// Last observation diagnostic remains owned after recovery; it is never a completion proof.
    pub fn retained_observation_error(&self) -> Option<&PlaybackSurfaceError<E::Error, S::Error>> {
        self.observation_failure.as_ref()
    }
    fn same(&self, request: CleanupRequest) -> bool {
        self.attempt
            .as_ref()
            .is_some_and(|a| a.request.stamp == request.stamp && a.request.reason == request.reason)
    }
    fn reject(&self) -> Outcome<E::Error, S::Error> {
        // Invalid callers cannot invoke either component or replace the original signal.
        match &self.attempt {
            Some(a) if a.terminal.is_none() => Ok(a.progress),
            _ => Err(PlaybackSurfaceError::Identity),
        }
    }
    fn record(
        &mut self,
        stage: ReleaseStage,
        result: Outcome<E::Error, S::Error>,
    ) -> Outcome<E::Error, S::Error> {
        self.unavailable = matches!(result, Ok(CleanupProgress::ObservationUnavailable));
        if self.unavailable {
            self.observation_failure = Some(
                match stage {
                    ReleaseStage::Engine => self
                        .engine
                        .observation_error()
                        .cloned()
                        .map(PlaybackSurfaceError::Engine),
                    ReleaseStage::Surface => self
                        .surface
                        .observation_error()
                        .cloned()
                        .map(PlaybackSurfaceError::Surface),
                }
                .unwrap_or(PlaybackSurfaceError::ObservationDiagnosticMissing(stage)),
            );
        }
        let attempt = self.attempt.as_mut().expect("original cleanup attempt");
        match &result {
            Ok(progress) => {
                attempt.progress = *progress;
                if *progress == CleanupProgress::Complete {
                    attempt.terminal = Some(result.clone());
                }
            }
            Err(error) => {
                match error {
                    PlaybackSurfaceError::Engine(error) => {
                        self.engine_failure = Some(error.clone())
                    }
                    PlaybackSurfaceError::Surface(error) => {
                        self.surface_failure = Some(error.clone())
                    }
                    _ => (),
                }
                attempt.terminal = Some(result.clone());
            }
        }
        result
    }
    fn drive(&mut self, begin_engine: bool) -> Outcome<E::Error, S::Error> {
        let attempt = self
            .attempt
            .as_ref()
            .ok_or(PlaybackSurfaceError::Identity)?;
        if let Some(terminal) = &attempt.terminal {
            return terminal.clone();
        }
        let request = attempt.request;
        let signal = attempt.signal.clone();
        if !self.engine_complete {
            let outcome = if begin_engine {
                self.engine.begin_cleanup(request, signal.clone())
            } else {
                self.engine.observe_cleanup(request)
            }
            .map_err(PlaybackSurfaceError::Engine);
            match outcome {
                Ok(CleanupProgress::Complete) => self.engine_complete = true,
                other => return self.record(ReleaseStage::Engine, other),
            }
        }
        let outcome = if !self.surface_started {
            self.surface_started = true;
            self.surface.begin_cleanup(request, signal)
        } else {
            self.surface.observe_cleanup(request)
        }
        .map_err(PlaybackSurfaceError::Surface);
        self.record(ReleaseStage::Surface, outcome)
    }
}
impl<E, S> NativeChild for PlaybackSurfaceChild<E, S>
where
    E: NativeChild,
    S: NativeChild,
    E::Error: Clone,
    S::Error: Clone,
{
    type Error = PlaybackSurfaceError<E::Error, S::Error>;
    fn parent_lost(&mut self) {
        self.engine.parent_lost();
        self.surface.parent_lost();
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Outcome<E::Error, S::Error> {
        if request.stamp.window() != self.window
            || request.stamp.child() != self.child
            || signal.stamp() != request.stamp
        {
            return self.reject();
        }
        if self.same(request) {
            return self.drive(false);
        }
        if let Some(attempt) = &self.attempt
            && (request.stamp.revocation() <= attempt.request.stamp.revocation()
                || request.stamp.attempt() <= attempt.request.stamp.attempt()
                || !matches!(attempt.terminal.as_ref(), Some(Err(_))))
        {
            return self.reject();
        }
        self.attempt = Some(Box::new(Attempt {
            request,
            signal,
            progress: CleanupProgress::Pending,
            terminal: None,
        }));
        self.surface_started = false;
        self.drive(true)
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Outcome<E::Error, S::Error> {
        if !self.same(request) {
            return self.reject();
        }
        self.drive(false)
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        if self.unavailable {
            self.observation_failure.as_ref()
        } else {
            None
        }
    }
}
