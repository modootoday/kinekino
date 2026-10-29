use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::{application::*, domain::*};
use sha2::{Digest, Sha256};
use std::{
    cell::Cell,
    collections::BTreeMap,
    io::{Cursor, Read},
    time::{Duration, Instant},
};

fn pin(id: &str, bytes: &[u8]) -> ArtifactPin {
    ArtifactPin::new(
        id.into(),
        format!("{:x}", Sha256::digest(bytes)),
        DecimalU64(bytes.len() as u64),
    )
    .unwrap()
}
fn wav(samples: u32) -> Vec<u8> {
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(36 + samples * 2).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt \x10\0\0\0\x01\0\x01\0\x80\x3e\0\0\0\x7d\0\0\x02\0\x10\0data");
    out.extend_from_slice(&(samples * 2).to_le_bytes());
    out.resize(44 + samples as usize * 2, 0);
    out
}
struct Reader(BTreeMap<String, Vec<u8>>);
#[test]
fn cancellation_at_actual_eof_is_checked_after_verified_hash_before_return() {
    struct EofReader<'a> {
        bytes: &'a [u8],
        original: &'a Control,
    }
    struct EofStream<'a> {
        cursor: Cursor<&'a [u8]>,
        original: &'a Control,
    }
    impl Read for EofStream<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let count = self.cursor.read(buffer)?;
            if count == 0 {
                self.original.cancelled.set(true);
            }
            Ok(count)
        }
    }
    impl ArtifactReader for EofReader<'_> {
        fn open(&self, _: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
            Ok(Box::new(EofStream {
                cursor: Cursor::new(self.bytes),
                original: self.original,
            }))
        }
    }
    let original = Control::fresh();
    let bytes = b"exact original artifact";
    let reader = EofReader {
        bytes,
        original: &original,
    };
    let error = read_verified_artifact(&reader, &pin("eof", bytes), MAX_RUNTIME_BYTES, &original)
        .unwrap_err();
    assert!(
        original.cancelled.get(),
        "The underlying reader must reach actual EOF"
    );
    assert_eq!(error.code, ErrorCode::Cancelled);
}
#[test]
fn public_artifact_reader_caps_size_before_opening_even_with_unbounded_caller_maximum() {
    struct NeverOpen;
    impl ArtifactReader for NeverOpen {
        fn open(&self, _: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
            panic!("Oversized pin must be rejected before opening");
        }
    }
    let oversized =
        ArtifactPin::new("oversized".into(), "0".repeat(64), DecimalU64(u64::MAX)).unwrap();
    assert_eq!(
        read_verified_artifact(&NeverOpen, &oversized, u64::MAX, &Control::fresh())
            .err()
            .unwrap()
            .code,
        ErrorCode::ResourceExhausted
    );
}
impl ArtifactReader for Reader {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
        self.0
            .get(pin.id())
            .map(|b| Box::new(Cursor::new(b.as_slice())) as Box<dyn Read>)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Fixture artifact absent"))
    }
}
struct Admission {
    allowed: VadProfile,
}
impl ProfileAdmission for Admission {
    fn authorize(&self, profile: &VadProfile) -> Result<(), ApiError> {
        if profile == &self.allowed {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Fixture admission differs",
            ))
        }
    }
}
struct Control {
    cancelled: Cell<bool>,
    deadline: Instant,
}
impl Control {
    fn fresh() -> Self {
        Self {
            cancelled: Cell::new(false),
            deadline: Instant::now() + Duration::from_secs(30),
        }
    }
}
impl AnalysisControl for Control {
    fn cancellation_requested(&self) -> bool {
        self.cancelled.get()
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
}
struct Engine {
    mode: u8,
    calls: Cell<u32>,
}
impl Engine {
    fn new(mode: u8) -> Self {
        Self {
            mode,
            calls: Cell::new(0),
        }
    }
}
impl VadEngine for Engine {
    fn run(
        &self,
        profile: &VadProfile,
        model: &[u8],
        runtime: &[u8],
        samples: &[i16],
        _control: &dyn AnalysisControl,
        scores: &mut dyn ScoreSink,
    ) -> Result<EngineCompletion, ApiError> {
        self.calls.set(self.calls.get() + 1);
        assert_eq!(profile.precision(), "fp32");
        assert_eq!((profile.intra_threads(), profile.inter_threads()), (2, 1));
        assert_eq!(model, b"fixture-model");
        assert_eq!(runtime, b"fixture-runtime");
        assert_eq!(samples.len(), 513);
        match self.mode {
            1 => {
                scores.push(0.6)?;
                Ok(EngineCompletion::Partial {
                    processed_samples: 512,
                })
            }
            2 => {
                let _ = scores.push(f32::NAN);
                scores.push(0.4)?;
                Ok(EngineCompletion::Complete)
            }
            3 => {
                scores.push(0.6)?;
                Ok(EngineCompletion::Complete)
            }
            4 => {
                scores.push(0.6)?;
                scores.push(0.4)?;
                let _ = scores.push(0.1);
                Ok(EngineCompletion::Complete)
            }
            5 => {
                scores.push(0.6)?;
                Ok(EngineCompletion::Partial {
                    processed_samples: 514,
                })
            }
            6 => {
                scores.push(-0.1)?;
                Ok(EngineCompletion::Complete)
            }
            7 => {
                scores.push(f32::INFINITY)?;
                Ok(EngineCompletion::Complete)
            }
            _ => {
                scores.push(0.6)?;
                scores.push(0.4)?;
                Ok(EngineCompletion::Complete)
            }
        }
    }
}
fn fixture() -> (VadProfile, VadInput, Admission, Reader) {
    let audio = wav(513);
    let profile = VadProfile::new(
        "fixture-only".into(),
        "onnx-community/silero-vad".into(),
        "1".repeat(40),
        pin("model", b"fixture-model"),
        pin("license", b"fixture-license"),
        pin("runtime", b"fixture-runtime"),
    )
    .unwrap();
    let input = VadInput::new(
        pin("wav", &audio),
        pin("pcm", &audio[44..]),
        DecimalU64(513),
        DecimalU64(0),
        DecimalU64(513),
    )
    .unwrap();
    let reader = Reader(BTreeMap::from([
        ("model".into(), b"fixture-model".to_vec()),
        ("runtime".into(), b"fixture-runtime".to_vec()),
        ("license".into(), b"fixture-license".to_vec()),
        ("wav".into(), audio),
    ]));
    let admission = Admission {
        allowed: profile.clone(),
    };
    (profile, input, admission, reader)
}
fn code<T>(result: Result<T, ApiError>) -> ErrorCode {
    match result {
        Err(e) => e.code,
        Ok(_) => panic!("Unexpected success"),
    }
}

struct WorkerControl {
    cancelled: Cell<bool>,
    cutoff: Cell<Instant>,
}
impl WorkerControl {
    fn fresh() -> Self {
        Self {
            cancelled: Cell::new(false),
            cutoff: Cell::new(Instant::now() + Duration::from_secs(30)),
        }
    }
}
impl AnalysisControl for WorkerControl {
    fn cancellation_requested(&self) -> bool {
        self.cancelled.get()
    }
    fn deadline(&self) -> Instant {
        self.cutoff.get()
    }
}
struct WorkerEngine<'a> {
    original: &'a WorkerControl,
    mode: u8,
}
impl VadEngine for WorkerEngine<'_> {
    fn run(
        &self,
        _: &VadProfile,
        _: &[u8],
        _: &[u8],
        samples: &[i16],
        _: &dyn AnalysisControl,
        scores: &mut dyn ScoreSink,
    ) -> Result<EngineCompletion, ApiError> {
        assert_eq!(samples.len(), 513);
        self.original.cancelled.set(true);
        if self.mode == 2 {
            return Err(ApiError::new(ErrorCode::Cancelled, "Original engine error"));
        }
        if self.mode == 4 {
            let _ = scores.push(f32::NAN);
            return Ok(EngineCompletion::Partial {
                processed_samples: 512,
            });
        }
        if self.mode == 6 {
            return Ok(EngineCompletion::Partial {
                processed_samples: 0,
            });
        }
        scores.push(0.6)?;
        if self.mode == 1 {
            scores.push(0.4)?;
            return Ok(EngineCompletion::Complete);
        }
        if self.mode == 5 {
            self.original.cutoff.set(Instant::now());
        }
        Ok(EngineCompletion::Partial {
            processed_samples: if self.mode == 3 { 513 } else { 512 },
        })
    }
}
#[test]
fn strict_analysis_still_rejects_cancelled_engine_partial() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    assert_eq!(
        code(analyze(
            &p,
            &i,
            &a,
            &r,
            &WorkerEngine {
                original: &original,
                mode: 0
            },
            &original
        )),
        ErrorCode::Cancelled
    );
    assert!(original.cancelled.get());
}
#[test]
fn worker_observation_preserves_checked_partial_and_original_cancel_flag() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    let outcome = observe_worker_analysis(
        &p,
        &i,
        &a,
        &r,
        &WorkerEngine {
            original: &original,
            mode: 0,
        },
        &original,
    )
    .unwrap();
    match outcome {
        WorkerAnalysisObservation::Partial(partial) => {
            assert_eq!(partial.processed_samples(), 512);
            assert_eq!(partial.scores(), &[0.6]);
            assert_eq!(partial.input().wav(), i.wav());
        }
        WorkerAnalysisObservation::Inferred(_) => {
            panic!("Cancellation observation became complete")
        }
    }
    assert!(original.cancelled.get());
}
#[test]
fn worker_complete_followed_by_cancel_is_still_cancelled_error() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    assert_eq!(
        code(observe_worker_analysis(
            &p,
            &i,
            &a,
            &r,
            &WorkerEngine {
                original: &original,
                mode: 1
            },
            &original
        )),
        ErrorCode::Cancelled
    );
}
#[test]
fn worker_engine_cancelled_error_is_not_rewritten_to_partial() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    let error = observe_worker_analysis(
        &p,
        &i,
        &a,
        &r,
        &WorkerEngine {
            original: &original,
            mode: 2,
        },
        &original,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.message, "Original engine error");
}
#[test]
fn worker_observation_rejects_inconsistent_partial_samples_and_scores() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    assert_eq!(
        code(observe_worker_analysis(
            &p,
            &i,
            &a,
            &r,
            &WorkerEngine {
                original: &original,
                mode: 3
            },
            &original
        )),
        ErrorCode::InvalidArgument
    );
}
#[test]
fn worker_observation_rejects_ignored_sink_error() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    assert_eq!(
        code(observe_worker_analysis(
            &p,
            &i,
            &a,
            &r,
            &WorkerEngine {
                original: &original,
                mode: 4
            },
            &original
        )),
        ErrorCode::EngineFailed
    );
}
#[test]
fn worker_partial_after_original_cutoff_stays_timeout_error() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    assert_eq!(
        code(observe_worker_analysis(
            &p,
            &i,
            &a,
            &r,
            &WorkerEngine {
                original: &original,
                mode: 5
            },
            &original
        )),
        ErrorCode::Timeout
    );
}
#[test]
fn worker_cancelled_before_analysis_does_not_call_engine() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    original.cancelled.set(true);
    let engine = Engine::new(0);
    assert_eq!(
        code(observe_worker_analysis(&p, &i, &a, &r, &engine, &original)),
        ErrorCode::Cancelled
    );
    assert_eq!(engine.calls.get(), 0);
}

