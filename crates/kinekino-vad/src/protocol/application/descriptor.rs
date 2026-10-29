use super::{MAX_DESCRIPTOR_BYTES, payload};
use crate::domain::{ArtifactPin, VadInput, VadProfile};
use crate::protocol::domain::DecodedDescriptor;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PinDto {
    id: String,
    sha256: String,
    bytes: DecimalU64,
}
impl PinDto {
    fn checked(self) -> Result<ArtifactPin, ApiError> {
        ArtifactPin::new(self.id, self.sha256, self.bytes)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileDto {
    id: String,
    repo: String,
    revision: String,
    precision: String,
    provider: String,
    intra_threads: u32,
    inter_threads: u32,
    license_id: String,
    weights: PinDto,
    license_document: PinDto,
    runtime: PinDto,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputDto {
    wav: PinDto,
    pcm: PinDto,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    samples: DecimalU64,
    start_sample: DecimalU64,
    end_sample: DecimalU64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DescriptorDto {
    schema_version: String,
    profile: ProfileDto,
    input: InputDto,
}
impl DescriptorDto {
    pub(super) fn checked(self) -> Result<(VadProfile, VadInput), ApiError> {
        if self.schema_version != "1"
            || self.input.sample_rate != 16000
            || self.input.channels != 1
            || self.input.bits_per_sample != 16
        {
            return Err(payload::invalid(
                "Unsupported descriptor schema/audio format",
            ));
        }
        let p = self.profile;
        let profile = VadProfile::new(
            p.id,
            p.repo,
            p.revision,
            p.weights.checked()?,
            p.license_document.checked()?,
            p.runtime.checked()?,
        )?;
        if p.precision != profile.precision()
            || p.provider != profile.provider()
            || p.intra_threads != profile.intra_threads()
            || p.inter_threads != profile.inter_threads()
            || p.license_id != profile.license_id()
        {
            return Err(payload::invalid(
                "Profile wire differs from P0 supported contract",
            ));
        }
        let i = self.input;
        let input = VadInput::new(
            i.wav.checked()?,
            i.pcm.checked()?,
            i.samples,
            i.start_sample,
            i.end_sample,
        )?;
        Ok((profile, input))
    }
}
pub fn decode_descriptor(
    reader: impl Read,
    expected: &ArtifactPin,
) -> Result<DecodedDescriptor, ApiError> {
    let raw = payload::read(reader, expected.bytes(), MAX_DESCRIPTOR_BYTES)?;
    if raw.sha256 != expected.sha256() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Descriptor SHA mismatch",
        ));
    }
    let (profile, input) = payload::parse::<DescriptorDto>(&raw.bytes)?.checked()?;
    Ok(DecodedDescriptor {
        raw: expected.clone(),
        profile,
        input,
    })
}
