//! OCR port mock contract (M4-8): success, backend failure, cancellation, refusal and input
//! checks of `ocr_frames` against the in-process mock. No model or network call is made.
use kinekino_core::port::Adapter;
use kinekino_inference::executor::{Frame, OcrSpans, StepError, TranscribeJob, ocr_frames};
use kinekino_inference::mock::MockBackend;
use kinekino_inference::settings::Budget;
use kinekino_inference::{
    Consent, ConsentScope, CostGuard, InferenceBackend, InferenceCaps, InferenceError,
    InferenceKind, InferenceRequest, InferenceResponse, Task,
};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

const MODEL: &str = "mock/ocr";

/// Mock wrapper that records every request, prices each call and can fail one frame.
struct Recording {
    inner: MockBackend,
    kind: InferenceKind,
    cost: f64,
    fail_at: HashMap<usize, InferenceError>,
    wrong_task_at: Option<usize>,
    calls: Rc<Cell<usize>>,
    /// Calls that returned a response (the cancel fixture watches this).
    done: Rc<Cell<usize>>,
    seen: Vec<InferenceRequest>,
}

impl Recording {
    fn new(inner: MockBackend) -> Self {
        Self {
            inner,
            kind: InferenceKind::Mock,
            cost: 0.01,
            fail_at: HashMap::new(),
            wrong_task_at: None,
            calls: Rc::new(Cell::new(0)),
            done: Rc::new(Cell::new(0)),
            seen: Vec::new(),
        }
    }
}

impl Adapter for Recording {
    type Kind = InferenceKind;
    type Caps = InferenceCaps;
    fn kind(&self) -> InferenceKind {
        self.kind
    }
    fn capabilities(&self) -> InferenceCaps {
        self.inner.capabilities()
    }
}

impl InferenceBackend for Recording {
    fn estimate_usd(&self, _request: &InferenceRequest) -> Option<f64> {
        Some(self.cost)
    }
    fn run(
        &mut self,
        request: &InferenceRequest,
        cancel: &dyn Fn() -> bool,
    ) -> Result<InferenceResponse, InferenceError> {
        let index = self.calls.get();
        self.calls.set(index + 1);
        self.seen.push(request.clone());
        if let Some(error) = self.fail_at.get(&index) {
            return Err(error.clone());
        }
        let mut response = self.inner.run(request, cancel)?;
        if self.wrong_task_at == Some(index) {
            response.output = kinekino_inference::port::Output::Generate { text: "x".into() };
        }
        response.provenance.cost_estimate_usd = self.cost;
        self.done.set(self.done.get() + 1);
        Ok(response)
    }
}

fn job(confirmed: bool) -> TranscribeJob<'static> {
    TranscribeJob {
        project: "p",
        job: "j",
        month: "2026-11",
        model: MODEL,
        confirmed,
    }
}

fn granted() -> Consent {
    let mut consent = Consent::default();
    consent.grant(ConsentScope::Project, "p", "j");
    consent
}

fn frames(n: u8) -> Vec<Frame> {
    (0..n)
        .map(|i| Frame {
            time: f64::from(i),
            png: vec![0x89, b'P', i],
        })
        .collect()
}

fn ledger_len(guard: &CostGuard) -> usize {
    guard.to_jsonl().lines().count()
}

fn run(
    b: &mut Recording,
    guard: &mut CostGuard,
    input: &[Frame],
    secs: f64,
) -> Result<OcrSpans, StepError<OcrSpans>> {
    ocr_frames(b, guard, &granted(), &job(true), input, secs, &|| false)
}

#[test]
fn mock_success_sends_each_frame_once_and_records_spend() {
    let mut b = Recording::new(MockBackend::default());
    let mut guard = CostGuard::new(Budget::default());
    let input = frames(3);
    let out = run(&mut b, &mut guard, &input, 1.0).unwrap();
    assert_eq!(out.spans.len(), 1, "equal mock texts merge into one span");
    assert_eq!(
        (
            out.spans[0].start,
            out.spans[0].end,
            out.spans[0].text.as_str()
        ),
        (0.0, 3.0, "TITLE")
    );
    assert_eq!(out.spend.provenance.len(), 3);
    assert!(
        out.spend
            .provenance
            .iter()
            .all(|p| p.backend == "mock" && p.model == MODEL)
    );
    assert!((out.spend.cost_usd - 0.03).abs() < 1e-9);
    assert_eq!(ledger_len(&guard), 3);
    let sent: Vec<_> = b
        .seen
        .iter()
        .map(|r| match r {
            InferenceRequest::Ocr { model, image } => (model.as_str(), image.clone()),
            other => panic!("unexpected request {other:?}"),
        })
        .collect();
    let expected: Vec<_> = input.iter().map(|f| (MODEL, f.png.clone())).collect();
    assert_eq!(
        sent, expected,
        "only the sampled frame bytes leave the executor"
    );
}