#[test]
fn worker_zero_processed_cancel_does_not_fabricate_partial_observation() {
    let (p, i, a, r) = fixture();
    let original = WorkerControl::fresh();
    assert_eq!(
        code(observe_worker_analysis(
            &p,
            &i,
            &a,
            &r,
            &WorkerEngine {
                original: &original,
                mode: 6
            },
            &original
        )),
        ErrorCode::Cancelled
    );
    assert!(original.cancelled.get());
}

#[test]
fn same_application_verifies_bytes_and_retains_exact_provenance() {
    let (p, i, a, r) = fixture();
    let engine = Engine::new(0);
    match analyze(&p, &i, &a, &r, &engine, &Control::fresh()).unwrap() {
        AnalysisOutcome::Complete(result) => {
            assert_eq!(result.profile(), &p);
            assert_eq!(result.input().wav(), i.wav());
            assert_eq!(result.input().pcm(), i.pcm());
            assert_eq!(result.input().samples(), 513);
            assert_eq!((result.input().start(), result.input().end()), (0, 513));
            assert_eq!(result.scores(), &[0.6, 0.4]);
            assert_eq!(result.speech_frames(), 1);
            assert_eq!(result.last_frame_valid_samples(), 1);
        }
        AnalysisOutcome::Partial(_) => panic!("Unexpected partial"),
    }
    assert_eq!(engine.calls.get(), 1);
}
#[test]
fn every_artifact_is_rehashed_instead_of_trusting_reader_metadata() {
    for id in ["model", "runtime", "license", "wav"] {
        for mutation in 0..3 {
            let (p, i, a, mut r) = fixture();
            match mutation {
                0 => r.0.get_mut(id).unwrap()[0] ^= 1,
                1 => {
                    r.0.get_mut(id).unwrap().push(0);
                }
                _ => {
                    r.0.get_mut(id).unwrap().pop();
                }
            }
            let engine = Engine::new(0);
            assert_eq!(
                code(analyze(&p, &i, &a, &r, &engine, &Control::fresh())),
                ErrorCode::SourceExpired
            );
            assert_eq!(engine.calls.get(), 0);
        }
    }
}
#[test]
fn pcm_identity_and_wav_format_are_checked_independently() {
    let (p, i, a, mut r) = fixture();
    let audio = r.0.get_mut("wav").unwrap();
    audio[24] = 0; // WAV claims another rate; outer SHA is updated deliberately.
    let changed = VadInput::new(
        pin("wav", audio),
        i.pcm().clone(),
        DecimalU64(513),
        DecimalU64(0),
        DecimalU64(513),
    )
    .unwrap();
    assert_eq!(
        code(analyze(
            &p,
            &changed,
            &a,
            &r,
            &Engine::new(0),
            &Control::fresh()
        )),
        ErrorCode::InvalidArgument
    );
    let (p, i, a, r) = fixture();
    let changed = VadInput::new(
        i.wav().clone(),
        pin("pcm", &vec![1; 1026]),
        DecimalU64(513),
        DecimalU64(0),
        DecimalU64(513),
    )
    .unwrap();
    assert_eq!(
        code(analyze(
            &p,
            &changed,
            &a,
            &r,
            &Engine::new(0),
            &Control::fresh()
        )),
        ErrorCode::SourceExpired
    );
}
#[test]
fn admission_does_not_accept_a_different_runtime_profile() {
    let (p, i, a, r) = fixture();
    let changed = VadProfile::new(
        "fixture-only".into(),
        p.repo().into(),
        p.revision().into(),
        p.weights().clone(),
        p.license_document().clone(),
        pin("runtime", b"changed"),
    )
    .unwrap();
    let engine = Engine::new(0);
    assert_eq!(
        code(analyze(&changed, &i, &a, &r, &engine, &Control::fresh())),
        ErrorCode::PermissionDenied
    );
    assert_eq!(engine.calls.get(), 0);
}
#[test]
fn partial_is_never_a_complete_candidate_and_invalid_scores_are_rejected() {
    let (p, i, a, r) = fixture();
    match analyze(&p, &i, &a, &r, &Engine::new(1), &Control::fresh()).unwrap() {
        AnalysisOutcome::Partial(observed) => {
            assert_eq!(observed.processed_samples(), 512);
            assert_eq!(observed.scores(), &[0.6]);
            assert_eq!(observed.profile(), &p);
            assert_eq!(observed.input().wav(), i.wav());
            assert_eq!(observed.input().pcm(), i.pcm());
            assert_eq!((observed.input().start(), observed.input().end()), (0, 513));
        }
        AnalysisOutcome::Complete(_) => panic!("Partial became complete"),
    }
    for mode in [2, 3, 4, 5, 6, 7] {
        assert!(analyze(&p, &i, &a, &r, &Engine::new(mode), &Control::fresh()).is_err());
    }
}

