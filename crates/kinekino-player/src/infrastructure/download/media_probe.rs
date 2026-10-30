//! Owned probe source preparation. No installed-tool/process support claim yet.
#[path = "media_probe/attempt.rs"]
mod attempt;
#[path = "media_probe/result.rs"]
mod result;
use crate::application::{
    api::{ApiError, ErrorCode},
    download::MediaIdentity,
};
use std::{collections::BTreeSet, path::PathBuf};

pub(crate) struct ProbeToolPin {
    pub executable: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}
#[derive(Clone, Copy)]
pub(crate) enum ProbeContainer {
    Mp4,
    Matroska,
    MpegTs,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrackKind {
    Video,
    Audio,
    Subtitle,
}
pub(crate) struct ExpectedTrack {
    pub index: u32,
    pub kind: TrackKind,
    pub codec: Option<String>,
}
pub(crate) struct ProbePlan {
    pub identity: MediaIdentity,
    pub container: ProbeContainer,
    pub tracks: Vec<ExpectedTrack>,
    pub minimum_duration_us: u64,
    pub maximum_duration_us: u64,
}
impl ProbePlan {
    pub(crate) fn validate(&self) -> Result<(), ApiError> {
        let mut indices = BTreeSet::new();
        if self.tracks.is_empty()
            || self.tracks.len() > 64
            || self.minimum_duration_us == 0
            || self.maximum_duration_us < self.minimum_duration_us
            || self.tracks.iter().any(|track| {
                !indices.insert(track.index)
                    || track.codec.as_ref().is_some_and(|codec| {
                        codec.is_empty()
                            || codec.len() > 64
                            || !codec
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                    })
            })
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid selected media probe plan",
            ));
        }
        Ok(())
    }
}
pub(crate) struct InspectedMedia {
    pub duration_us: u64,
    pub streams: usize,
}

pub(crate) struct MediaProbe {
    pub tool: std::sync::Arc<ProbeToolPin>,
    pub plan: std::sync::Arc<ProbePlan>,
    pub evidence_root: PathBuf,
    pub cleanup_reserve: std::time::Duration,
    pub sequence: std::sync::atomic::AtomicU64,
}
impl crate::application::download::DownloadVerifier for MediaProbe {
    fn verify(
        &self,
        _path: &std::path::Path,
        _expected: &MediaIdentity,
        _control: &dyn crate::application::jobs::ExecutionControl,
    ) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "External probe requires the core-owned verification lease",
        ))
    }
    fn begin_owned(
        &self,
        lease: std::sync::Arc<dyn crate::application::download::VerificationLease>,
        expected: MediaIdentity,
        control: &dyn crate::application::jobs::ExecutionControl,
    ) -> Result<crate::application::download::OwnedVerification, ApiError> {
        self.plan.validate()?;
        if expected != self.plan.identity || control.execution_identity().is_none() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Probe requires the original selected media and job binding",
            ));
        }
        let sequence = self
            .sequence
            .try_update(
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
                |value| value.checked_add(1),
            )
            .map_err(|_| ApiError::new(ErrorCode::ResourceExhausted, "Probe sequence exhausted"))?;
        let binding = control.execution_identity().unwrap();
        let name = format!(
            "probe-{}-{}-{sequence}",
            binding.job_id().as_str(),
            binding.attempt()
        );
        let attempt = attempt::ProbeAttempt::prepare(
            self.tool.clone(),
            self.plan.clone(),
            lease,
            &self.evidence_root,
            &name,
            self.cleanup_reserve,
            control,
        )?;
        Ok(crate::application::download::OwnedVerification::Pending(
            Box::new(attempt),
        ))
    }
}
