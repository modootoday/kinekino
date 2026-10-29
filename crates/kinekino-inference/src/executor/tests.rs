use super::*;
use crate::consent::ConsentScope;
use crate::mock::MockBackend;
use crate::port::InferenceKind;
use crate::settings::Budget;
use kinekino_core::port::Adapter;

pub(super) fn track(secs: u32) -> PcmTrack {
    PcmTrack {
        sample_rate: 16_000,
        samples: (0..16_000 * secs).map(|i| (i % 64) as i16).collect(),
    }
}

pub(super) fn job(confirmed: bool) -> TranscribeJob<'static> {
    TranscribeJob {
        project: "p",
        job: "j",
        month: "2026-10",
        model: "openai/whisper-large-v3",
        confirmed,
    }
}

pub(super) fn granted() -> Consent {
    let mut c = Consent::default();
    c.grant(ConsentScope::Project, "p", "j");
    c
}

pub(super) fn regions() -> Vec<SpeechRegion> {
    vec![
        SpeechRegion {
            start: 1.0,
            end: 2.0,
        },
        SpeechRegion {
            start: 5.0,
            end: 5.6,
        },
    ]
}

/// Mock with a fixed per-call cost and a selectable kind.
pub(super) struct Priced(
    pub(super) MockBackend,
    pub(super) f64,
    pub(super) InferenceKind,
);

impl Adapter for Priced {
    type Kind = InferenceKind;
    type Caps = crate::port::InferenceCaps;
    fn kind(&self) -> InferenceKind {
        self.2
    }
    fn capabilities(&self) -> Self::Caps {
        self.0.capabilities()
    }
}

impl InferenceBackend for Priced {
    fn estimate_usd(&self, _r: &InferenceRequest) -> Option<f64> {
        Some(self.1)
    }
    fn run(
        &mut self,
        r: &InferenceRequest,
        cancel: &dyn Fn() -> bool,
    ) -> Result<crate::InferenceResponse, InferenceError> {
        let mut resp = self.0.run(r, cancel)?;
        resp.provenance.cost_estimate_usd = self.1;
        Ok(resp)
    }
}

#[test]
fn regions_merge_pad_split_and_clamp() {
    let scores = [0.1, 0.9, 0.8, 0.2, 0.9, 0.1, 0.1, 0.1, 0.95, 0.9];
    let r = speech_regions(&scores, 1.0, 0.5, 1.0, 0.25, 0.0, 10.0);
    assert_eq!(
        r,
        vec![
            SpeechRegion {
                start: 0.75,
                end: 5.25
            },
            SpeechRegion {
                start: 7.75,
                end: 10.0
            },
        ]
    );
    let split = speech_regions(&scores, 1.0, 0.5, 1.0, 0.0, 2.0, 10.0);
    assert_eq!(split.len(), 3);
    assert!(split.iter().all(|s| s.end - s.start <= 2.0 + 1e-9));
    assert!(speech_regions(&[0.1; 4], 1.0, 0.5, 0.0, 0.0, 0.0, 4.0).is_empty());
}

#[test]
fn wav_header_and_bounds() {
    let t = track(3);
    let wav = wav_segment(
        &t,
        SpeechRegion {
            start: 1.0,
            end: 1.5,
        },
    )
    .unwrap();
    assert_eq!(&wav[..4], b"RIFF");
    assert_eq!(&wav[8..16], b"WAVEfmt ");
    assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 16_000);
    assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 16_000);
    assert_eq!(wav.len(), 44 + 16_000);
    for bad in [(2.0, 1.0), (-1.0, 1.0), (4.0, 5.0)] {
        let r = SpeechRegion {
            start: bad.0,
            end: bad.1,
        };
        assert!(matches!(
            wav_segment(&t, r),
            Err(InferenceError::InvalidInput(_))
        ));
    }
}

