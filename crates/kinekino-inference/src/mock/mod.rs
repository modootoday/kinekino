//! In-process `MockBackend`: deterministic outputs for every task (contract reference).
use crate::port::{
    Chunk, InferenceBackend, InferenceCaps, InferenceError, InferenceKind, InferenceRequest,
    InferenceResponse, Output, Provenance, require_task,
};
use kinekino_core::port::Adapter;

#[derive(Debug, Clone)]
pub struct MockBackend {
    caps: InferenceCaps,
    /// Forced failure for error-path tests.
    pub fail_with: Option<InferenceError>,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self {
            caps: InferenceCaps::all(),
            fail_with: None,
        }
    }
}

impl MockBackend {
    pub fn with_caps(caps: InferenceCaps) -> Self {
        Self {
            caps,
            fail_with: None,
        }
    }
}

impl Adapter for MockBackend {
    type Kind = InferenceKind;
    type Caps = InferenceCaps;
    fn kind(&self) -> InferenceKind {
        InferenceKind::Mock
    }
    fn capabilities(&self) -> InferenceCaps {
        self.caps
    }
}

/// Canonical outputs the HF mock-server fixtures also encode (schema parity tests).
pub fn reference_output(request: &InferenceRequest) -> Output {
    match request {
        InferenceRequest::Transcribe { .. } => Output::Transcribe {
            text: "hello world".into(),
            chunks: vec![
                Chunk {
                    start: 0.0,
                    end: 0.5,
                    text: "hello".into(),
                },
                Chunk {
                    start: 0.5,
                    end: 1.0,
                    text: "world".into(),
                },
            ],
        },
        InferenceRequest::Diarize { .. } => Output::Diarize {
            segments: vec![(0.0, 1.0, "SPEAKER_00".into())],
        },
        InferenceRequest::Ocr { .. } => Output::Ocr {
            text: "TITLE".into(),
        },
        InferenceRequest::SegmentSubject { .. } => Output::SegmentSubject {
            mask_png: vec![0x89, b'P', b'N', b'G'],
        },
        InferenceRequest::Generate { .. } => Output::Generate { text: "ok".into() },
        InferenceRequest::Embed { .. } => Output::Embed {
            vector: vec![0.25, 0.5, 0.25],
        },
    }
}

impl InferenceBackend for MockBackend {
    fn estimate_usd(&self, _request: &InferenceRequest) -> Option<f64> {
        Some(0.0)
    }

    fn run(
        &mut self,
        request: &InferenceRequest,
        cancel: &dyn Fn() -> bool,
    ) -> Result<InferenceResponse, InferenceError> {
        require_task(self.caps, request)?;
        if cancel() {
            return Err(InferenceError::Cancelled);
        }
        if let Some(e) = &self.fail_with {
            return Err(e.clone());
        }
        Ok(InferenceResponse {
            output: reference_output(request),
            provenance: Provenance {
                backend: "mock".into(),
                model: request.model().into(),
                provider: None,
                cost_estimate_usd: 0.0,
                fallback_from: None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::Task;
    use kinekino_core::port::contract;

    #[test]
    fn mock_meets_contract() {
        let mut m = MockBackend::with_caps(InferenceCaps::of(&[Task::Generate]));
        contract::check_identity(&m, InferenceCaps::all());
        let r = InferenceRequest::Embed {
            model: "e".into(),
            text: "t".into(),
        };
        contract::check_unsupported("embed", m.run(&r, &|| false));
        let g = InferenceRequest::Generate {
            model: "g".into(),
            prompt: "p".into(),
        };
        assert_eq!(m.run(&g, &|| true), Err(InferenceError::Cancelled));
        assert_eq!(m.run(&g, &|| false).unwrap().output.task(), Task::Generate);
    }
}
