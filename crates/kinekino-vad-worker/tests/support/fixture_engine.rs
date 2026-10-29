use super::{
    fixture_barrier,
    fixture_data::{FixtureData, MODEL},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::{
    application::{AnalysisControl, EngineCompletion, ScoreSink, VadEngine},
    domain::VadProfile,
};
use std::{path::Path, time::Duration};
#[derive(Clone, Copy)]
pub enum FixtureMode {
    Complete,
    WaitCancel,
}
impl FixtureMode {
    pub fn parse(value: &str) -> Result<Self, ApiError> {
        match value {
            "complete" => Ok(Self::Complete),
            "wait-cancel" => Ok(Self::WaitCancel),
            _ => Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Unknown explicit test fixture mode",
            )),
        }
    }
}
pub struct FixtureEngine<'a> {
    pub mode: FixtureMode,
    pub root: &'a Path,
    pub data: &'a FixtureData,
}
impl VadEngine for FixtureEngine<'_> {
    fn run(
        &self,
        profile: &VadProfile,
        model: &[u8],
        runtime: &[u8],
        samples: &[i16],
        control: &dyn AnalysisControl,
        scores: &mut dyn ScoreSink,
    ) -> Result<EngineCompletion, ApiError> {
        if profile != &self.data.profile
            || model != MODEL
            || runtime.len() as u64 != profile.runtime().bytes()
            || samples.len() != 513
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Fixed synthetic engine input differs",
            ));
        }
        if control.checkpoint()? {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original fixture cancellation before work",
            ));
        }
        // One synthetic frame is processed before parent cancellation may be Partial.
        scores.push(0.2)?;
        let _ready =
            fixture_barrier::publish(self.root, fixture_barrier::ENGINE_READY, b"ready\n")?;
        match self.mode {
            FixtureMode::Complete => {
                if control.checkpoint()? {
                    return Ok(EngineCompletion::Partial {
                        processed_samples: 512,
                    });
                }
                scores.push(0.7)?;
                Ok(EngineCompletion::Complete)
            }
            FixtureMode::WaitCancel => loop {
                if control.checkpoint()? {
                    return Ok(EngineCompletion::Partial {
                        processed_samples: 512,
                    });
                }
                if fixture_barrier::released(self.root, fixture_barrier::ENGINE_RELEASE)? {
                    if control.checkpoint()? {
                        return Ok(EngineCompletion::Partial {
                            processed_samples: 512,
                        });
                    }
                    scores.push(0.7)?;
                    return Ok(EngineCompletion::Complete);
                }
                std::thread::sleep(Duration::from_millis(1));
            },
        }
    }
}
