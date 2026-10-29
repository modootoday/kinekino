use super::{ArtifactPin, invalid};
use kinekino_media_contracts::{api_values::DecimalU64, errors::ApiError};

pub const MAX_AUDIO_BYTES: u64 = 1024 * 1024;
pub const MAX_SAMPLES: u64 = 480_000;
pub const CHUNK_SAMPLES: u64 = 512;

#[derive(Debug, Clone)]
pub struct VadInput {
    wav: ArtifactPin,
    pcm: ArtifactPin,
    samples: u64,
    start: u64,
    end: u64,
}
impl VadInput {
    pub fn new(
        wav: ArtifactPin,
        pcm: ArtifactPin,
        samples: DecimalU64,
        start: DecimalU64,
        end: DecimalU64,
    ) -> Result<Self, ApiError> {
        if wav.bytes() > MAX_AUDIO_BYTES
            || !(1..=MAX_SAMPLES).contains(&samples.0)
            || samples.0.checked_mul(2) != Some(pcm.bytes())
            || start.0 >= end.0
            || end.0 > samples.0
        {
            return Err(invalid("Invalid bounded mono16k PCM16 input/range"));
        }
        Ok(Self {
            wav,
            pcm,
            samples: samples.0,
            start: start.0,
            end: end.0,
        })
    }
    pub fn wav(&self) -> &ArtifactPin {
        &self.wav
    }
    pub fn pcm(&self) -> &ArtifactPin {
        &self.pcm
    }
    pub fn samples(&self) -> u64 {
        self.samples
    }
    pub fn start(&self) -> u64 {
        self.start
    }
    pub fn end(&self) -> u64 {
        self.end
    }
    pub fn selected_samples(&self) -> u64 {
        self.end - self.start
    }
    pub fn expected_frames(&self) -> u64 {
        self.selected_samples().div_ceil(CHUNK_SAMPLES)
    }
}