#[test]
fn completion_after_cancellation_cannot_return_a_complete_candidate() {
    struct LateCancel<'a>(&'a Cell<bool>);
    impl VadEngine for LateCancel<'_> {
        fn run(
            &self,
            _: &VadProfile,
            _: &[u8],
            _: &[u8],
            _: &[i16],
            _: &dyn AnalysisControl,
            sink: &mut dyn ScoreSink,
        ) -> Result<EngineCompletion, ApiError> {
            sink.push(0.6)?;
            sink.push(0.4)?;
            self.0.set(true);
            Ok(EngineCompletion::Complete)
        }
    }
    let (p, i, a, r) = fixture();
    let control = Control::fresh();
    assert_eq!(
        code(analyze(
            &p,
            &i,
            &a,
            &r,
            &LateCancel(&control.cancelled),
            &control
        )),
        ErrorCode::Cancelled
    );
}

#[test]
fn malformed_chunk_lengths_and_duplicate_data_do_not_reach_engine() {
    for mode in 0..3 {
        let (p, i, a, mut r) = fixture();
        let bytes = r.0.get_mut("wav").unwrap();
        match mode {
            0 => bytes[40..44].copy_from_slice(&u32::MAX.to_le_bytes()),
            1 => {
                bytes.extend_from_slice(b"data\x02\0\0\0\0\0");
                let riff_len = (bytes.len() - 8) as u32;
                bytes[4..8].copy_from_slice(&riff_len.to_le_bytes());
            }
            _ => {
                bytes.pop();
            }
        }
        let changed = VadInput::new(
            pin("wav", bytes),
            i.pcm().clone(),
            DecimalU64(513),
            DecimalU64(0),
            DecimalU64(513),
        )
        .unwrap();
        let engine = Engine::new(0);
        assert_eq!(
            code(analyze(&p, &changed, &a, &r, &engine, &Control::fresh())),
            ErrorCode::InvalidArgument
        );
        assert_eq!(engine.calls.get(), 0);
    }
}
#[test]
fn cancellation_and_deadline_do_not_execute_or_claim_cleanup() {
    let (p, i, a, r) = fixture();
    let control = Control::fresh();
    control.cancelled.set(true);
    let engine = Engine::new(0);
    assert_eq!(
        code(analyze(&p, &i, &a, &r, &engine, &control)),
        ErrorCode::Cancelled
    );
    let control = Control {
        cancelled: Cell::new(false),
        deadline: Instant::now(),
    };
    assert_eq!(
        code(analyze(&p, &i, &a, &r, &engine, &control)),
        ErrorCode::Timeout
    );
    assert_eq!(engine.calls.get(), 0);
}
#[test]
fn arithmetic_and_pins_are_bounded_before_allocation() {
    let (_, i, _, _) = fixture();
    for (samples, start, end) in [
        (u64::MAX, 0, 1),
        (513, 0, 514),
        (513, 1, 1),
        (513, u64::MAX, 1),
    ] {
        assert!(
            VadInput::new(
                i.wav().clone(),
                i.pcm().clone(),
                DecimalU64(samples),
                DecimalU64(start),
                DecimalU64(end)
            )
            .is_err()
        );
    }
    assert!(ArtifactPin::new("../escape".into(), "a".repeat(64), DecimalU64(1)).is_err());
    assert!(ArtifactPin::new("id".into(), "A".repeat(64), DecimalU64(1)).is_err());
    let fixed =
        VadProfile::cached_silero(pin("license", b"fixture"), pin("runtime", b"fixture")).unwrap();
    assert_eq!(fixed.weights().bytes(), 2_243_022);
    assert_eq!(fixed.revision(), "e71cae966052b992a7eca6b17738916ce0eca4ec");
    assert!(
        VadProfile::new(
            "oversize".into(),
            fixed.repo().into(),
            fixed.revision().into(),
            ArtifactPin::new(
                "weights".into(),
                "a".repeat(64),
                DecimalU64(MAX_MODEL_BYTES + 1)
            )
            .unwrap(),
            fixed.license_document().clone(),
            fixed.runtime().clone()
        )
        .is_err()
    );
}