#[test]
fn mock_success_without_frames_makes_no_call() {
    let mut b = Recording::new(MockBackend::default());
    let mut guard = CostGuard::new(Budget::default());
    let out = run(&mut b, &mut guard, &[], 1.0).unwrap();
    assert_eq!(out, OcrSpans::default());
    assert_eq!((b.calls.get(), ledger_len(&guard)), (0, 0));
}

#[test]
fn backend_failure_stops_at_frame_and_keeps_partial_without_charging_it() {
    for error in [
        InferenceError::Unavailable("offline".into()),
        InferenceError::RateLimited("429".into()),
        InferenceError::Timeout("slow".into()),
        InferenceError::Auth("401".into()),
        InferenceError::Budget("402".into()),
    ] {
        let mut b = Recording::new(MockBackend::default());
        b.fail_at.insert(2, error.clone());
        let mut guard = CostGuard::new(Budget::default());
        let e = run(&mut b, &mut guard, &frames(4), 1.0).unwrap_err();
        let StepError::Failed {
            index,
            error: got,
            partial,
        } = e
        else {
            panic!("expected failure for {error:?}")
        };
        assert_eq!((index, got), (2, error));
        assert_eq!(partial.spans.len(), 1);
        assert_eq!((partial.spans[0].start, partial.spans[0].end), (0.0, 2.0));
        assert_eq!(partial.spend.provenance.len(), 2);
        assert_eq!(b.calls.get(), 3, "no call after the failing frame");
        assert_eq!(ledger_len(&guard), 2, "the failed call is not charged");
    }
}

#[test]
fn mock_forced_failure_and_missing_ocr_capability_fail_first_frame() {
    let mut forced = MockBackend::default();
    forced.fail_with = Some(InferenceError::Unavailable("down".into()));
    let mut b = Recording::new(forced);
    let mut guard = CostGuard::new(Budget::default());
    let e = run(&mut b, &mut guard, &frames(2), 1.0).unwrap_err();
    assert!(
        matches!(&e, StepError::Failed { index: 0, error: InferenceError::Unavailable(_), partial } if partial.spans.is_empty()),
        "{e:?}"
    );
    let mut b = Recording::new(MockBackend::with_caps(InferenceCaps::of(&[Task::Generate])));
    let e = run(&mut b, &mut guard, &frames(2), 1.0).unwrap_err();
    assert!(
        matches!(
            &e,
            StepError::Failed {
                index: 0,
                error: InferenceError::Unsupported(_),
                ..
            }
        ),
        "{e:?}"
    );
    assert_eq!(ledger_len(&guard), 0);
}

#[test]
fn wrong_output_task_is_protocol_failure_and_not_charged() {
    let mut b = Recording::new(MockBackend::default());
    b.wrong_task_at = Some(1);
    let mut guard = CostGuard::new(Budget::default());
    let e = run(&mut b, &mut guard, &frames(3), 1.0).unwrap_err();
    let StepError::Failed {
        index,
        error,
        partial,
    } = e
    else {
        panic!()
    };
    assert_eq!(index, 1);
    assert!(matches!(error, InferenceError::Protocol(_)), "{error:?}");
    assert_eq!(partial.spans.len(), 1);
    assert_eq!(ledger_len(&guard), 1);
    assert_eq!(b.calls.get(), 2);
}

