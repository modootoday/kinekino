use super::*;
use crate::domain::identifiers::{valid_id, valid_sha256};
use std::collections::BTreeSet;
fn pin(value: &ArtifactPin) -> Result<(), &'static str> {
    if !valid_id(&value.artifact_id) || value.revision.0 == 0 || !valid_sha256(&value.sha256) {
        return Err("Invalid clip artifact pin");
    }
    Ok(())
}
fn bound(value: &SourceBoundArtifact, source: &ClipSourcePin) -> Result<(), &'static str> {
    pin(&value.artifact)?;
    if value.source_sha256 != source.sha256 {
        return Err("Clip artifact belongs to another source");
    }
    Ok(())
}
impl ClipDocument {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "1"
            || !valid_id(&self.document_id)
            || self.revision.0 == 0
            || self.sources.is_empty()
            || self.sources.len() > MAX_SOURCES
            || self.segments.is_empty()
            || self.segments.len() > MAX_SEGMENTS
        {
            return Err("Invalid clip document identity or count");
        }
        pin(&self.preset.catalog)?;
        if self.preset.width == 0 || self.preset.height == 0 {
            return Err("Invalid clip output dimensions");
        }
        self.preset.frame_rate.validate()?;
        let mut sources = BTreeSet::new();
        for source in &self.sources {
            if !valid_id(&source.media_id)
                || !sources.insert(&source.media_id)
                || !valid_sha256(&source.sha256)
                || source.duration_us.0 == 0
                || source.width == 0
                || source.height == 0
                || !valid_sha256(&source.clock.observation_sha256)
                || source.tracks.is_empty()
                || source.tracks.len() > 64
            {
                return Err("Invalid clip source declaration");
            }
            source.clock.transform()?;
            let mut tracks = BTreeSet::new();
            for track in &source.tracks {
                if !tracks.insert(track.stream_index) || !valid_id(&track.codec_id) {
                    return Err("Invalid or duplicate container stream index");
                }
            }
        }
        let mut segments = BTreeSet::new();
        let mut duration = 0_u64;
        for segment in &self.segments {
            if !valid_id(&segment.segment_id) || !segments.insert(&segment.segment_id) {
                return Err("Invalid or duplicate clip segment");
            }
            let source = self
                .sources
                .iter()
                .find(|source| source.media_id == segment.media_id)
                .ok_or("Unknown clip segment source")?;
            let range = segment.range;
            if range.start_us.0 >= range.end_us.0 || range.end_us.0 > source.duration_us.0 {
                return Err("Clip range must be a nonempty source interval");
            }
            duration = duration
                .checked_add(range.end_us.0 - range.start_us.0)
                .ok_or("Clip output duration overflow")?;
            if !source.tracks.iter().any(|track| {
                track.stream_index == segment.video_stream_index
                    && track.kind == SourceTrackKind::Video
            }) {
                return Err("Selected video stream is not bound to this source");
            }
            if let AudioSelection::Selected {
                stream_index,
                volume_percent,
                ..
            } = segment.audio
                && (volume_percent > 100
                    || self.preset.audio_codec.is_none()
                    || !source.tracks.iter().any(|track| {
                        track.stream_index == stream_index && track.kind == SourceTrackKind::Audio
                    }))
            {
                return Err("Invalid source audio selection or output audio policy");
            }
            if segment.crop_keys.is_empty()
                || segment.crop_keys.len() > MAX_CROP_KEYS
                || segment.crop_keys[0].source_us != range.start_us
            {
                return Err("Crop keys must begin at the clip source interval");
            }
            let mut previous = None;
            for key in &segment.crop_keys {
                key.crop.validate()?;
                if key.source_us.0 < range.start_us.0
                    || key.source_us.0 >= range.end_us.0
                    || previous.is_some_and(|value| key.source_us.0 <= value)
                {
                    return Err("Crop keys must increase inside the half-open source interval");
                }
                previous = Some(key.source_us.0);
            }
            if let Some(subject) = &segment.subject_ref {
                bound(subject, source)?
            }
            if let Some(subtitles) = &segment.subtitles {
                bound(&subtitles.document, source)?;
                pin(&subtitles.style)?;
                if subtitles.cue_ids.is_empty() || subtitles.cue_ids.len() > MAX_SELECTED_CUES {
                    return Err("Invalid selected clip cue count");
                }
                let mut cues = BTreeSet::new();
                for id in &subtitles.cue_ids {
                    if !valid_id(id) || !cues.insert(id) {
                        return Err("Invalid or duplicate selected clip cue");
                    }
                }
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| "Invalid clip document serialization")?
            .len()
            > MAX_DOCUMENT_BYTES
        {
            return Err("Clip document byte limit exceeded");
        }
        Ok(())
    }
    pub fn duration_us(&self) -> Result<DecimalU64, &'static str> {
        self.validate()?;
        self.segments
            .iter()
            .try_fold(0_u64, |total, segment| {
                total
                    .checked_add(segment.range.end_us.0 - segment.range.start_us.0)
                    .ok_or("Clip output duration overflow")
            })
            .map(DecimalU64)
    }
}
