use super::{
    AnalysisControl, ArtifactReader, EngineCompletion, ProfileAdmission, ScoreSink, VadEngine,
};
use crate::domain::*;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use sha2::{Digest, Sha256};
use std::io::Read;

struct Scores {
    values: Vec<f32>,
    limit: usize,
    rejected: bool,
}
impl ScoreSink for Scores {
    fn push(&mut self, score: f32) -> Result<(), ApiError> {
        if self.rejected || self.values.len() >= self.limit {
            self.rejected = true;
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "VAD score bound exceeded",
            ));
        }
        if !score.is_finite() || !(0.0..=1.0).contains(&score) {
            self.rejected = true;
            return Err(ApiError::new(ErrorCode::EngineFailed, "Invalid VAD score"));
        }
        self.values.push(score);
        Ok(())
    }
}
struct AdmittedProfile<'a> {
    spec: &'a VadProfile,
    model: Vec<u8>,
    runtime: Vec<u8>,
    _license: Vec<u8>,
}
impl<'a> AdmittedProfile<'a> {
    fn read(
        spec: &'a VadProfile,
        admission: &dyn ProfileAdmission,
        reader: &dyn ArtifactReader,
        control: &dyn AnalysisControl,
    ) -> Result<Self, ApiError> {
        admission.authorize(spec)?;
        Ok(Self {
            spec,
            model: read_verified_artifact(reader, spec.weights(), MAX_MODEL_BYTES, control)?,
            runtime: read_verified_artifact(reader, spec.runtime(), MAX_RUNTIME_BYTES, control)?,
            _license: read_verified_artifact(
                reader,
                spec.license_document(),
                MAX_LICENSE_BYTES,
                control,
            )?,
        })
    }
}
fn check(control: &dyn AnalysisControl) -> Result<(), ApiError> {
    if control.checkpoint()? {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Analysis cancellation requested; cleanup is not acknowledged",
        ));
    }
    if std::time::Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Analysis deadline expired",
        ));
    }
    Ok(())
}
/// Reuses analysis bounds, original control checks and exact artifact hashing.
/// The caller supplies its existing control; this creates no deadline or permit.
pub fn read_verified_artifact(
    reader: &dyn ArtifactReader,
    pin: &ArtifactPin,
    max: u64,
    control: &dyn AnalysisControl,
) -> Result<Vec<u8>, ApiError> {
    check(control)?;
    if pin.bytes() > max.min(MAX_RUNTIME_BYTES) {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Artifact exceeds bound",
        ));
    }
    let mut stream = reader.open(pin)?.take(pin.bytes() + 1);
    let mut bytes = Vec::with_capacity(pin.bytes() as usize);
    let mut chunk = [0u8; 8192];
    loop {
        check(control)?;
        let n = stream
            .read(&mut chunk)
            .map_err(|_| ApiError::new(ErrorCode::EngineFailed, "Artifact read failed"))?;
        if n == 0 {
            break;
        }
        if bytes.len() as u64 + n as u64 > pin.bytes() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Artifact exceeds declared bytes",
            ));
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    verify(&bytes, pin)?;
    check(control)?;
    Ok(bytes)
}
fn verify(bytes: &[u8], pin: &ArtifactPin) -> Result<(), ApiError> {
    if bytes.len() as u64 != pin.bytes() || format!("{:x}", Sha256::digest(bytes)) != pin.sha256() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Artifact bytes/SHA differ from pin",
        ));
    }
    Ok(())
}
/// Strict existing PCM16/mono16k RIFF decoder shared by input preparation and P0.
/// Callers retain original byte bounds, pins and immutable guards separately.
pub fn wav_pcm(bytes: &[u8]) -> Result<&[u8], ApiError> {
    let bad = || {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Expected strict mono16k PCM16 RIFF WAV",
        )
    };
    if bytes.len() < 12
        || &bytes[..4] != b"RIFF"
        || &bytes[8..12] != b"WAVE"
        || u32::from_le_bytes(bytes[4..8].try_into().map_err(|_| bad())?) as usize
            != bytes.len() - 8
    {
        return Err(bad());
    }
    let mut offset = 12usize;
    let mut format = false;
    let mut data = None;
    while offset < bytes.len() {
        let header_end = offset
            .checked_add(8)
            .filter(|&n| n <= bytes.len())
            .ok_or_else(bad)?;
        let size = u32::from_le_bytes(
            bytes[offset + 4..header_end]
                .try_into()
                .map_err(|_| bad())?,
        ) as usize;
        let end = header_end
            .checked_add(size)
            .filter(|&n| n <= bytes.len())
            .ok_or_else(bad)?;
        match &bytes[offset..offset + 4] {
            b"fmt " => {
                if format
                    || size != 16
                    || bytes[header_end..end]
                        != [1, 0, 1, 0, 128, 62, 0, 0, 0, 125, 0, 0, 2, 0, 16, 0]
                {
                    return Err(bad());
                }
                format = true;
            }
            b"data" => {
                if data.is_some() || size == 0 || !size.is_multiple_of(2) {
                    return Err(bad());
                }
                data = Some(&bytes[header_end..end]);
            }
            _ => {}
        }
        offset = end
            .checked_add(size % 2)
            .filter(|&n| n <= bytes.len())
            .ok_or_else(bad)?;
    }
    if !format {
        return Err(bad());
    }
    data.ok_or_else(bad)
}

