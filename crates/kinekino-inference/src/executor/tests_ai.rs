//! Translate, OCR and diarize executor tests (P5).
use super::tests::*;
use super::*;
use crate::mock::MockBackend;
use crate::port::InferenceKind;
use crate::settings::Budget;
use kinekino_core::port::Adapter;

fn cues() -> Vec<Chunk> {
    vec![
        Chunk {
            start: 0.0,
            end: 1.0,
            text: "hello".into(),
        },
        Chunk {
            start: 1.0,
            end: 2.0,
            text: "  ".into(),
        },
        Chunk {
            start: 2.0,
            end: 3.0,
            text: "bye".into(),
        },
    ]
}

#[test]
fn translate_keeps_timing_skips_blank_and_records_cost() {
    let mut b = Priced(MockBackend::default(), 0.01, InferenceKind::HfCloud);
    let mut guard = CostGuard::new(Budget::default());
    let t = translate(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        "ko",
        &cues(),
        &|| false,
    )
    .unwrap();
    assert_eq!(t.cues.len(), 3);
    assert_eq!(
        t.cues[0],
        Chunk {
            start: 0.0,
            end: 1.0,
            text: "ok".into()
        }
    );
    assert_eq!(t.cues[1].text, "  ");
    assert_eq!(t.spend.provenance.len(), 2);
    assert!((guard.total() - 0.02).abs() < 1e-9);
    assert!(translation_prompt("ko", "x").contains("<cue>x</cue>"));
}

#[test]
fn executors_require_consent_except_local_pack() {
    let mut cloud = Priced(MockBackend::default(), 0.0, InferenceKind::HfCloud);
    let mut guard = CostGuard::new(Budget::default());
    let none = Consent::default();
    let e = translate(
        &mut cloud,
        &mut guard,
        &none,
        &job(true),
        "ko",
        &cues(),
        &|| false,
    )
    .unwrap_err();
    assert!(
        matches!(
            e,
            StepError::Failed {
                index: 0,
                error: InferenceError::ConsentRequired(_),
                ..
            }
        ),
        "{e:?}"
    );
    let frames = [Frame {
        time: 0.0,
        png: vec![1],
    }];
    assert!(
        ocr_frames(
            &mut cloud,
            &mut guard,
            &none,
            &job(true),
            &frames,
            1.0,
            &|| false
        )
        .is_err()
    );
    assert!(
        diarize(
            &mut cloud,
            &mut guard,
            &none,
            &job(true),
            &track(6),
            &regions(),
            &|| false
        )
        .is_err()
    );
    let mut local = Priced(MockBackend::default(), 0.0, InferenceKind::LocalPack);
    assert!(
        translate(
            &mut local,
            &mut guard,
            &none,
            &job(true),
            "ko",
            &cues(),
            &|| false
        )
        .is_ok()
    );
    assert!(
        diarize(
            &mut local,
            &mut guard,
            &none,
            &job(true),
            &track(6),
            &regions(),
            &|| false
        )
        .is_ok()
    );
}

#[test]
fn executors_need_confirm_then_budget_refusal_keeps_partial() {
    let budget = Budget {
        total_usd: 1.0,
        per_call_usd: 0.5,
        monthly_usd: 10.0,
        confirm_ratio: 0.8,
    };
    let mut b = Priced(MockBackend::default(), 0.45, InferenceKind::HfCloud);
    let mut guard = CostGuard::new(budget.clone());
    let e = translate(
        &mut b,
        &mut guard,
        &granted(),
        &job(false),
        "ko",
        &cues(),
        &|| false,
    )
    .unwrap_err();
    assert!(
        matches!(e, StepError::NeedsConfirm { index: 2, .. }),
        "{e:?}"
    );
    let mut guard = CostGuard::new(budget);
    guard.record(LedgerEntry {
        month: "2026-10".into(),
        backend: "x".into(),
        model: "m".into(),
        cost_usd: 0.2,
    });
    let e = translate(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        "ko",
        &cues(),
        &|| false,
    )
    .unwrap_err();
    let StepError::Failed {
        index,
        error,
        partial,
    } = e
    else {
        panic!()
    };
    assert_eq!(index, 2);
    assert!(matches!(error, InferenceError::Budget(_)));
    assert_eq!(partial.cues.len(), 2);
}