#[test]
fn nonzero_range_preserves_signed_pcm_and_skips_padded_metadata() {
    struct SelectedRange;
    impl VadEngine for SelectedRange {
        fn run(
            &self,
            _: &VadProfile,
            _: &[u8],
            _: &[u8],
            samples: &[i16],
            _: &dyn AnalysisControl,
            scores: &mut dyn ScoreSink,
        ) -> Result<EngineCompletion, ApiError> {
            assert_eq!(samples, &[-1, 0, 1]);
            scores.push(0.7)?;
            Ok(EngineCompletion::Complete)
        }
    }
    let (profile, _, admission, mut reader) = fixture();
    let mut audio = wav(5);
    for (bytes, sample) in
        audio[44..]
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .zip([i16::MIN, -1, 0, 1, i16::MAX])
    {
        bytes.copy_from_slice(&sample.to_le_bytes());
    }
    let pcm = audio[44..].to_vec();
    // Odd-size non-audio chunks include a padding byte outside the chunk size.
    audio.extend_from_slice(b"JUNK\x01\0\0\0\x7f\0");
    let riff_size = (audio.len() - 8) as u32;
    audio[4..8].copy_from_slice(&riff_size.to_le_bytes());
    let input = VadInput::new(
        pin("wav", &audio),
        pin("pcm", &pcm),
        DecimalU64(5),
        DecimalU64(1),
        DecimalU64(4),
    )
    .unwrap();
    reader.0.insert("wav".into(), audio);
    match analyze(
        &profile,
        &input,
        &admission,
        &reader,
        &SelectedRange,
        &Control::fresh(),
    )
    .unwrap()
    {
        AnalysisOutcome::Complete(result) => {
            assert_eq!((result.input().start(), result.input().end()), (1, 4));
            assert_eq!(result.input().samples(), 5);
            assert_eq!(result.scores(), &[0.7]);
            assert_eq!(result.last_frame_valid_samples(), 3);
        }
        AnalysisOutcome::Partial(_) => panic!("Selected input did not complete"),
    }
}