pub fn analyze(
    profile: &VadProfile,
    input: &VadInput,
    admission: &dyn ProfileAdmission,
    reader: &dyn ArtifactReader,
    engine: &dyn VadEngine,
    control: &dyn AnalysisControl,
) -> Result<AnalysisOutcome, ApiError> {
    analyze_inner(
        profile,
        input,
        admission,
        reader,
        engine,
        control,
        CompletionPolicy::Strict,
    )
}

/// Preserve a genuine, validated engine Partial as a worker observation even
/// when the same original control reports cancellation. This is never a success,
/// jobs marker or cleanup ACK. All pre-engine checks retain strict cancellation.
pub fn observe_worker_analysis(
    profile: &VadProfile,
    input: &VadInput,
    admission: &dyn ProfileAdmission,
    reader: &dyn ArtifactReader,
    engine: &dyn VadEngine,
    control: &dyn AnalysisControl,
) -> Result<WorkerAnalysisObservation, ApiError> {
    match analyze_inner(
        profile,
        input,
        admission,
        reader,
        engine,
        control,
        CompletionPolicy::WorkerPartial,
    )? {
        AnalysisOutcome::Complete(result) => Ok(WorkerAnalysisObservation::Inferred(result)),
        AnalysisOutcome::Partial(observation) => {
            Ok(WorkerAnalysisObservation::Partial(observation))
        }
    }
}

enum CompletionPolicy {
    Strict,
    WorkerPartial,
}
fn analyze_inner(
    profile: &VadProfile,
    input: &VadInput,
    admission: &dyn ProfileAdmission,
    reader: &dyn ArtifactReader,
    engine: &dyn VadEngine,
    control: &dyn AnalysisControl,
    policy: CompletionPolicy,
) -> Result<AnalysisOutcome, ApiError> {
    check(control)?;
    let admitted = AdmittedProfile::read(profile, admission, reader, control)?;
    let wav = read_verified_artifact(reader, input.wav(), MAX_AUDIO_BYTES, control)?;
    let pcm = wav_pcm(&wav)?;
    verify(pcm, input.pcm())?;
    if pcm.len() as u64 / 2 != input.samples() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "PCM sample count differs",
        ));
    }
    let selected = &pcm[(input.start() * 2) as usize..(input.end() * 2) as usize];
    let samples: Vec<i16> = selected
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect();
    let mut scores = Scores {
        values: Vec::with_capacity(input.expected_frames() as usize),
        limit: input.expected_frames() as usize,
        rejected: false,
    };
    check(control)?;
    let completion = engine.run(
        admitted.spec,
        &admitted.model,
        &admitted.runtime,
        &samples,
        control,
        &mut scores,
    )?;
    if matches!(policy, CompletionPolicy::WorkerPartial)
        && matches!(completion, EngineCompletion::Partial { .. })
        && control.checkpoint()?
    {
        if matches!(
            completion,
            EngineCompletion::Partial {
                processed_samples: 0
            }
        ) {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Worker stopped before any samples; no partial cancellation observation",
            ));
        }
        if std::time::Instant::now() >= control.deadline() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original worker observation cutoff expired",
            ));
        }
        // The original flag stays set. Only checked Partial may leave below;
        // Complete and engine Err never enter this branch.
    } else {
        check(control)?;
    }
    if scores.rejected {
        return Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Engine ignored rejected score",
        ));
    }
    match completion {
        EngineCompletion::Complete => Ok(AnalysisOutcome::Complete(CompleteVadResult::checked(
            profile.clone(),
            input.clone(),
            scores.values,
        )?)),
        EngineCompletion::Partial { processed_samples } => Ok(AnalysisOutcome::Partial(
            PartialVadObservation::checked(profile, input, processed_samples, scores.values)?,
        )),
    }
}
