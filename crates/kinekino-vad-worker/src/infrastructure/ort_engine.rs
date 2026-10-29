use super::LoadedRuntimeAdmission;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::{
    application::{AnalysisControl, EngineCompletion, ScoreSink, VadEngine},
    domain::{CHUNK_SAMPLES, VadProfile},
};
use std::time::Instant;

/// The P0 application supplies already verified model/runtime bytes and selected PCM.
/// Native session calls are synchronous; parent owns forced deadline/cleanup.
pub struct OrtVadEngine<V: LoadedRuntimeAdmission> {
    runtime: V,
}
impl<V: LoadedRuntimeAdmission> OrtVadEngine<V> {
    pub fn new(runtime: V) -> Self {
        Self { runtime }
    }
}

fn engine(error: impl std::fmt::Display) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, error.to_string())
}
fn stopped(control: &dyn AnalysisControl) -> Result<bool, ApiError> {
    if control.checkpoint()? {
        return Ok(true);
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original VAD cutoff expired",
        ));
    }
    Ok(false)
}
impl<V: LoadedRuntimeAdmission> VadEngine for OrtVadEngine<V> {
    fn run(
        &self,
        profile: &VadProfile,
        model: &[u8],
        _runtime: &[u8],
        samples: &[i16],
        control: &dyn AnalysisControl,
        scores: &mut dyn ScoreSink,
    ) -> Result<EngineCompletion, ApiError> {
        if stopped(control)? {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "VAD stopped before native session; no partial observation",
            ));
        }
        let _prerequisites = self.runtime.before_session(profile)?;
        if stopped(control)? {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "VAD stopped during runtime admission; no partial observation",
            ));
        }
        // Declare before environment/session so their Drop runs before module
        // guards even on early Partial/error returns; initialization stays later.
        let _loaded_modules;
        let environment = ort::init().with_telemetry(false).build().map_err(engine)?;
        let mut session = ort::session::Session::builder(&environment)
            .map_err(engine)?
            .with_intra_threads(profile.intra_threads() as usize)
            .map_err(engine)?
            .with_inter_threads(profile.inter_threads() as usize)
            .map_err(engine)?
            .with_execution_providers([ort::ep::CPU::default().build()])
            .map_err(engine)?
            .commit_from_memory(model)
            .map_err(engine)?;
        _loaded_modules = self.runtime.before_inference(profile)?;
        let mut state = vec![0.0_f32; 256];
        let mut context = [0.0_f32; 64];
        let mut processed = 0_u64;
        for chunk in samples.chunks(CHUNK_SAMPLES as usize) {
            if stopped(control)? {
                if processed == 0 {
                    return Err(ApiError::new(
                        ErrorCode::Cancelled,
                        "VAD stopped before first inference; no partial observation",
                    ));
                }
                return Ok(EngineCompletion::Partial {
                    processed_samples: processed,
                });
            }
            let mut wave = Vec::with_capacity(576);
            wave.extend_from_slice(&context);
            wave.extend(chunk.iter().map(|sample| f32::from(*sample) / 32768.0));
            wave.resize(576, 0.0);
            context.copy_from_slice(&wave[512..576]);
            let outputs = session.run(ort::inputs![
                "input" => ort::value::Tensor::from_array(([1, 576], wave)).map_err(engine)?,
                "state" => ort::value::Tensor::from_array(([2, 1, 128], state)).map_err(engine)?,
                "sr" => ort::value::Tensor::from_array((Vec::<i64>::new(), vec![16000_i64])).map_err(engine)?
            ]).map_err(engine)?;
            let probability_tensor = outputs.get("output").ok_or_else(|| {
                ApiError::new(ErrorCode::EngineFailed, "Silero output tensor absent")
            })?;
            let state_tensor = outputs.get("stateN").ok_or_else(|| {
                ApiError::new(ErrorCode::EngineFailed, "Silero state tensor absent")
            })?;
            let (_, probabilities) = probability_tensor
                .try_extract_tensor::<f32>()
                .map_err(engine)?;
            let (_, next_state) = state_tensor.try_extract_tensor::<f32>().map_err(engine)?;
            if probabilities.len() != 1
                || next_state.len() != 256
                || next_state.iter().any(|value| !value.is_finite())
            {
                return Err(ApiError::new(
                    ErrorCode::EngineFailed,
                    "Invalid Silero tensor output",
                ));
            }
            scores.push(probabilities[0])?;
            state = next_state.to_vec();
            processed += chunk.len() as u64;
        }
        if stopped(control)? {
            return Ok(EngineCompletion::Partial {
                processed_samples: processed,
            });
        }
        Ok(EngineCompletion::Complete)
    }
}
