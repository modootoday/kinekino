//! Real pending-resolution and durable state objects; no executor, browser or OS process.
use super::*;
use crate::application::{
    jobs::{CancelJob, CancelledJob, JobsApi},
    playback_session::transition::TransitionSlot,
};
use kinekino_jobs::domain::{PreparedJobInput, SourceBlobRef};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

struct Jobs {
    record: Mutex<JobRecord>,
    cancels: AtomicUsize,
    fail: std::sync::atomic::AtomicBool,
}
impl JobsApi for Jobs {
    fn get(&self, input: GetJob) -> Result<JobRecord, ApiError> {
        let job = self.record.lock().unwrap();
        assert_eq!(input.job_id, job.job_id);
        Ok(job.clone())
    }
    fn cancel(&self, input: CancelJob) -> Result<CancelledJob, ApiError> {
        self.cancels.fetch_add(1, Ordering::SeqCst);
        if self.fail.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "original cancel failure",
            ));
        }
        let mut job = self.record.lock().unwrap();
        assert_eq!(input.job_id, job.job_id);
        assert_eq!(input.expected_attempt, job.attempt);
        let disposition = job.request_cancel().unwrap();
        Ok(CancelledJob {
            job: job.clone(),
            disposition,
        })
    }
}
fn fixture() -> (Jobs, PendingAdjacentResolution) {
    let target = "https://catalog.example.test/episode/retirement-fixture".to_owned();
    let mut record = JobRecord::queued(
        "job-00000000000000000000000000000001"
            .to_owned()
            .try_into()
            .unwrap(),
        PreparedJobInput {
            operation: "episodes.resolve".into(),
            parameters: serde_json::json!({"episode_url":target}),
            source: SourceBlobRef {
                sha256: "a".repeat(64),
                bytes: DecimalU64(1),
            },
            execution_timeout_ms: DecimalU64(30000),
        },
    )
    .unwrap();
    record.start().unwrap();
    let pending = PendingAdjacentResolution {
        prepared: PreparedAdjacentEpisode::retirement_fixture(target),
        expected_instance_id: DecimalU64(1),
        job_id: record.job_id.clone(),
        attempt: record.attempt,
        paused: false,
        volume_percent: 0.,
    };
    (
        Jobs {
            record: Mutex::new(record),
            cancels: AtomicUsize::new(0),
            fail: std::sync::atomic::AtomicBool::new(false),
        },
        pending,
    )
}
#[test]
fn retirement_cancels_only_original_attempt_and_waits_for_ack_before_fresh_transition() {
    let slot = TransitionSlot::default();
    let (jobs, pending) = fixture();
    let (old, original_job) = slot.begin(|| Ok(pending)).unwrap();
    let ticket = slot.pause_route().unwrap();
    assert!(!slot.poll_route_pause(&ticket, &jobs).unwrap());
    assert!(!slot.poll_route_pause(&ticket, &jobs).unwrap());
    assert_eq!(jobs.cancels.load(Ordering::SeqCst), 1);
    assert_eq!(
        slot.resume_route(&ticket, || panic!("pending owner must block reactivation"))
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        slot.observe(old, &jobs).unwrap_err().code,
        ErrorCode::Cancelled
    );
    jobs.record.lock().unwrap().acknowledge_cancel().unwrap();
    assert!(slot.poll_route_pause(&ticket, &jobs).unwrap());
    assert_eq!(jobs.record.lock().unwrap().job_id, original_job);
    slot.resume_route(&ticket, || Ok(())).unwrap();
    assert_eq!(
        slot.observe(old, &jobs).unwrap_err().code,
        ErrorCode::NotFound
    );
    let (fresh_jobs, fresh) = fixture();
    let (new, _) = slot.begin(|| Ok(fresh)).unwrap();
    assert!(new > old);
    assert_eq!(
        slot.observe(new, &fresh_jobs).unwrap().state,
        JobState::Running
    );
    slot.release(new).unwrap();
    slot.close_admission();
    assert_eq!(
        slot.begin(|| panic!("parent close is permanent"))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
}
#[test]
fn original_cancel_failure_retains_pending_job_and_prevents_rearm() {
    let slot = TransitionSlot::default();
    let (jobs, pending) = fixture();
    slot.begin(|| Ok(pending)).unwrap();
    let ticket = slot.pause_route().unwrap();
    jobs.fail.store(true, Ordering::SeqCst);
    let error = slot.poll_route_pause(&ticket, &jobs).unwrap_err();
    assert_eq!(error.message, "original cancel failure");
    assert_eq!(
        slot.resume_route(&ticket, || panic!(
            "failed cancellation retains pending owner"
        ))
        .unwrap_err()
        .code,
        ErrorCode::NeedsInput
    );
    let retained = slot.end().unwrap().unwrap();
    assert_eq!(retained, jobs.record.lock().unwrap().job_id);
    assert_eq!(
        slot.resume_route(&ticket, || Ok(())).unwrap_err().code,
        ErrorCode::Cancelled
    );
}
