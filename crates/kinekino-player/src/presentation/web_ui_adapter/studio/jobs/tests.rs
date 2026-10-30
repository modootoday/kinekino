use super::{labels, list::*, selection::*};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        jobs::{JobPage, JobRecord},
    },
    domain::api_values::DecimalU64,
};
use kinekino_jobs::domain::{PreparedJobInput, SourceBlobRef};
use std::num::NonZeroU64;

fn route(n: u64) -> NonZeroU64 {
    NonZeroU64::new(n).unwrap()
}
fn record(key: char) -> JobRecord {
    JobRecord::queued(
        format!("job-{}", key.to_string().repeat(32))
            .try_into()
            .unwrap(),
        PreparedJobInput {
            operation: "subtitle.convert".into(),
            parameters: serde_json::json!({"fixture":"list display only"}),
            source: SourceBlobRef {
                sha256: "c".repeat(64),
                bytes: DecimalU64(12),
            },
            execution_timeout_ms: DecimalU64(30000),
        },
    )
    .unwrap()
}
fn page(rows: Vec<JobRecord>) -> JobPage {
    JobPage {
        jobs: rows,
        next_cursor: None,
    }
}
fn loaded() -> JobsListProjection {
    let mut list = JobsListProjection::new(route(1));
    let stamp = list.begin().unwrap();
    list.accept(stamp, &page(vec![record('a'), record('b')]))
        .unwrap();
    list
}

#[test]
fn not_requested_empty_and_unsupported_remain_distinct_without_implicit_selection() {
    let mut list = JobsListProjection::new(route(1));
    let selection = JobsSelection::new(route(1));
    assert_eq!(list.status(), labels::NOT_REQUESTED);
    assert!(list.page().is_none());
    assert!(selection.active().is_none());
    let stamp = list.begin().unwrap();
    list.accept(stamp, &page(vec![])).unwrap();
    assert_eq!(list.status(), labels::EMPTY);
    let stamp = list.begin().unwrap();
    list.fail(
        stamp,
        &ApiError::new(ErrorCode::Unsupported, "no jobs host"),
    )
    .unwrap();
    assert_eq!(list.status(), labels::FAILED);
    assert!(list.diagnostic().contains("Unsupported"));
    assert!(list.previous_snapshot());
    assert!(selection.active().is_none());
}
#[test]
fn late_list_reply_cannot_clear_the_current_request_or_replace_last_good_rows() {
    let mut list = JobsListProjection::new(route(1));
    let old = list.begin().unwrap();
    list.accept(old, &page(vec![record('a')])).unwrap();
    let current = list.begin().unwrap();
    assert_eq!(list.begin(), Err(ListPresentationError::Busy));
    assert_eq!(
        list.accept(old, &page(vec![record('b')])),
        Err(ListPresentationError::Stale)
    );
    assert_eq!(list.status(), labels::LOADING);
    assert!(list.contains(&record('a').job_id));
    list.accept(current, &page(vec![record('b')])).unwrap();
    assert!(list.contains(&record('b').job_id));
    assert!(!list.previous_snapshot());
}
#[test]
fn duplicate_or_invalid_records_preserve_original_page_and_pending_result() {
    let mut list = loaded();
    let stamp = list.begin().unwrap();
    assert_eq!(
        list.accept(stamp, &page(vec![record('a'), record('a')])),
        Err(ListPresentationError::DuplicateIdentity)
    );
    let mut invalid = record('c');
    invalid.revision = DecimalU64(0);
    assert!(matches!(
        list.accept(stamp, &page(vec![invalid])),
        Err(ListPresentationError::InvalidRecord(_))
    ));
    assert_eq!(list.page().unwrap().jobs.len(), 2);
    assert_eq!(list.status(), labels::LOADING);
}
#[test]
fn explicit_selection_requires_a_known_row_and_commits_only_after_host_handoff() {
    let list = loaded();
    let mut selection = JobsSelection::new(route(1));
    assert_eq!(
        selection.request(&list, Some(record('c').job_id)),
        Err(SelectionError::UnknownRow)
    );
    let first = selection.request(&list, Some(record('a').job_id)).unwrap();
    assert!(selection.active().is_none());
    assert!(!selection.accepts_result(&first));
    selection.commit(&first).unwrap();
    assert!(selection.accepts_result(&first));
    let next = selection.request(&list, Some(record('b').job_id)).unwrap();
    assert_eq!(selection.active().unwrap().job(), first.job());
    assert!(!selection.accepts_result(&first));
    selection.commit(&next).unwrap();
    assert!(!selection.accepts_result(&first));
    assert!(selection.accepts_result(&next));
}
#[test]
fn failed_handoff_keeps_last_good_selection_but_never_revives_old_results() {
    let list = loaded();
    let mut selection = JobsSelection::new(route(1));
    let first = selection.request(&list, Some(record('a').job_id)).unwrap();
    selection.commit(&first).unwrap();
    let next = selection.request(&list, Some(record('b').job_id)).unwrap();
    selection.abort(&next).unwrap();
    assert_eq!(selection.active(), Some(&first));
    assert!(!selection.accepts_result(&first));
    assert_eq!(selection.commit(&next), Err(SelectionError::Stale));
    let retry = selection.request(&list, Some(record('a').job_id)).unwrap();
    assert!(retry.generation() > next.generation());
    selection.commit(&retry).unwrap();
    assert!(selection.accepts_result(&retry));
}
#[test]
fn foreign_route_or_close_rejects_selection_and_late_list_results() {
    let mut list = loaded();
    let mut foreign = JobsSelection::new(route(2));
    assert_eq!(
        foreign.request(&list, Some(record('a').job_id)),
        Err(SelectionError::ForeignRoute)
    );
    let stamp = list.begin().unwrap();
    list.begin_close();
    assert_eq!(
        list.accept(stamp, &page(vec![])),
        Err(ListPresentationError::Stale)
    );
    assert_eq!(list.begin(), Err(ListPresentationError::Closing));
    assert!(list.contains(&record('a').job_id));
    foreign.begin_close();
    assert_eq!(foreign.request(&list, None), Err(SelectionError::Closing));
}
