//! Original render resources travel with the verified candidate through publication.
use super::*;
use crate::{
    application::clip_render::subtitles::PreparedClipSubtitles,
    infrastructure::{
        clip_sources::RetainedClipBoundary,
        clip_subtitle_assets::{NativeSubtitleDocument, NativeSubtitleStyle},
    },
};
type Subtitles = PreparedClipSubtitles<NativeSubtitleDocument, NativeSubtitleStyle>;
struct ClipRenderContext {
    _media: PreparedClipMedia<NativeClipSourceLease>,
    _encoder: PreparedClipEncoder<PinnedClipEncoder>,
    _decoded: RetainedClipBoundary,
    _subtitles: Option<Subtitles>,
    _overlay: Option<subtitles::RetainedClipOverlay>,
}
/// Opaque ownership, not a manifest/worker declaration and not reconstructible from JSON.
pub(crate) struct RetainedClipRenderCandidate {
    verified: OwnedVerifiedClipCandidate,
    _context: ClipRenderContext,
}
#[must_use = "Keep original candidate context until all original working inputs are released"]
pub(crate) struct PendingClipRenderRelease {
    _candidate: RetainedClipRenderCandidate,
    working: Vec<crate::infrastructure::clip_working_file::ClipWorkingFileRelease>,
}
impl PendingClipRenderRelease {
    pub(crate) fn poll(&mut self) -> Result<bool, ApiError> {
        let mut complete = true;
        for working in &mut self.working {
            if !working.poll()? {
                complete = false;
            }
        }
        Ok(complete)
    }
}
fn mismatch() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Retained clip context differs from its original verified candidate",
    )
}
impl RetainedClipRenderCandidate {
    pub(crate) fn new(
        verified: OwnedVerifiedClipCandidate,
        media: PreparedClipMedia<NativeClipSourceLease>,
        encoder: PreparedClipEncoder<PinnedClipEncoder>,
        decoded: RetainedClipBoundary,
        subtitles: Option<Subtitles>,
        overlay: Option<subtitles::RetainedClipOverlay>,
    ) -> Result<Self, ApiError> {
        if !verified.has_working() {
            return Err(mismatch());
        }
        let facts = verified.candidate().facts();
        let source = media
            .sources()
            .iter()
            .find(|source| {
                decoded.belongs_to(source)
                    && source.facts().source.media_id == facts.plan.source_media_id
                    && source.facts().source.sha256 == facts.plan.source_sha256
            })
            .ok_or_else(mismatch)?;
        if media.project().document_id != facts.project.document_id
            || media.project().publication_revision != facts.project.publication_revision
            || media.project().publication_sha256 != facts.project.publication_sha256
            || encoder.document_sha256() != facts.plan.document_sha256
            || encoder.lease().inventory().tool_sha256 != facts.encoder_tool_sha256
            || decoded.observation().raw_frame_window_sha256 != facts.boundary_window_sha256
            || decoded.observation().source_sha256 != facts.plan.source_sha256
            || decoded.observation().source_observation_sha256
                != facts.plan.source_observation_sha256
            || serde_json::to_value(encoder.preset_evidence()).map_err(|_| mismatch())?
                != serde_json::to_value(&facts.preset_definition).map_err(|_| mismatch())?
        {
            return Err(mismatch());
        }
        match (&subtitles, &overlay, &facts.subtitles) {
            (None, None, None) => {}
            (Some(admitted), Some(overlay), Some(recorded)) => {
                let segment = media
                    .publication()
                    .document
                    .segments
                    .first()
                    .ok_or_else(mismatch)?;
                admitted.require_binding(&source.facts().source, segment)?;
                if overlay.facts().decoded_window_sha256 != facts.boundary_window_sha256
                    || serde_json::to_value(overlay.facts()).map_err(|_| mismatch())?
                        != serde_json::to_value(recorded).map_err(|_| mismatch())?
                {
                    return Err(mismatch());
                }
            }
            _ => return Err(mismatch()),
        }
        Ok(Self {
            verified,
            _context: ClipRenderContext {
                _media: media,
                _encoder: encoder,
                _decoded: decoded,
                _subtitles: subtitles,
                _overlay: overlay,
            },
        })
    }
    pub(crate) fn as_verified(&self) -> VerifiedClipCandidate<'_> {
        self.verified.as_verified()
    }
    pub(crate) fn into_release(mut self) -> PendingClipRenderRelease {
        let mut working = vec![
            self.verified
                .take_working_release()
                .expect("Original verified working input"),
        ];
        if let Some(overlay) = &mut self._context._overlay
            && let Some(release) = overlay.take_working_release()
        {
            working.push(release);
        }
        PendingClipRenderRelease {
            _candidate: self,
            working,
        }
    }
    /// Legacy worker bridge; pending/error keeps the same candidate and release carriers.
    pub(crate) fn finish_release(self) -> Result<(), ApiError> {
        let mut pending = self.into_release();
        let mut first = None;
        loop {
            match pending.poll() {
                Ok(true) => return first.map_or(Ok(()), Err),
                Ok(false) => {}
                Err(error) => {
                    if first.is_none() {
                        first = Some(error);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_render_candidate_can_be_carried_by_original_worker() {
        fn require_send<T: Send>() {}
        require_send::<RetainedClipRenderCandidate>();
    }
}
