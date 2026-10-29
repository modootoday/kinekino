use kinekino_media_contracts::errors::ApiError;
use kinekino_vad::protocol::{
    application::{ValidatedWorkerRequest, encode_worker_response},
    domain::WorkerResult,
};
use kinekino_vad::{
    application::{
        AnalysisControl, ArtifactReader, ProfileAdmission, VadEngine, observe_worker_analysis,
    },
    domain::WorkerAnalysisObservation,
};

/// Parent supplies approved immutable artifacts through the existing P0 ports.
/// Genuine engine Partial stays an observation. Errors remain errors; this
/// creates no parent signal, jobs marker, flag masking or cleanup ACK.
pub fn run_request(
    request: &ValidatedWorkerRequest,
    admission: &dyn ProfileAdmission,
    reader: &dyn ArtifactReader,
    engine: &dyn VadEngine,
    control: &dyn AnalysisControl,
) -> Result<Vec<u8>, ApiError> {
    let expected = request.expected();
    let outcome = observe_worker_analysis(
        expected.profile(),
        expected.input(),
        admission,
        reader,
        engine,
        control,
    );
    let result = match outcome {
        Ok(WorkerAnalysisObservation::Inferred(complete)) => WorkerResult::Complete {
            scores: complete.scores().to_vec(),
        },
        Ok(WorkerAnalysisObservation::Partial(partial)) => WorkerResult::Partial {
            processed_samples: partial.processed_samples(),
            scores: partial.scores().to_vec(),
        },
        Err(error) => WorkerResult::Failure {
            code: error.code,
            message: error.message,
        },
    };
    encode_worker_response(request, &result)
}
