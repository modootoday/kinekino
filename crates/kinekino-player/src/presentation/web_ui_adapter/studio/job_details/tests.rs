use super::*;
use crate::domain::api_values::DecimalU64;
use kinekino_jobs::domain::{PreparedJobInput, SourceBlobRef};

pub(crate) fn record() -> JobRecord {
    JobRecord::queued(
        format!("job-{}", "a".repeat(32)).try_into().unwrap(),
        PreparedJobInput {
            operation: "subtitle.convert".into(),
            parameters: serde_json::json!({"source":"fixture only"}),
            source: SourceBlobRef {
                sha256: "b".repeat(64),
                bytes: DecimalU64(8),
            },
            execution_timeout_ms: DecimalU64(30000),
        },
    )
    .unwrap()
}
#[test]
fn unrequested_job_does_not_invent_state_progress_or_a_received_snapshot() {
    let view = JobDetailsProjection::new(record().job_id);
    let props = view.props();
    assert_eq!(props["stateLabel"], Value::String("—".into()));
    assert_eq!(props["outputPath"], Value::String("—".into()));
    assert_eq!(
        props["snapshotLabel"],
        Value::String("아직 조회한 작업 기록이 없습니다.".into())
    );
    assert!(view.record().is_none());
}
#[test]
fn exact_typed_record_is_displayed_and_refresh_keeps_explicit_previous_snapshot() {
    let job = record();
    let mut view = JobDetailsProjection::new(job.job_id.clone());
    view.received(job.clone()).unwrap();
    assert_eq!(
        view.props()["revisionLabel"],
        Value::String(job.revision.0.to_string())
    );
    assert_eq!(
        view.props()["sourceSha256"],
        Value::String(job.input.source.sha256.clone())
    );
    view.loading = true;
    assert_eq!(view.record().unwrap().job_id, job.job_id);
    assert_eq!(
        view.props()["snapshotLabel"],
        Value::String("이전 조회값은 최신 상태를 보장하지 않습니다.".into())
    );
}
#[test]
fn foreign_identity_and_older_revision_do_not_replace_last_good_job_record() {
    let old = record();
    let mut newer = old.clone();
    newer.start().unwrap();
    let mut view = JobDetailsProjection::new(old.job_id.clone());
    view.received(newer.clone()).unwrap();
    assert!(view.received(old.clone()).is_err());
    let mut foreign = old;
    foreign.job_id = format!("job-{}", "c".repeat(32)).try_into().unwrap();
    assert!(view.received(foreign).is_err());
    assert_eq!(view.record().unwrap().revision, newer.revision);
    assert_eq!(view.record().unwrap().job_id, newer.job_id);
}