#[test]
fn cancel_before_start_and_between_frames_keeps_partial() {
    let mut b = Recording::new(MockBackend::default());
    let mut guard = CostGuard::new(Budget::default());
    let e = ocr_frames(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        &frames(3),
        1.0,
        &|| true,
    )
    .unwrap_err();
    assert!(
        matches!(&e, StepError::Failed { index: 0, error: InferenceError::Cancelled, partial } if partial.spans.is_empty()),
        "{e:?}"
    );
    assert_eq!(b.calls.get(), 0);

    let mut b = Recording::new(MockBackend::default());
    let done = Rc::clone(&b.done);
    let cancel = move || done.get() >= 2;
    let e = ocr_frames(
        &mut b,
        &mut guard,
        &granted(),
        &job(true),
        &frames(4),
        1.0,
        &cancel,
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
    assert_eq!((index, error), (2, InferenceError::Cancelled));
    assert_eq!(partial.spans.len(), 1);
    assert_eq!(partial.spans[0].end, 2.0);
    assert_eq!(b.calls.get(), 2, "the cancelled frame is never sent");
    assert_eq!(ledger_len(&guard), 2);
}

#[test]
fn consent_and_confirm_gate_frames_before_any_upload() {
    let mut b = Recording::new(MockBackend::default());
    let mut guard = CostGuard::new(Budget::default());
    let e = ocr_frames(
        &mut b,
        &mut guard,
        &Consent::default(),
        &job(true),
        &frames(2),
        1.0,
        &|| false,
    )
    .unwrap_err();
    assert!(
        matches!(
            &e,
            StepError::Failed {
                index: 0,
                error: InferenceError::ConsentRequired(_),
                ..
            }
        ),
        "{e:?}"
    );
    assert_eq!(b.calls.get(), 0);

    let mut local = Recording::new(MockBackend::default());
    local.kind = InferenceKind::LocalPack;
    let out = ocr_frames(
        &mut local,
        &mut guard,
        &Consent::default(),
        &job(true),
        &frames(1),
        1.0,
        &|| false,
    )
    .unwrap();
    assert_eq!(
        out.spans.len(),
        1,
        "local packs never upload, so no consent is needed"
    );

    let mut near = Recording::new(MockBackend::default());
    near.cost = 0.45;
    let mut guard = CostGuard::new(Budget {
        total_usd: 1.0,
        ..Budget::default()
    });
    let e = ocr_frames(
        &mut near,
        &mut guard,
        &granted(),
        &job(false),
        &frames(3),
        1.0,
        &|| false,
    )
    .unwrap_err();
    assert!(
        matches!(e, StepError::NeedsConfirm { index: 1, .. }),
        "{e:?}"
    );
    assert_eq!(
        near.calls.get(),
        1,
        "nothing is sent for the frame awaiting confirmation"
    );
    assert_eq!(ledger_len(&guard), 1);
}

#[test]
fn invalid_frame_times_and_interval_are_refused_before_any_call() {
    let bad_secs = [0.0, -1.0, f64::NAN, f64::INFINITY];
    for secs in bad_secs {
        let mut b = Recording::new(MockBackend::default());
        let mut guard = CostGuard::new(Budget::default());
        let e = run(&mut b, &mut guard, &frames(2), secs).unwrap_err();
        assert!(
            matches!(
                &e,
                StepError::Failed {
                    index: 0,
                    error: InferenceError::InvalidInput(_),
                    ..
                }
            ),
            "interval {secs}: {e:?}"
        );
        assert_eq!(b.calls.get(), 0, "interval {secs}");
    }
    let bad_times: [&[f64]; 6] = [
        &[1.0, 0.5],
        &[1.0, 1.0],
        &[f64::NAN],
        &[0.0, f64::NAN],
        &[-0.5, 1.0],
        &[0.0, f64::INFINITY],
    ];
    for times in bad_times {
        let input: Vec<Frame> = times
            .iter()
            .map(|&time| Frame { time, png: vec![1] })
            .collect();
        let mut b = Recording::new(MockBackend::default());
        let mut guard = CostGuard::new(Budget::default());
        let e = run(&mut b, &mut guard, &input, 1.0).unwrap_err();
        assert!(
            matches!(
                &e,
                StepError::Failed {
                    index: 0,
                    error: InferenceError::InvalidInput(_),
                    ..
                }
            ),
            "times {times:?}: {e:?}"
        );
        assert_eq!(b.calls.get(), 0, "times {times:?}");
    }
}
