use super::*;
use kinekino_media_contracts::api_values::DecimalU64;
use serde_json::json;
fn record() -> JobRecord {
    JobRecord::queued(
        "job-00000000000000000000000000000001"
            .to_owned()
            .try_into()
            .unwrap(),
        PreparedJobInput {
            operation: "subtitles.convert".into(),
            parameters: json!({"input_format":"srt"}),
            source: SourceBlobRef {
                sha256: "a".repeat(64),
                bytes: DecimalU64(10),
            },
            execution_timeout_ms: DecimalU64(30000),
        },
    )
    .unwrap()
}
#[test]
fn cancellation_request_is_not_acknowledgement_and_prevents_publication() {
    let mut job = record();
    job.start().unwrap();
    assert_eq!(job.request_cancel().unwrap(), CancelDisposition::Requested);
    assert_eq!(job.state, JobState::Cancelling);
    assert!(job.begin_publication().is_err());
    job.acknowledge_cancel().unwrap();
    assert_eq!(job.state, JobState::Cancelled);
    let input_sha = job.input_sha256.clone();
    job.retry().unwrap();
    assert_eq!(job.attempt.0, 2);
    assert_eq!(job.input_sha256, input_sha);
    job.validate().unwrap();
}
#[test]
fn publication_wins_cancellation_and_keeps_verified_receipt() {
    let mut job = record();
    job.start().unwrap();
    job.begin_publication().unwrap();
    assert_eq!(
        job.request_cancel().unwrap(),
        CancelDisposition::PublicationInProgress
    );
    assert!(job.acknowledge_cancel().is_err());
    job.begin_verification().unwrap();
    job.complete(VerifiedJobOutput {
        path: "output.vtt".into(),
        sha256: "b".repeat(64),
        bytes: DecimalU64(20),
        receipt: json!({"cue_count":1}),
    })
    .unwrap();
    assert_eq!(
        job.request_cancel().unwrap(),
        CancelDisposition::AlreadyTerminal
    );
    assert!(job.output.is_some());
    assert!(job.retry().is_err());
    job.validate().unwrap();
}
#[test]
fn restart_interrupts_without_automatic_retry_and_blocks_ambiguous_publication_retry() {
    let mut job = record();
    job.start().unwrap();
    job.begin_publication().unwrap();
    job.interrupt().unwrap();
    assert_eq!(job.state, JobState::Interrupted);
    assert_eq!(job.attempt.0, 1);
    assert!(job.recovery_required());
    assert!(job.retry().is_err());
    let mut queued = record();
    queued.interrupt().unwrap();
    assert_eq!(queued.state, JobState::Interrupted);
    queued.retry().unwrap();
    assert_eq!(queued.attempt.0, 2);
    queued.validate().unwrap();
}
