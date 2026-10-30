//! One SDK engine port, late-bound once by trusted assembly. No GUI or OS discovery.
use super::{
    NativeEpisodeEngine,
    parent_binding::{ParentBinding, ParentOwnedEngine},
};
use crate::application::{
    api::ApiError,
    playback_control::ControlReceipt,
    playback_session::{
        CaptureEpisodeFrame, CapturedEpisodeFrame, CloseEpisodePlayback, ClosedEpisodePlayback,
        ControlEpisodePlayback, EpisodePlaybackEngine, EpisodePlaybackObservation,
        EpisodePlaybackSource, OpenedEpisodePlayback, SelectEpisodeTrack, SetEpisodeRate,
        TrackControlReceipt,
    },
};
use std::num::NonZeroU64;

impl ParentOwnedEngine for NativeEpisodeEngine {
    fn replacement_owner(
        &self,
    ) -> Option<&dyn crate::application::playback_session::transition::EpisodeReplacementOwner>
    {
        Some(self)
    }
    fn shutdown_for_parent(&self) -> Result<Option<ClosedEpisodePlayback>, ApiError> {
        self.shutdown()
    }
}

#[derive(Clone)]
pub struct DeferredEpisodeEngine {
    binding: ParentBinding,
    epoch: NonZeroU64,
}
impl DeferredEpisodeEngine {
    pub fn new(binding: ParentBinding, epoch: NonZeroU64) -> Self {
        Self { binding, epoch }
    }
}
impl EpisodePlaybackEngine for DeferredEpisodeEngine {
    fn native_export_issuer(
        &self,
    ) -> Result<crate::application::playback_session::v1::NativeExportIssuer, ApiError> {
        self.binding
            .admit(self.epoch)?
            .capture_export_issuer::<()>(None)
    }
    fn observe_native_guard_for_export(
        &self,
        issuer: &crate::application::playback_session::v1::NativeExportIssuer,
    ) -> Result<crate::application::playback_session::NativeGuardObservation, ApiError> {
        let owner = self.binding.admit(self.epoch)?;
        owner.require_export_issuer::<()>(issuer, None)?;
        owner.engine.observe_native_guard_for_export(issuer)
    }
    fn submit_exported_guarded_seek_v1(
        &self,
        mut input: crate::application::playback_session::v1::ExportGuardedSeekV1,
    ) -> Result<crate::application::playback_session::v1::ExportedGuardedSeekV1, ApiError> {
        let owner = self.binding.admit(self.epoch)?;
        input.seek.guard = input
            .seek
            .guard
            .prepare_routed_export(owner.prepare_export_read_owner::<()>(None)?)?;
        owner.engine.submit_exported_guarded_seek_v1(input)
    }
    fn submit_exported_guarded_seek(
        &self,
        mut input: crate::application::playback_session::ExportGuardedSeek,
    ) -> Result<crate::application::playback_session::ExportedGuardedSeekSubmission, ApiError> {
        let owner = self.binding.admit(self.epoch)?;
        input.seek.guard = input
            .seek
            .guard
            .prepare_routed_export(owner.prepare_export_read_owner::<()>(None)?)?;
        owner.engine.submit_exported_guarded_seek(input)
    }
    fn exported_seek_references(
        &self,
        provenance: crate::application::playback_session::NativeResultProvenance,
    ) -> Result<Vec<crate::application::playback_session::NativeResultReference>, ApiError> {
        self.binding.list_bound_exports(self.epoch, provenance)
    }
    fn exported_seek_result(
        &self,
        input: crate::application::playback_session::NativeExportResultQuery,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        self.binding.read_bound_export(self.epoch, input)
    }
    fn observe_native_guard(
        &self,
    ) -> Result<crate::application::playback_session::NativeGuardObservation, ApiError> {
        self.binding
            .admit(self.epoch)?
            .engine
            .observe_native_guard()
    }
    fn submit_guarded_seek(
        &self,
        input: crate::application::playback_session::GuardedSeek,
    ) -> Result<crate::application::playback_session::GuardedSeekSubmission, ApiError> {
        let owner = self.binding.admit(self.epoch)?;
        let submission = owner.engine.submit_guarded_seek(input)?;
        owner.bind_result::<()>(submission, None)
    }
    fn guarded_seek_result(
        &self,
        input: crate::application::playback_session::GuardedSeekResultQuery,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        self.binding.read_bound_result(self.epoch, input)
    }
    fn options_available(&self) -> bool {
        self.binding
            .admit(self.epoch)
            .is_ok_and(|owner| owner.engine.options_available())
    }
    fn capture_frame_available(&self) -> bool {
        self.binding
            .admit(self.epoch)
            .is_ok_and(|owner| owner.engine.capture_frame_available())
    }
    fn open(
        &self,
        source: EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        self.binding
            .admit(self.epoch)?
            .engine
            .open(source, paused, volume_percent)
    }
    fn close(&self, input: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        self.binding.admit(self.epoch)?.engine.close(input)
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        self.binding.admit(self.epoch)?.engine.observe()
    }
    fn control(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        self.binding.admit(self.epoch)?.engine.control(input)
    }
    fn set_rate(&self, input: SetEpisodeRate) -> Result<ControlReceipt, ApiError> {
        self.binding.admit(self.epoch)?.engine.set_rate(input)
    }
    fn select_track(&self, input: SelectEpisodeTrack) -> Result<TrackControlReceipt, ApiError> {
        self.binding.admit(self.epoch)?.engine.select_track(input)
    }
    fn capture_frame(&self, input: CaptureEpisodeFrame) -> Result<CapturedEpisodeFrame, ApiError> {
        self.binding.admit(self.epoch)?.engine.capture_frame(input)
    }
}
