use super::*;
use crate::domain::{api_values::DecimalU64, review_source::*};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(super) struct Permit;
impl PublicationGuard for Permit {
    fn before_publish(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Cancel;
impl PublicationGuard for Cancel {
    fn before_publish(&self) -> Result<(), ApiError> {
        Err(ApiError::new(ErrorCode::Cancelled, "test cancellation"))
    }
}
pub(super) fn root() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "review-source-files-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    p
}
pub(super) fn body() -> TransactionBody {
    let case = ReviewCase {
        package_sha256: "a".repeat(64),
        item_id: "item".into(),
        source_request_sha256: "b".repeat(64),
    };
    let a = AssociationRevision {
        association_id: "binding".into(),
        case: case.clone(),
        media: MediaReference {
            asset_id: "media".into(),
            sha256: "c".repeat(64),
        },
        revision: DecimalU64(1),
        timeline_status: TimelineStatus::Unverified,
        evidence_refs: vec![],
        note: "source note".into(),
    };
    let d = RangeDraft {
        draft_id: "draft".into(),
        association_id: a.association_id.clone(),
        association_revision: a.revision,
        draft_revision: DecimalU64(1),
        authored_by: DraftAuthor::User,
        ranges: vec![UserRange {
            start_us: DecimalU64(1),
            end_us: DecimalU64(3),
        }],
        note: "user note".into(),
    };
    TransactionBody {
        schema_version: "1".into(),
        request: MutationRequest {
            case,
            idempotency_key: "first".into(),
            expected_revision: None,
            mutation: Mutation::SaveDraft {
                draft: Box::new(d.clone()),
                create_association: Some(Box::new(a.clone())),
            },
        },
        association: a,
        draft: Some(d),
    }
}
#[test]
fn full_case_capacity_preserves_committed_replay_and_rejects_new_publication() {
    let root = root();
    let store = UserReviewFiles::open(&root, true).unwrap();
    let original = body();
    drop(
        store
            .open(&original.request.case, StoreAccess::Write)
            .unwrap(),
    );
    let directory = directory(&root);
    let mut first = None;
    for index in 0..MAX_CASE_TRANSACTIONS {
        let mut record = original.clone();
        record.association.association_id = format!("capacity-{index}");
        record.request.idempotency_key = format!("capacity-key-{index}");
        record.request.mutation = Mutation::CreateAssociation(Box::new(record.association.clone()));
        record.draft = None;
        record.validate().unwrap();
        let bytes = serde_json::to_vec(&record).unwrap();
        let hash = content_digest::sha256(&mut Cursor::new(&bytes), MAX_TRANSACTION_BYTES)
            .unwrap()
            .1;
        fs::write(directory.join(format!("{hash}.json")), &bytes).unwrap();
        if index == 0 {
            first = Some((record, hash));
        }
    }
    let (first, hash) = first.unwrap();
    let mut transaction = store
        .open(&original.request.case, StoreAccess::Write)
        .unwrap();
    let replay = transaction.replay(&first.request).unwrap().unwrap();
    assert_eq!(replay.envelope_sha256, hash);
    assert_eq!(transaction.publish(&first, &Permit).unwrap(), replay);
    assert!(
        matches!(transaction.publish(&original,&Permit),Err(e) if e.code==ErrorCode::ResourceExhausted)
    );
    drop(transaction);
    assert_eq!(json_count(&root), MAX_CASE_TRANSACTIONS);
    assert_eq!(
        store
            .open(&original.request.case, StoreAccess::Read)
            .unwrap()
            .replay(&first.request)
            .unwrap(),
        Some(replay)
    );
}
fn directory(root: &Path) -> PathBuf {
    fs::read_dir(root.join("cases"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("transactions")
}
fn json_count(root: &Path) -> usize {
    fs::read_dir(directory(root))
        .unwrap()
        .filter(|p| {
            p.as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|s| s == "json")
        })
        .count()
}

#[test]
fn complete_envelope_reopens_readonly_and_preserves_exact_replay_without_a_side_index() {
    let path = root();
    let store = UserReviewFiles::open(&path, true).unwrap();
    let body = body();
    let mut tx = UserReviewStore::open(&store, &body.request.case, StoreAccess::Write).unwrap();
    let receipt = tx.publish(&body, &Permit).unwrap();
    assert_eq!(json_count(&path), 1);
    drop(tx);
    drop(store);
    let store = UserReviewFiles::open(&path, false).unwrap();
    let mut tx = UserReviewStore::open(&store, &body.request.case, StoreAccess::Read).unwrap();
    assert_eq!(tx.replay(&body.request).unwrap(), Some(receipt));
    assert_eq!(tx.draft("draft", None).unwrap(), body.draft);
    assert_eq!(
        tx.publish(&body, &Permit).unwrap_err().code,
        ErrorCode::Unsupported
    );
    drop(tx);
    assert_eq!(
        UserReviewStore::open(&store, &body.request.case, StoreAccess::Write)
            .err()
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(json_count(&path), 1);
}
#[test]
fn actual_windows_reservation_excludes_writers_and_recovers_after_release() {
    let path = root();
    let a = UserReviewFiles::open(&path, true).unwrap();
    let b = UserReviewFiles::open(&path, true).unwrap();
    let case = body().request.case;
    let first = UserReviewStore::open(&a, &case, StoreAccess::Write).unwrap();
    assert_eq!(
        UserReviewStore::open(&b, &case, StoreAccess::Write)
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        UserReviewStore::open(&b, &case, StoreAccess::Read)
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    drop(first);
    let read_a = UserReviewStore::open(&a, &case, StoreAccess::Read).unwrap();
    let read_b = UserReviewStore::open(&b, &case, StoreAccess::Read).unwrap();
    assert_eq!(
        UserReviewStore::open(&b, &case, StoreAccess::Write)
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    drop(read_a);
    drop(read_b);
    drop(UserReviewStore::open(&b, &case, StoreAccess::Write).unwrap());
}
#[test]
fn late_cancel_never_publishes_and_ack_loss_is_recovered_by_reopen_same_key() {
    let path = root();
    let store = UserReviewFiles::open(&path, true).unwrap();
    let body = body();
    let mut tx = UserReviewStore::open(&store, &body.request.case, StoreAccess::Write).unwrap();
    assert_eq!(
        tx.publish(&body, &Cancel).unwrap_err().code,
        ErrorCode::Cancelled
    );
    assert_eq!(json_count(&path), 0);
    store
        .root
        .lose_ack
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let error = tx.publish(&body, &Permit).unwrap_err();
    assert_eq!(error.details["delivery"], "execution_unknown");
    assert_eq!(json_count(&path), 1);
    drop(tx);
    drop(store);
    let reopened = UserReviewFiles::open(&path, true).unwrap();
    let mut tx = UserReviewStore::open(&reopened, &body.request.case, StoreAccess::Write).unwrap();
    let receipt = tx.replay(&body.request).unwrap().unwrap();
    assert_eq!(receipt.body, body);
    assert_eq!(tx.publish(&body, &Cancel).unwrap(), receipt);
    assert_eq!(json_count(&path), 1);
}
#[test]
fn case_writer_enforces_cas_and_changed_payload_key_conflicts_on_real_files() {
    let path = root();
    let store = UserReviewFiles::open(&path, true).unwrap();
    let first = body();
    let mut tx = UserReviewStore::open(&store, &first.request.case, StoreAccess::Write).unwrap();
    tx.publish(&first, &Permit).unwrap();
    let mut changed = first.clone();
    if let Mutation::SaveDraft { draft, .. } = &mut changed.request.mutation {
        draft.note.push('!');
    }
    changed.draft.as_mut().unwrap().note.push('!');
    assert_eq!(
        tx.publish(&changed, &Permit).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let mut update = first.clone();
    update.request.idempotency_key = "update".into();
    update.request.expected_revision = Some(DecimalU64(1));
    update.draft.as_mut().unwrap().draft_revision = DecimalU64(2);
    update.request.mutation = Mutation::SaveDraft {
        draft: Box::new(update.draft.clone().unwrap()),
        create_association: None,
    };
    tx.publish(&update, &Permit).unwrap();
    update.request.idempotency_key = "stale".into();
    assert_eq!(
        tx.publish(&update, &Permit).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(json_count(&path), 2);
    drop(tx);
    drop(store);
    let reopened = UserReviewFiles::open(&path, false).unwrap();
    let tx = UserReviewStore::open(&reopened, &first.request.case, StoreAccess::Read).unwrap();
    assert_eq!(tx.draft("draft", Some(DecimalU64(1))).unwrap(), first.draft);
    assert_eq!(
        tx.draft("draft", None).unwrap().unwrap().draft_revision,
        DecimalU64(2)
    );
}
#[test]
fn crashed_temporary_alias_of_committed_bytes_is_retained_but_orphan_partial_fails_closed() {
    let path = root();
    let store = UserReviewFiles::open(&path, true).unwrap();
    let body = body();
    let mut tx = UserReviewStore::open(&store, &body.request.case, StoreAccess::Write).unwrap();
    let receipt = tx.publish(&body, &Permit).unwrap();
    drop(tx);
    let directory = directory(&path);
    let committed = directory.join(format!("{}.json", receipt.envelope_sha256));
    let before = fs::read(&committed).unwrap();
    let alias = directory.join(".export-crashed.tmp");
    fs::hard_link(&committed, &alias).unwrap();
    drop(UserReviewStore::open(&store, &body.request.case, StoreAccess::Read).unwrap());
    assert!(alias.is_file());
    let partial = directory.join(".export-partial.tmp");
    fs::write(&partial, b"{").unwrap();
    let error = UserReviewStore::open(&store, &body.request.case, StoreAccess::Read)
        .err()
        .unwrap();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert_eq!(error.details["phase"], "unpublished_partial");
    assert_eq!(fs::read(&committed).unwrap(), before);
    assert_eq!(fs::read(partial).unwrap(), b"{");
}
#[test]
fn corrupted_committed_bytes_are_preserved_and_not_replaced_with_empty_history() {
    let path = root();
    let store = UserReviewFiles::open(&path, true).unwrap();
    let body = body();
    let mut tx = UserReviewStore::open(&store, &body.request.case, StoreAccess::Write).unwrap();
    let receipt = tx.publish(&body, &Permit).unwrap();
    drop(tx);
    let committed = directory(&path).join(format!("{}.json", receipt.envelope_sha256));
    fs::write(&committed, b"{}").unwrap();
    assert_eq!(
        UserReviewStore::open(&store, &body.request.case, StoreAccess::Write)
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(fs::read(committed).unwrap(), b"{}");
    assert_eq!(json_count(&path), 1);
}