#[test]
fn partial_nonzero_range_retains_complete_profile_and_input_context() {
    struct PartialRange;
    impl VadEngine for PartialRange {
        fn run(
            &self,
            _: &VadProfile,
            _: &[u8],
            _: &[u8],
            samples: &[i16],
            _: &dyn AnalysisControl,
            scores: &mut dyn ScoreSink,
        ) -> Result<EngineCompletion, ApiError> {
            assert_eq!(samples.len(), 512);
            scores.push(0.25)?;
            Ok(EngineCompletion::Partial {
                processed_samples: 512,
            })
        }
    }
    let (profile, input, admission, reader) = fixture();
    let selected = VadInput::new(
        input.wav().clone(),
        input.pcm().clone(),
        DecimalU64(513),
        DecimalU64(1),
        DecimalU64(513),
    )
    .unwrap();
    match analyze(
        &profile,
        &selected,
        &admission,
        &reader,
        &PartialRange,
        &Control::fresh(),
    )
    .unwrap()
    {
        AnalysisOutcome::Partial(observed) => {
            assert_eq!(observed.profile(), &profile);
            assert_eq!(observed.profile().weights(), profile.weights());
            assert_eq!(observed.profile().runtime(), profile.runtime());
            assert_eq!(
                observed.profile().license_document(),
                profile.license_document()
            );
            assert_eq!(observed.input().wav(), input.wav());
            assert_eq!(observed.input().pcm(), input.pcm());
            assert_eq!(observed.input().samples(), 513);
            assert_eq!((observed.input().start(), observed.input().end()), (1, 513));
            assert_eq!(observed.processed_samples(), 512);
            assert_eq!(observed.scores(), &[0.25]);
        }
        AnalysisOutcome::Complete(_) => panic!("Partial context became complete"),
    }
}
