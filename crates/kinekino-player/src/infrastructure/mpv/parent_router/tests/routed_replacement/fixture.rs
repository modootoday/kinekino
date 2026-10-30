use super::*;

pub(super) struct ReplacementEngine {
    pub base: Arc<FakeEngine>,
    pub replacements: AtomicUsize,
    pub options: Mutex<Option<(bool, f64)>>,
}
impl ReplacementEngine {
    pub fn new(id: u64, router: &ParentEngineRouter) -> Arc<Self> {
        let base = FakeEngine::new(id);
        *base.router.lock().unwrap() = Arc::downgrade(&router.shared);
        Arc::new(Self {
            base,
            replacements: AtomicUsize::new(0),
            options: Mutex::new(None),
        })
    }
}
impl ParentOwnedEngine for ReplacementEngine {
    fn replacement_owner(&self) -> Option<&dyn EpisodeReplacementOwner> {
        Some(self)
    }
    fn shutdown_for_parent(&self) -> Result<Option<ClosedEpisodePlayback>, ApiError> {
        self.base.shutdown_for_parent()
    }
}
impl EpisodePlaybackEngine for ReplacementEngine {
    fn open(
        &self,
        source: EpisodePlaybackSource,
        paused: bool,
        volume: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        self.base.open(source, paused, volume)
    }
    fn close(&self, input: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        self.base.close(input)
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        self.base.observe()
    }
    fn control(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        self.base.control(input)
    }
}
impl EpisodeReplacementOwner for ReplacementEngine {
    fn replace_verified(
        &self,
        expected_instance: DecimalU64,
        expected_artifact: &str,
        source: EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<ReplacedEpisode, ApiError> {
        self.base.require(expected_instance.0)?;
        if expected_artifact != "a".repeat(64) {
            return Err(conflict("Synthetic artifact changed"));
        }
        self.base.call(0);
        self.replacements.fetch_add(1, Ordering::SeqCst);
        *self.options.lock().unwrap() = Some((paused, volume_percent));
        Ok(ReplacedEpisode {
            previous: closed(self.base.id, true),
            current: OpenedEpisodePlayback {
                engine_instance_id: DecimalU64(self.base.id + 1),
                resolution_artifact_id: source.artifact_id().into(),
                completion: ControlCompletion::Unobserved,
            },
        })
    }
}
