//! One SDK engine port; trusted assembly alone controls parent generations.
mod replacement;
mod route;
#[cfg(test)]
mod tests;
use super::{
    NativeEpisodeEngine, ParentBinding, ParentOwnedEngine, parent_binding::EngineAdmission,
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::ControlReceipt,
        playback_session::*,
    },
    infrastructure::thread_completion::{CompletionWake, DispatcherHandle},
};
pub use route::{
    ParentRouteHandle, RetirementHandle, RetirementObservation, RetirementStart,
    RoutePreparationFailure, RouterStatus,
};
use route::{Phase, Route, Shared, State};
#[cfg(any(test, feature = "native-proposal-window"))]
pub use route::{RetirementFailure, RetirementProgress};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
#[must_use = "retain the router until its routes and dispatcher are explicitly drained"]
pub struct ParentEngineRouter {
    shared: Arc<Shared>,
    #[cfg(feature = "native-proposal-window")]
    transition_slot: Option<Arc<crate::application::playback_session::transition::TransitionSlot>>,
    #[cfg(feature = "native-proposal-window")]
    transition_jobs: Option<Arc<dyn crate::application::jobs::JobsApi>>,
    #[cfg(all(test, feature = "native-proposal-window"))]
    observation_fault: Arc<std::sync::atomic::AtomicBool>,
}
impl ParentEngineRouter {
    /// A failed prepare minted no route. Retain cleanup of that exact engine on this dispatcher.
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn prepare_failed_engine_cleanup(
        &self,
        engine: &Arc<NativeEpisodeEngine>,
        wake: CompletionWake,
    ) -> Result<super::ParentCleanupTask, ApiError> {
        // Epoch is local to this private cleanup-only binding, never a router generation.
        let epoch = std::num::NonZeroU64::new(1).expect("nonzero cleanup-local epoch");
        let binding = ParentBinding::new_scoped(epoch, engine.guard_scope())?;
        binding
            .bind(epoch, engine.clone())
            .map_err(|failure| failure.error)?;
        binding.revoke_admission();
        Ok(super::ParentCleanupTask::new(
            binding,
            epoch,
            self.shared.dispatcher.clone(),
            wake,
        ))
    }
    /// Trusted ready assembly only: select and capture under the original Active lock.
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn capture_active_native_export_issuer(
        &self,
        original: &ParentRouteHandle,
    ) -> Result<v1::NativeExportIssuer, ApiError> {
        self.check(original)?;
        let state = self.shared.state.lock().map_err(|_| internal())?;
        match &state.phase {
            Phase::Active(route) if route.handle.same(original) => original
                .binding
                .admit(original.epoch)?
                .capture_export_issuer(Some(Arc::downgrade(&self.shared))),
            Phase::Empty | Phase::Prepared(_) => Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Original parent route is not active",
            )),
            _ => Err(conflict("Original active parent route changed or retired")),
        }
    }

    /// Capture only the caller's explicit original route; never select current.
    pub fn capture_native_export_issuer(
        &self,
        route: &ParentRouteHandle,
    ) -> Result<v1::NativeExportIssuer, ApiError> {
        self.check(route)?;
        route
            .binding
            .admit(route.epoch)?
            .capture_export_issuer(Some(Arc::downgrade(&self.shared)))
    }
    pub(crate) fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    sequence: 0,
                    phase: Phase::Empty,
                }),
                dispatcher,
            }),
            #[cfg(feature = "native-proposal-window")]
            transition_slot: None,
            #[cfg(feature = "native-proposal-window")]
            transition_jobs: None,
            #[cfg(all(test, feature = "native-proposal-window"))]
            observation_fault: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    /// Trusted composition fixes the same slot before publishing this router or its sessions.
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn new_with_transitions(
        dispatcher: DispatcherHandle,
        slot: Arc<crate::application::playback_session::transition::TransitionSlot>,
    ) -> Self {
        let mut router = Self::new(dispatcher);
        router.transition_slot = Some(slot);
        router
    }
    /// Parent-loss path: no state lock, slot drain, cancellation claim or new authority.
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn close_transition_admission(&self) {
        if let Some(slot) = &self.transition_slot {
            slot.close_admission();
        }
    }
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn with_transition_jobs(
        mut self,
        jobs: Arc<dyn crate::application::jobs::JobsApi>,
    ) -> Self {
        self.transition_jobs = Some(jobs);
        self
    }
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn pause_route_transitions(
        &self,
    ) -> Result<crate::application::playback_session::transition::RoutePause, ApiError> {
        self.transition_slot
            .as_ref()
            .ok_or_else(|| conflict("Original transition slot missing"))?
            .pause_route()
    }
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn poll_route_transitions(
        &self,
        ticket: &crate::application::playback_session::transition::RoutePause,
    ) -> Result<bool, ApiError> {
        self.transition_slot
            .as_ref()
            .ok_or_else(|| conflict("Original transition slot missing"))?
            .poll_route_pause(
                ticket,
                self.transition_jobs
                    .as_deref()
                    .ok_or_else(|| conflict("Original transition jobs missing"))?,
            )
    }
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn resume_route_transitions(
        &self,
        ticket: &crate::application::playback_session::transition::RoutePause,
        fresh: &ParentRouteHandle,
    ) -> Result<(), ApiError> {
        self.check(fresh)?;
        self.transition_slot
            .as_ref()
            .ok_or_else(|| conflict("Original transition slot missing"))?
            .resume_route(ticket, || {
                let state = self.shared.state.lock().map_err(|_| internal())?;
                match &state.phase {
                    Phase::Active(route) if route.handle.same(fresh) => {
                        fresh.binding.admit(fresh.epoch)?;
                        Ok(())
                    }
                    _ => Err(conflict("Fresh Viewing route is not active")),
                }
            })
    }
    /// Owner identity only; no route/task lock or current-generation selection.
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn validate_route_identity(
        &self,
        route: &ParentRouteHandle,
    ) -> Result<(), ApiError> {
        self.check(route)
    }
    /// Revoke only the immutable original binding; safe on the UI parent-loss path.
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn revoke_route_admission(&self, route: &ParentRouteHandle) -> Result<(), ApiError> {
        self.check(route)?;
        route.binding.revoke_admission();
        Ok(())
    }
    /// Recover the same owner after a failed observation/start, without submitting work.
    #[cfg(feature = "native-proposal-window")]
    pub(crate) fn retirement_handle(
        &self,
        route: &ParentRouteHandle,
    ) -> Result<Option<RetirementHandle>, ApiError> {
        self.check(route)?;
        let state = self.shared.state.lock().map_err(|_| internal())?;
        match &state.phase {
            Phase::Retiring(owner) if owner.route.same(route) => Ok(Some(RetirementHandle {
                owner: owner.clone(),
            })),
            Phase::Prepared(current) | Phase::Active(current) if current.handle.same(route) => {
                Ok(None)
            }
            _ => Err(conflict("Retirement route is stale")),
        }
    }
    #[cfg(all(test, feature = "native-proposal-window"))]
    pub(crate) fn prepare_test_engine(
        &self,
        engine: Arc<dyn ParentOwnedEngine>,
    ) -> Result<ParentRouteHandle, ApiError> {
        self.prepare_engine(&engine)
    }
    #[cfg(all(test, feature = "native-proposal-window"))]
    pub(crate) fn fail_observation_for_test(&self, fail: bool) {
        self.observation_fault
            .store(fail, std::sync::atomic::Ordering::Release);
    }
    #[cfg(all(test, feature = "native-proposal-window"))]
    pub(crate) fn same_retirement_for_test(a: &RetirementHandle, b: &RetirementHandle) -> bool {
        Arc::ptr_eq(&a.owner, &b.owner)
    }
    #[cfg(all(test, feature = "native-proposal-window"))]
    pub(crate) fn with_route_lock_for_test(&self, body: impl FnOnce()) {
        let _guard = self.shared.state.lock().unwrap();
        body();
    }
    /// The native backend supplies process-wide checked instance IDs. No remote token input.
    pub(crate) fn prepare(
        &self,
        engine: Arc<NativeEpisodeEngine>,
    ) -> Result<ParentRouteHandle, Box<RoutePreparationFailure>> {
        let scope = engine.guard_scope();
        let owner: Arc<dyn ParentOwnedEngine> = engine.clone();
        self.prepare_engine_scoped(&owner, Some(scope))
            .map_err(|error| Box::new(RoutePreparationFailure { error, engine }))
    }
    #[cfg(test)]
    fn prepare_engine(
        &self,
        engine: &Arc<dyn ParentOwnedEngine>,
    ) -> Result<ParentRouteHandle, ApiError> {
        self.prepare_engine_scoped(engine, None)
    }
    fn prepare_engine_scoped(
        &self,
        engine: &Arc<dyn ParentOwnedEngine>,
        scope: Option<std::sync::Weak<super::guarded_command::HostScope>>,
    ) -> Result<ParentRouteHandle, ApiError> {
        let mut state = self.shared.state.lock().map_err(|_| internal())?;
        match &state.phase {
            Phase::Empty => {}
            Phase::Retiring(owner) if owner.complete() => {}
            Phase::Retiring(_) => {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Original parent cleanup has not joined successfully",
                ));
            }
            _ => return Err(conflict("A parent route is already prepared or active")),
        }
        let sequence = state
            .sequence
            .checked_add(1)
            .and_then(std::num::NonZeroU64::new)
            .ok_or_else(|| {
                ApiError::new(ErrorCode::ResourceExhausted, "Parent generation exhausted")
            })?;
        let binding = match scope {
            Some(scope) => ParentBinding::new_scoped(sequence, scope)?,
            None => ParentBinding::new(sequence),
        };
        binding
            .bind(sequence, engine.clone())
            .map_err(|failure| failure.error)?;
        let handle = ParentRouteHandle::new(&self.shared, sequence, binding);
        state.sequence = sequence.get();
        state.phase = Phase::Prepared(Route {
            handle: handle.clone(),
        });
        Ok(handle)
    }
    pub(crate) fn activate(&self, prepared: &ParentRouteHandle) -> Result<(), ApiError> {
        self.check(prepared)?;
        let mut state = self.shared.state.lock().map_err(|_| internal())?;
        match &state.phase {
            Phase::Prepared(route) if route.handle.same(prepared) => {
                // Validate that atomic revocation did not win before this state lock.
                drop(prepared.binding.admit(prepared.epoch)?);
                state.phase = Phase::Active(Route {
                    handle: prepared.clone(),
                });
                Ok(())
            }
            _ => Err(conflict("Prepared parent route is stale or already active")),
        }
    }
    pub(crate) fn retire(
        &self,
        route: &ParentRouteHandle,
        wake: CompletionWake,
    ) -> Result<RetirementStart, ApiError> {
        self.check(route)?;
        // The handle fixes the original binding. Never select a new current parent to revoke.
        route.binding.revoke_admission();
        let owner = {
            let mut state = self.shared.state.lock().map_err(|_| internal())?;
            match &state.phase {
                Phase::Prepared(current) | Phase::Active(current) if current.handle.same(route) => {
                    let owner = Arc::new(route::RetirementOwner::new(
                        route.clone(),
                        self.shared.dispatcher.clone(),
                    ));
                    state.phase = Phase::Retiring(owner.clone());
                    owner
                }
                Phase::Retiring(owner) if owner.route.same(route) => owner.clone(),
                _ => return Err(conflict("Retirement route is stale")),
            }
        };
        let dispatch_error = owner.start_once(wake)?;
        Ok(RetirementStart {
            handle: RetirementHandle { owner },
            dispatch_error,
        })
    }
    pub(crate) fn observe_retirement(
        &self,
        handle: &RetirementHandle,
    ) -> Result<RetirementObservation, ApiError> {
        self.check(&handle.owner.route)?;
        #[cfg(all(test, feature = "native-proposal-window"))]
        if self
            .observation_fault
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return Err(internal());
        }
        handle.owner.observe()
    }
    /// A fresh attempt receives its own notification; earlier callbacks retain their own wake.
    pub(crate) fn retry_retirement(
        &self,
        handle: &RetirementHandle,
        wake: CompletionWake,
    ) -> Result<(), ApiError> {
        self.check(&handle.owner.route)?;
        handle.owner.retry(wake)
    }
    pub(crate) fn status(&self) -> Result<RouterStatus, ApiError> {
        let state = self.shared.state.lock().map_err(|_| internal())?;
        Ok(match &state.phase {
            Phase::Empty => RouterStatus::EmptyNeverReady,
            Phase::Prepared(route) => RouterStatus::Prepared {
                generation: route.handle.generation(),
            },
            Phase::Active(route) => RouterStatus::Active {
                generation: route.handle.generation(),
            },
            Phase::Retiring(owner) if owner.complete() => RouterStatus::JoinedCompleteNoRoute {
                generation: owner.route.generation(),
            },
            Phase::Retiring(owner) => RouterStatus::Retiring {
                generation: owner.route.generation(),
            },
        })
    }
    fn check(&self, handle: &ParentRouteHandle) -> Result<(), ApiError> {
        if !handle.belongs(&self.shared) {
            return Err(conflict("Parent route belongs to another router"));
        }
        Ok(())
    }
    /// Recovery of a dropped lifecycle handle retains the same opaque binding identity.
    pub(crate) fn current_route(&self) -> Result<Option<ParentRouteHandle>, ApiError> {
        let state = self.shared.state.lock().map_err(|_| internal())?;
        Ok(match &state.phase {
            Phase::Empty => None,
            Phase::Prepared(route) | Phase::Active(route) => Some(route.handle.clone()),
            Phase::Retiring(owner) => Some(owner.route.clone()),
        })
    }
    fn admitted(&self) -> Result<EngineAdmission, ApiError> {
        let state = self.shared.state.lock().map_err(|_| internal())?;
        match &state.phase {
            Phase::Active(route) => route.handle.binding.admit(route.handle.epoch),
            Phase::Empty | Phase::Prepared(_) => Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Parent route is not active",
            )),
            Phase::Retiring(_) => Err(ApiError::new(
                ErrorCode::Cancelled,
                "Parent route was retired",
            )),
        }
    }
}
fn internal() -> ApiError {
    ApiError::new(ErrorCode::Internal, "Parent route ownership lock failed")
}
fn conflict(message: &str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}
impl EpisodePlaybackEngine for ParentEngineRouter {
    fn observe_native_guard_for_export(
        &self,
        issuer: &v1::NativeExportIssuer,
    ) -> Result<NativeGuardObservation, ApiError> {
        let owner = self.admitted()?;
        owner.require_export_issuer(issuer, Some(Arc::downgrade(&self.shared)))?;
        owner.engine.observe_native_guard_for_export(issuer)
    }
    fn submit_exported_guarded_seek_v1(
        &self,
        mut input: v1::ExportGuardedSeekV1,
    ) -> Result<v1::ExportedGuardedSeekV1, ApiError> {
        let owner = self.admitted()?;
        input.seek.guard = input.seek.guard.prepare_routed_export(
            owner.prepare_export_read_owner(Some(Arc::downgrade(&self.shared)))?,
        )?;
        owner.engine.submit_exported_guarded_seek_v1(input)
    }
    fn submit_exported_guarded_seek(
        &self,
        mut input: ExportGuardedSeek,
    ) -> Result<ExportedGuardedSeekSubmission, ApiError> {
        let owner = self.admitted()?;
        input.seek.guard = input.seek.guard.prepare_routed_export(
            owner.prepare_export_read_owner(Some(Arc::downgrade(&self.shared)))?,
        )?;
        owner.engine.submit_exported_guarded_seek(input)
    }
    fn exported_seek_result(
        &self,
        input: NativeExportResultQuery,
    ) -> Result<GuardedSeekResult, ApiError> {
        ParentBinding::read_routed_export(&Arc::downgrade(&self.shared), input)
    }
    fn observe_native_guard(&self) -> Result<NativeGuardObservation, ApiError> {
        self.admitted()?.engine.observe_native_guard()
    }
    fn submit_guarded_seek(&self, input: GuardedSeek) -> Result<GuardedSeekSubmission, ApiError> {
        let owner = self.admitted()?;
        let submission = owner.engine.submit_guarded_seek(input)?;
        owner.bind_result(submission, Some(Arc::downgrade(&self.shared)))
    }
    fn guarded_seek_result(
        &self,
        input: GuardedSeekResultQuery,
    ) -> Result<GuardedSeekResult, ApiError> {
        ParentBinding::read_routed_result(&Arc::downgrade(&self.shared), input)
    }
    fn options_available(&self) -> bool {
        self.admitted()
            .is_ok_and(|owner| owner.engine.options_available())
    }
    fn capture_frame_available(&self) -> bool {
        self.admitted()
            .is_ok_and(|owner| owner.engine.capture_frame_available())
    }
    fn open(
        &self,
        source: EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        let owner = self.admitted()?;
        owner.engine.open(source, paused, volume_percent)
    }
    fn close(&self, input: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        let owner = self.admitted()?;
        owner.engine.close(input)
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        let owner = self.admitted()?;
        owner.engine.observe()
    }
    fn control(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        let owner = self.admitted()?;
        owner.engine.control(input)
    }
    fn set_rate(&self, input: SetEpisodeRate) -> Result<ControlReceipt, ApiError> {
        let owner = self.admitted()?;
        owner.engine.set_rate(input)
    }
    fn select_track(&self, input: SelectEpisodeTrack) -> Result<TrackControlReceipt, ApiError> {
        let owner = self.admitted()?;
        owner.engine.select_track(input)
    }
    fn capture_frame(&self, input: CaptureEpisodeFrame) -> Result<CapturedEpisodeFrame, ApiError> {
        let owner = self.admitted()?;
        owner.engine.capture_frame(input)
    }
}