#[test]
fn transcribes_regions_with_track_offsets_and_ledger() {
    let mut backend = Priced(MockBackend::default(), 0.01, InferenceKind::HfCloud);
    let mut guard = CostGuard::new(Budget::default());
    let t = transcribe(
        &mut backend,
        &mut guard,
        &granted(),
        &job(false),
        &track(6),
        &regions(),
        &|| false,
    )
    .unwrap();
    assert_eq!(t.text, "hello world hello world");
    let starts: Vec<f64> = t.chunks.iter().map(|c| c.start).collect();
    assert_eq!(starts, vec![1.0, 1.5, 5.0, 5.5]);
    assert!(t.chunks.iter().all(|c| c.end <= 5.6 + 1e-9));
    assert_eq!(t.provenance.len(), 2);
    assert!((t.cost_usd - 0.02).abs() < 1e-9);
    assert!((guard.month_total("2026-10") - 0.02).abs() < 1e-9);
}

#[test]
fn cloud_needs_consent_but_local_pack_does_not() {
    let mut cloud = Priced(MockBackend::default(), 0.0, InferenceKind::HfCloud);
    let mut guard = CostGuard::new(Budget::default());
    let err = transcribe(
        &mut cloud,
        &mut guard,
        &Consent::default(),
        &job(false),
        &track(6),
        &regions(),
        &|| false,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        TranscribeError::Failed {
            region: 0,
            error: InferenceError::ConsentRequired(_),
            ..
        }
    ));
    assert_eq!(guard.total(), 0.0);
    let mut local = Priced(MockBackend::default(), 0.0, InferenceKind::LocalPack);
    assert!(
        transcribe(
            &mut local,
            &mut guard,
            &Consent::default(),
            &job(false),
            &track(6),
            &regions(),
            &|| false
        )
        .is_ok()
    );
}

#[test]
fn guard_confirm_blocks_until_confirmed_and_refusal_keeps_partial() {
    let budget = Budget {
        total_usd: 1.0,
        per_call_usd: 0.5,
        monthly_usd: 10.0,
        confirm_ratio: 0.8,
    };
    let mut backend = Priced(MockBackend::default(), 0.45, InferenceKind::HfCloud);
    let mut guard = CostGuard::new(budget.clone());
    let err = transcribe(
        &mut backend,
        &mut guard,
        &granted(),
        &job(false),
        &track(6),
        &regions(),
        &|| false,
    )
    .unwrap_err();
    assert!(
        matches!(err, TranscribeError::NeedsConfirm { region: 1, .. }),
        "{err:?}"
    );
    let mut guard = CostGuard::new(budget);
    guard.record(LedgerEntry {
        month: "2026-10".into(),
        backend: "x".into(),
        model: "m".into(),
        cost_usd: 0.2,
    });
    let err = transcribe(
        &mut backend,
        &mut guard,
        &granted(),
        &job(true),
        &track(6),
        &regions(),
        &|| false,
    )
    .unwrap_err();
    let TranscribeError::Failed {
        region,
        error,
        partial,
    } = err
    else {
        panic!()
    };
    assert_eq!(region, 1);
    assert!(matches!(error, InferenceError::Budget(_)));
    assert_eq!(partial.chunks.len(), 2);
    assert!((guard.total() - 0.65).abs() < 1e-9);
}

#[test]
fn cancel_and_backend_errors_stop_at_region() {
    let mut backend = MockBackend::default();
    let mut guard = CostGuard::new(Budget::default());
    let calls = std::cell::Cell::new(0);
    let cancel = || {
        calls.set(calls.get() + 1);
        calls.get() > 2
    };
    let err = transcribe(
        &mut backend,
        &mut guard,
        &granted(),
        &job(false),
        &track(6),
        &regions(),
        &cancel,
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            TranscribeError::Failed {
                region: 1,
                error: InferenceError::Cancelled,
                ..
            }
        ),
        "{err:?}"
    );
    backend.fail_with = Some(InferenceError::RateLimited("429".into()));
    let err = transcribe(
        &mut backend,
        &mut guard,
        &granted(),
        &job(false),
        &track(6),
        &regions(),
        &|| false,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        TranscribeError::Failed {
            region: 0,
            error: InferenceError::RateLimited(_),
            ..
        }
    ));
    let bad = [SpeechRegion {
        start: 9.0,
        end: 10.0,
    }];
    let err = transcribe(
        &mut MockBackend::default(),
        &mut guard,
        &granted(),
        &job(false),
        &track(6),
        &bad,
        &|| false,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        TranscribeError::Failed {
            error: InferenceError::InvalidInput(_),
            ..
        }
    ));
}