#[test]
fn executors_cancel_and_reject_bad_input() {
    let mut b = MockBackend::default();
    let mut guard = CostGuard::new(Budget::default());
    let e = translate(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        "ko",
        &cues(),
        &|| true,
    )
    .unwrap_err();
    assert!(matches!(
        e,
        StepError::Failed {
            error: InferenceError::Cancelled,
            ..
        }
    ));
    let e = diarize(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        &track(6),
        &regions(),
        &|| true,
    )
    .unwrap_err();
    assert!(matches!(
        e,
        StepError::Failed {
            index: 0,
            error: InferenceError::Cancelled,
            ..
        }
    ));
    assert!(
        translate(
            &mut b,
            &mut guard,
            &granted(),
            &job(true),
            " ",
            &cues(),
            &|| false
        )
        .is_err()
    );
    let back = [
        Frame {
            time: 1.0,
            png: vec![],
        },
        Frame {
            time: 0.5,
            png: vec![],
        },
    ];
    let e = ocr_frames(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        &back,
        1.0,
        &|| false,
    )
    .unwrap_err();
    assert!(matches!(
        e,
        StepError::Failed {
            error: InferenceError::InvalidInput(_),
            ..
        }
    ));
    assert!(
        ocr_frames(
            &mut b,
            &mut guard,
            &granted(),
            &job(true),
            &[],
            0.0,
            &|| false
        )
        .is_err()
    );
}

/// Returns a canned output for every call, regardless of task.
struct Canned(Vec<crate::port::Output>);

impl Adapter for Canned {
    type Kind = InferenceKind;
    type Caps = crate::port::InferenceCaps;
    fn kind(&self) -> InferenceKind {
        InferenceKind::Mock
    }
    fn capabilities(&self) -> Self::Caps {
        crate::port::InferenceCaps::all()
    }
}

impl InferenceBackend for Canned {
    fn estimate_usd(&self, _r: &InferenceRequest) -> Option<f64> {
        Some(0.0)
    }
    fn run(
        &mut self,
        r: &InferenceRequest,
        _c: &dyn Fn() -> bool,
    ) -> Result<crate::InferenceResponse, InferenceError> {
        let output = self.0.remove(0);
        Ok(crate::InferenceResponse {
            output,
            provenance: Provenance {
                backend: "canned".into(),
                model: r.model().into(),
                cost_estimate_usd: 0.0,
                provider: None,
                fallback_from: None,
            },
        })
    }
}

#[test]
fn ocr_merges_equal_consecutive_texts_and_breaks_on_empty() {
    use crate::port::Output::Ocr;
    let texts = ["A", " A ", "", "A", "B"];
    let mut b = Canned(texts.iter().map(|t| Ocr { text: (*t).into() }).collect());
    let frames: Vec<Frame> = (0..5)
        .map(|i| Frame {
            time: i as f64,
            png: vec![i],
        })
        .collect();
    let mut guard = CostGuard::new(Budget::default());
    let o = ocr_frames(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        &frames,
        0.5,
        &|| false,
    )
    .unwrap();
    let got: Vec<_> = o
        .spans
        .iter()
        .map(|c| (c.start, c.end, c.text.as_str()))
        .collect();
    assert_eq!(got, vec![(0.0, 1.5, "A"), (3.0, 3.5, "A"), (4.0, 4.5, "B")]);
}

#[test]
fn protocol_mismatch_is_failure_not_panic() {
    let mut b = Canned(vec![crate::port::Output::Ocr { text: "x".into() }]);
    let mut guard = CostGuard::new(Budget::default());
    let e = translate(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        "ko",
        &cues(),
        &|| false,
    )
    .unwrap_err();
    assert!(
        matches!(
            e,
            StepError::Failed {
                index: 0,
                error: InferenceError::Protocol(_),
                ..
            }
        ),
        "{e:?}"
    );
    assert_eq!(guard.total(), 0.0);
}

#[test]
fn diarize_offsets_clamps_and_assigns_by_overlap() {
    use crate::port::Output::Diarize;
    let mut b = Canned(vec![
        Diarize {
            segments: vec![(0.0, 0.4, "A".into()), (0.4, 9.0, "B".into())],
        },
        Diarize {
            segments: vec![(0.0, 0.6, "A".into()), (0.7, 0.9, "C".into())],
        },
    ]);
    let mut guard = CostGuard::new(Budget::default());
    let d = diarize(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        &track(6),
        &regions(),
        &|| false,
    )
    .unwrap();
    let got: Vec<_> = d
        .segments
        .iter()
        .map(|s| (s.start, s.end, s.speaker.as_str()))
        .collect();
    assert_eq!(got.len(), 3, "{got:?}");
    assert_eq!(got[1], (1.4, 2.0, "B"));
    assert!((got[2].1 - 5.6).abs() < 1e-9);
    let chunks = vec![
        Chunk {
            start: 1.0,
            end: 1.9,
            text: "x".into(),
        },
        Chunk {
            start: 3.0,
            end: 4.0,
            text: "y".into(),
        },
    ];
    assert_eq!(
        assign_speakers(&chunks, &d.segments),
        vec![Some("B".into()), None]
    );
}
