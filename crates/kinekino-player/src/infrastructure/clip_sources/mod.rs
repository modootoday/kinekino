//! Fixed local catalog, original file/tool guards and bounded owned FFprobe execution.
mod boundaries;
mod observation;
mod probe;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_render::{ClipSourceAdmission, ClipSourceLease},
        review_source::MutationControl,
    },
    domain::{clip_document::ClipSourcePin, frame_capture::MAX_MEDIA_BYTES, identifiers::valid_id},
    infrastructure::{
        content_digest,
        frame_capture::{process, tools},
        immutable_storage_paths as paths,
    },
};
pub(crate) use boundaries::RetainedClipBoundary;
pub use boundaries::{ClipBoundaryObservation, NativeClipBoundaryLease};
pub use observation::{DurationBasis, ProbeFacts};
pub(crate) use probe::PreparedClipSourceProbe;
impl crate::application::clip_render::ClipSourceInspector for FfmpegClipSources {
    fn inspect(
        &self,
        media_id: &str,
        control: &dyn MutationControl,
    ) -> Result<ProbeFacts, ApiError> {
        let retained = self.inspect_source(media_id, control)?;
        Ok(retained.facts().clone())
    }
}
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub enum ClipContainer {
    Mp4,
    Matroska,
}
impl ClipContainer {
    fn demuxer(self) -> &'static str {
        match self {
            Self::Mp4 => "mov",
            Self::Matroska => "matroska",
        }
    }
    fn matches(self, header: &[u8; 12]) -> bool {
        match self {
            Self::Mp4 => &header[4..8] == b"ftyp",
            Self::Matroska => header[..4] == [0x1a, 0x45, 0xdf, 0xa3],
        }
    }
}
/// Assembly-owned selection. No Deserialize and no caller-supplied paths in admission.
pub struct FixedClipSource {
    pub path: PathBuf,
    pub container: ClipContainer,
}
struct ProbeTool {
    path: PathBuf,
    sha256: String,
    _guards: Vec<File>,
}
pub struct FfmpegClipSources {
    catalog: BTreeMap<String, FixedClipSource>,
    tool: Arc<ProbeTool>,
    max_bytes: u64,
    timeout: Duration,
}
pub struct NativeClipSourceLease {
    custody: Arc<()>,
    container: ClipContainer,
    facts: ProbeFacts,
    path: PathBuf,
    raw_probe: Vec<u8>,
    _guards: Vec<File>,
    _tool: Arc<ProbeTool>,
}
impl ClipSourceLease for NativeClipSourceLease {
    fn observation(&self) -> &ClipSourcePin {
        &self.facts.source
    }
}
impl NativeClipSourceLease {
    pub(crate) fn demuxer(&self) -> &'static str {
        self.container.demuxer()
    }
    pub fn facts(&self) -> &ProbeFacts {
        &self.facts
    }
    pub fn raw_probe(&self) -> &[u8] {
        &self.raw_probe
    }
    /// Only a trusted rendering adapter receives this retained local input.
    pub fn path(&self) -> &Path {
        &self.path
    }
}
struct ProbeControl<'a> {
    outer: &'a dyn MutationControl,
    deadline: Instant,
}
impl MutationControl for ProbeControl<'_> {
    fn native_processes(&self) -> Option<&crate::application::review_source::NativeProcessLedger> {
        self.outer.native_processes()
    }
    fn checkpoint(&self) -> Result<(), ApiError> {
        self.outer.checkpoint()?;
        if Instant::now() >= self.deadline {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Clip source hash/probe deadline elapsed",
            ))
        } else {
            Ok(())
        }
    }
}
impl FfmpegClipSources {
    pub fn new(
        catalog: BTreeMap<String, FixedClipSource>,
        probe: &Path,
        probe_sha256: &str,
        max_bytes: u64,
        timeout: Duration,
    ) -> Result<Self, ApiError> {
        if catalog.is_empty()
            || catalog.len() > 128
            || catalog.keys().any(|key| !valid_id(key))
            || max_bytes == 0
            || max_bytes > MAX_MEDIA_BYTES
            || timeout.is_zero()
            || timeout > Duration::from_secs(120)
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid bounded clip source catalog or budget",
            ));
        }
        let (path, guards) = tools::open(probe, probe_sha256)?;
        Ok(Self {
            catalog,
            tool: Arc::new(ProbeTool {
                path,
                sha256: probe_sha256.into(),
                _guards: guards,
            }),
            max_bytes,
            timeout,
        })
    }
    /// Generates candidate project source values; not encoder, frame or quality approval.
    pub fn inspect_source(
        &self,
        media_id: &str,
        control: &dyn MutationControl,
    ) -> Result<NativeClipSourceLease, ApiError> {
        let mut prepared = self.prepare_source_probe(media_id, control)?;
        if let Err(error) = prepared.run(control) {
            while !prepared.poll_close(Instant::now(), None).may_release() {
                std::thread::sleep(Duration::from_millis(10));
            }
            return Err(error);
        }
        prepared.take_source(control)
    }
    /// Original media/tool/operation owner is returned before any probe launch.
    pub(crate) fn prepare_source_probe(
        &self,
        media_id: &str,
        control: &dyn MutationControl,
    ) -> Result<PreparedClipSourceProbe, ApiError> {
        let deadline = Instant::now() + self.timeout;
        control.checkpoint()?;
        let selection = self.catalog.get(media_id).ok_or_else(|| {
            ApiError::new(
                ErrorCode::NotFound,
                "Clip source is absent from the fixed host catalog",
            )
        })?;
        PreparedClipSourceProbe::prepare(
            media_id,
            selection,
            self.tool.clone(),
            self.max_bytes,
            deadline,
            control,
        )
    }
}
impl ClipSourceAdmission for FfmpegClipSources {
    type Lease = NativeClipSourceLease;
    fn open_source(
        &self,
        expected: &ClipSourcePin,
        control: &dyn MutationControl,
    ) -> Result<Self::Lease, ApiError> {
        self.inspect_source(&expected.media_id, control)
    }
}
#[cfg(test)]
mod tests;
