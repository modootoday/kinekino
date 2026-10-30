//! Contract fixtures only; filesystem atomicity/recovery require the durable adapter tests.
use super::timeline_evidence;
use super::*;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{api_values::DecimalU64, review_source::*},
};
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

#[derive(Default)]
struct Leases {
    case: AtomicUsize,
    media: AtomicUsize,
    media_calls: AtomicUsize,
    denied: AtomicBool,
    unavailable: AtomicBool,
}
struct Cases(Arc<Leases>);
struct CaseHold(ReviewCase, Arc<Leases>);
impl Drop for CaseHold {
    fn drop(&mut self) {
        self.1.case.fetch_sub(1, Ordering::SeqCst);
    }
}
impl CaseLease for CaseHold {
    fn case(&self) -> &ReviewCase {
        &self.0
    }
    fn check_current(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
impl CaseResolver for Cases {
    fn retain(&self, case: &ReviewCase) -> Result<Box<dyn CaseLease + '_>, ApiError> {
        if self.0.denied.load(Ordering::SeqCst) {
            return Err(ApiError::new(ErrorCode::NotFound, "case denied"));
        }
        self.0.case.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(CaseHold(case.clone(), self.0.clone())))
    }
}
struct Media(Arc<Leases>);
struct MediaHold(MediaReference, Arc<Leases>);
impl Drop for MediaHold {
    fn drop(&mut self) {
        self.1.media.fetch_sub(1, Ordering::SeqCst);
    }
}
impl MediaLease for MediaHold {
    fn identity(&self) -> &MediaReference {
        &self.0
    }
    fn check_current(&self) -> Result<(), ApiError> {
        Ok(())
    }
    fn verified_duration_us(&self) -> Option<u64> {
        Some(100)
    }
}
impl MediaResolver for Media {
    fn retain(
        &self,
        _: &ReviewCase,
        media: &MediaReference,
    ) -> Result<Box<dyn MediaLease + '_>, ApiError> {
        self.0.media_calls.fetch_add(1, Ordering::SeqCst);
        if self.0.unavailable.load(Ordering::SeqCst) {
            return Err(ApiError::new(ErrorCode::Unsupported, "media unavailable"));
        }
        self.0.media.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(MediaHold(media.clone(), self.0.clone())))
    }
}
#[derive(Default)]
struct Control {
    calls: AtomicUsize,
    cancel: AtomicBool,
    cancel_at: usize,
}
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if self.cancel.load(Ordering::SeqCst) || self.cancel_at == call {
            Err(ApiError::new(
                ErrorCode::Cancelled,
                "cancelled before publication",
            ))
        } else {
            Ok(())
        }
    }
}
#[derive(Default)]
struct State {
    commits: Vec<MutationReceipt>,
    ack_loss: bool,
    corrupt_return: bool,
    revoke_at_publish: Option<Arc<AtomicBool>>,
}
#[derive(Default)]
struct Store {
    state: Mutex<State>,
    leases: Arc<Leases>,
}
struct Tx<'a> {
    state: MutexGuard<'a, State>,
    case: ReviewCase,
    access: StoreAccess,
    leases: Arc<Leases>,
}
impl UserReviewStore for Store {
    fn open(
        &self,
        case: &ReviewCase,
        access: StoreAccess,
    ) -> Result<Box<dyn CaseTransaction + '_>, ApiError> {
        assert_eq!(self.leases.case.load(Ordering::SeqCst), 1);
        Ok(Box::new(Tx {
            state: self.state.lock().unwrap(),
            case: case.clone(),
            access,
            leases: self.leases.clone(),
        }))
    }
}
impl CaseTransaction for Tx<'_> {
    fn replay(&self, request: &MutationRequest) -> Result<Option<MutationReceipt>, ApiError> {
        for receipt in &self.state.commits {
            let old = &receipt.body.request;
            if old.case == request.case
                && old.scope() == request.scope()
                && old.idempotency_key == request.idempotency_key
            {
                return if old == request {
                    Ok(Some(receipt.clone()))
                } else {
                    Err(conflict("changed payload same key"))
                };
            }
        }
        Ok(None)
    }
    fn association(
        &self,
        id: &str,
        revision: Option<DecimalU64>,
    ) -> Result<Option<AssociationRevision>, ApiError> {
        Ok(self
            .state
            .commits
            .iter()
            .filter(|r| r.body.request.case == self.case)
            .map(|r| &r.body.association)
            .filter(|a| a.association_id == id && revision.is_none_or(|rev| rev == a.revision))
            .max_by_key(|a| a.revision)
            .cloned())
    }
    fn draft(
        &self,
        id: &str,
        revision: Option<DecimalU64>,
    ) -> Result<Option<RangeDraft>, ApiError> {
        Ok(self
            .state
            .commits
            .iter()
            .filter(|r| r.body.request.case == self.case)
            .filter_map(|r| r.body.draft.as_ref())
            .filter(|d| d.draft_id == id && revision.is_none_or(|rev| rev == d.draft_revision))
            .max_by_key(|d| d.draft_revision)
            .cloned())
    }
    fn publish(
        &mut self,
        body: &TransactionBody,
        guard: &dyn PublicationGuard,
    ) -> Result<MutationReceipt, ApiError> {
        assert_eq!(self.access, StoreAccess::Write);
        let expected = if matches!(body.request.mutation, Mutation::VerifyAssociation { .. }) {
            2
        } else {
            1
        };
        assert_eq!(self.leases.case.load(Ordering::SeqCst), expected);
        assert_eq!(self.leases.media.load(Ordering::SeqCst), expected);
        if let Some(revoke) = &self.state.revoke_at_publish {
            revoke.store(true, Ordering::SeqCst);
        }
        guard.before_publish()?;
        let receipt = MutationReceipt {
            envelope_sha256: "d".repeat(64),
            body: body.clone(),
        };
        self.state.commits.push(receipt.clone());
        if self.state.ack_loss {
            self.state.ack_loss = false;
            let mut e = ApiError::new(ErrorCode::EngineFailed, "ack lost");
            e.details =
                serde_json::json!({"delivery":"execution_unknown","publication_may_exist":true});
            return Err(e);
        }
        let mut returned = receipt;
        if self.state.corrupt_return {
            self.state.corrupt_return = false;
            returned.body.association.note.push('!');
        }
        Ok(returned)
    }
}
fn binding() -> AssociationRevision {
    AssociationRevision {
        association_id: "binding".into(),
        case: ReviewCase {
            package_sha256: "a".repeat(64),
            item_id: "item".into(),
            source_request_sha256: "b".repeat(64),
        },
        media: MediaReference {
            asset_id: "media".into(),
            sha256: "c".repeat(64),
        },
        revision: DecimalU64(1),
        timeline_status: TimelineStatus::Unverified,
        evidence_refs: vec![],
        note: String::new(),
    }
}
fn draft() -> RangeDraft {
    RangeDraft {
        draft_id: "draft".into(),
        association_id: "binding".into(),
        association_revision: DecimalU64(1),
        draft_revision: DecimalU64(1),
        authored_by: DraftAuthor::User,
        ranges: vec![UserRange {
            start_us: DecimalU64(1),
            end_us: DecimalU64(3),
        }],
        note: "user".into(),
    }
}
fn request(mutation: Mutation, key: &str, expected: Option<u64>) -> MutationRequest {
    MutationRequest {
        case: binding().case,
        idempotency_key: key.into(),
        expected_revision: expected.map(DecimalU64),
        mutation,
    }
}
fn fixture() -> (Cases, Media, Store) {
    let leases = Arc::new(Leases::default());
    (
        Cases(leases.clone()),
        Media(leases.clone()),
        Store {
            leases,
            ..Store::default()
        },
    )
}
struct Evidence {
    facts: crate::domain::review_source::timeline_evidence::TimelineObservations,
    revoked: Arc<AtomicBool>,
}
impl timeline_evidence::TimelineEvidenceLease for Evidence {
    fn observations(
        &self,
    ) -> &crate::domain::review_source::timeline_evidence::TimelineObservations {
        &self.facts
    }
    fn check_current(&self) -> Result<(), ApiError> {
        if self.revoked.load(Ordering::SeqCst) {
            Err(conflict("Evidence revoked"))
        } else {
            Ok(())
        }
    }
}
impl timeline_evidence::TimelineEvidenceResolver for Evidence {
    fn retain(
        &self,
        _: &ReviewCase,
        _: &MediaReference,
        _: &EvidenceReference,
    ) -> Result<Box<dyn timeline_evidence::TimelineEvidenceLease + '_>, ApiError> {
        Ok(Box::new(Evidence {
            facts: self.facts.clone(),
            revoked: self.revoked.clone(),
        }))
    }
}
fn verification() -> (Evidence, MutationRequest) {
    let mut facts = timeline_evidence::tests::observations();
    facts.case = binding().case;
    facts.media = binding().media;
    facts.origin.source_sha256 = facts.media.sha256.clone();
    facts.pcm.source_sha256 = facts.media.sha256.clone();
    facts.start_capture.source_sha256 = facts.media.sha256.clone();
    facts.end_capture.source_sha256 = facts.media.sha256.clone();
    let request = request(
        Mutation::VerifyAssociation {
            association_id: "binding".into(),
            media: facts.media.clone(),
            evidence: facts.reference.clone(),
            note: "proof admitted".into(),
        },
        "verify",
        Some(1),
    );
    (
        Evidence {
            facts,
            revoked: Arc::new(AtomicBool::new(false)),
        },
        request,
    )
}
#[cfg(windows)]
#[test]
fn verified_sdk_wire_reopens_real_store_and_reports_adapter_capability() {
    use crate::application::api::{PlayerSdk, Request};
    use crate::infrastructure::review_source::UserReviewFiles;
    use serde_json::json;
    let root = std::env::temp_dir().join(format!(
        "verified-sdk-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let (evidence, _) = verification();
    let reference = evidence.facts.reference.clone();
    let revoked = evidence.revoked.clone();
    let leases = Arc::new(Leases::default());
    let service = api::ReviewSourceService::new(
        Cases(leases.clone()),
        Media(leases.clone()),
        UserReviewFiles::open(&root, true).unwrap(),
        true,
    )
    .with_timeline_evidence(evidence);
    let sdk = PlayerSdk::default().with_review_source(service);
    let payload = json!({"case":binding().case,"association_id":"binding","media":binding().media,"note":"wire"});
    let mut create = Request::new("reviews.association.create");
    create.context.idempotency_key = Some("create".into());
    create.payload = payload.clone();
    assert!(sdk.execute(create).ok);
    let mut verify = Request::new("reviews.association.revise");
    verify.context.idempotency_key = Some("verify".into());
    verify.context.expected_revision = Some("1".into());
    verify.payload = payload;
    verify.payload["timeline_evidence"] = json!(reference);
    let result = sdk.execute(verify.clone());
    assert!(result.ok, "{:?}", result.error);
    let saved = result.data.unwrap();
    assert_eq!(
        saved["receipt"]["body"]["request"]["mutation"]["kind"],
        "verify_association"
    );
    assert!(saved["receipt"]["body"]["association"]["timeline_status"]["verified"].is_object());
    assert_eq!(saved["playback_ready"], false);
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(
        caps["review_source"]["timeline_verification_available"],
        true
    );
    let mut schema_request = Request::new("schema");
    schema_request.payload = json!({"command":"reviews.association.revise"});
    let schema = sdk.execute(schema_request.clone()).data.unwrap();
    assert_eq!(schema["timeline_verification_available"], true);
    assert!(
        schema["request"]["properties"]["payload"]["properties"]["timeline_evidence"].is_object()
    );
    let mut forged = verify.clone();
    forged.payload["timeline_status"] = json!("verified");
    assert_eq!(
        sdk.execute(forged).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    let mut create_with_evidence = verify.clone();
    create_with_evidence.command = "reviews.association.create".into();
    create_with_evidence.context.expected_revision = None;
    assert_eq!(
        sdk.execute(create_with_evidence).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    let (readonly_evidence, _) = verification();
    let readonly = PlayerSdk::default().with_review_source(
        api::ReviewSourceService::new(
            Cases(leases.clone()),
            Media(leases.clone()),
            UserReviewFiles::open(&root, false).unwrap(),
            false,
        )
        .with_timeline_evidence(readonly_evidence),
    );
    assert_eq!(
        readonly.execute(Request::new("capabilities")).data.unwrap()["review_source"]["timeline_verification_available"],
        false
    );
    assert_eq!(
        readonly.execute(verify.clone()).error.unwrap().code,
        ErrorCode::Unsupported
    );
    drop(readonly);
    drop(sdk);
    revoked.store(true, Ordering::SeqCst);
    leases.unavailable.store(true, Ordering::SeqCst);
    let reopened = PlayerSdk::default().with_review_source(api::ReviewSourceService::new(
        Cases(leases.clone()),
        Media(leases.clone()),
        UserReviewFiles::open(&root, true).unwrap(),
        true,
    ));
    assert_eq!(reopened.execute(verify.clone()).data.unwrap(), saved);
    let schema = reopened.execute(schema_request).data.unwrap();
    assert_eq!(schema["timeline_verification_available"], false);
    let mut inspect = Request::new("reviews.association.inspect");
    inspect.payload = json!({"case":binding().case,"association_id":"binding"});
    assert_eq!(
        reopened.execute(inspect).data.unwrap()["stored"]["association"]["revision"],
        "2"
    );
    leases.unavailable.store(false, Ordering::SeqCst);
    verify.context.idempotency_key = Some("new-verification".into());
    verify.context.expected_revision = Some("2".into());
    assert_eq!(
        reopened.execute(verify).error.unwrap().code,
        ErrorCode::Unsupported
    );
    // Keep the real envelopes as local verification evidence; no cleanup of prior runs.
    assert_eq!(leases.case.load(Ordering::SeqCst), 0);
    assert_eq!(leases.media.load(Ordering::SeqCst), 0);
}
#[test]
fn verified_publication_retains_proof_and_replays_without_adapter_media_or_control() {
    let (cases, media, store) = fixture();
    mutate(
        &cases,
        &media,
        &store,
        &Control::default(),
        request(
            Mutation::CreateAssociation(Box::new(binding())),
            "create",
            None,
        ),
    )
    .unwrap();
    let (evidence, verify) = verification();
    let saved = mutate_with_evidence(
        &cases,
        &media,
        &store,
        &Control::default(),
        verify.clone(),
        Some(&evidence),
    )
    .unwrap();
    assert!(matches!(
        saved.body.association.timeline_status,
        TimelineStatus::Verified(_)
    ));
    assert!(!saved.body.association.playback_ready());
    saved.body.validate().unwrap();
    let mut forged = saved.body.association.clone();
    forged.revision = DecimalU64(3);
    assert!(
        request(
            Mutation::ReviseAssociation(Box::new(forged)),
            "forged",
            Some(2)
        )
        .validate()
        .is_err()
    );
    evidence.revoked.store(true, Ordering::SeqCst);
    media.0.unavailable.store(true, Ordering::SeqCst);
    let cancelled = Control::default();
    cancelled.cancel.store(true, Ordering::SeqCst);
    assert_eq!(
        mutate(&cases, &media, &store, &cancelled, verify).unwrap(),
        saved
    );
    assert_eq!(store.state.lock().unwrap().commits.len(), 2);
    assert_eq!(store.leases.case.load(Ordering::SeqCst), 0);
    assert_eq!(store.leases.media.load(Ordering::SeqCst), 0);
}
#[test]
fn verified_publication_rechecks_late_revocation_and_cancellation_without_writing() {
    for revoke in [false, true] {
        let (cases, media, store) = fixture();
        mutate(
            &cases,
            &media,
            &store,
            &Control::default(),
            request(
                Mutation::CreateAssociation(Box::new(binding())),
                "create",
                None,
            ),
        )
        .unwrap();
        let (evidence, verify) = verification();
        let control = Control {
            cancel_at: if revoke { 0 } else { 3 },
            ..Control::default()
        };
        if revoke {
            store.state.lock().unwrap().revoke_at_publish = Some(evidence.revoked.clone());
        }
        let failure =
            mutate_with_evidence(&cases, &media, &store, &control, verify, Some(&evidence))
                .unwrap_err();
        assert_eq!(
            failure.code,
            if revoke {
                ErrorCode::RevisionConflict
            } else {
                ErrorCode::Cancelled
            }
        );
        assert_eq!(store.state.lock().unwrap().commits.len(), 1);
        assert_eq!(store.leases.case.load(Ordering::SeqCst), 0);
        assert_eq!(store.leases.media.load(Ordering::SeqCst), 0);
    }
}
fn combined() -> MutationRequest {
    request(
        Mutation::SaveDraft {
            draft: Box::new(draft()),
            create_association: Some(Box::new(binding())),
        },
        "combined",
        None,
    )
}

#[test]
fn association_revisions_and_old_draft_readback_keep_exact_media_binding() {
    let (cases, media, store) = fixture();
    let control = Control::default();
    mutate(
        &cases,
        &media,
        &store,
        &control,
        request(
            Mutation::CreateAssociation(Box::new(binding())),
            "create",
            None,
        ),
    )
    .unwrap();
    mutate(
        &cases,
        &media,
        &store,
        &control,
        request(
            Mutation::SaveDraft {
                draft: Box::new(draft()),
                create_association: None,
            },
            "save",
            None,
        ),
    )
    .unwrap();
    let mut next = binding();
    next.revision = DecimalU64(2);
    next.media.sha256 = "e".repeat(64);
    mutate(
        &cases,
        &media,
        &store,
        &control,
        request(
            Mutation::ReviseAssociation(Box::new(next.clone())),
            "revise",
            Some(1),
        ),
    )
    .unwrap();
    assert_eq!(
        inspect_association(&cases, &store, &binding().case, "binding", None).unwrap(),
        next
    );
    let old = read_draft(&cases, &store, &binding().case, "draft", None).unwrap();
    assert_eq!(old.association, binding());
    let mut relink = draft();
    relink.association_revision = DecimalU64(2);
    relink.draft_revision = DecimalU64(2);
    assert_eq!(
        mutate(
            &cases,
            &media,
            &store,
            &control,
            request(
                Mutation::SaveDraft {
                    draft: Box::new(relink),
                    create_association: None
                },
                "relink",
                Some(1)
            )
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(store.state.lock().unwrap().commits.len(), 3);
}
#[test]
fn committed_replay_survives_media_unavailability_and_cancel_but_never_bypasses_case_authority() {
    let (cases, media, store) = fixture();
    let control = Control::default();
    store.state.lock().unwrap().ack_loss = true;
    let original = combined();
    let error = mutate(&cases, &media, &store, &control, original.clone()).unwrap_err();
    assert_eq!(error.details["delivery"], "execution_unknown");
    media.0.unavailable.store(true, Ordering::SeqCst);
    control.cancel.store(true, Ordering::SeqCst);
    let replay = mutate(&cases, &media, &store, &control, original.clone()).unwrap();
    assert_eq!(replay.body.request, original);
    assert_eq!(media.0.media_calls.load(Ordering::SeqCst), 1);
    assert_eq!(store.state.lock().unwrap().commits.len(), 1);
    assert_eq!(
        read_draft(&cases, &store, &binding().case, "draft", None)
            .unwrap()
            .draft,
        draft()
    );
    let mut changed = original.clone();
    if let Mutation::SaveDraft { draft, .. } = &mut changed.mutation {
        draft.note.push('!');
    }
    assert_eq!(
        mutate(&cases, &media, &store, &control, changed)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    cases.0.denied.store(true, Ordering::SeqCst);
    assert_eq!(
        mutate(&cases, &media, &store, &control, original)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}
#[test]
fn early_and_last_publication_checkpoint_cancellation_leave_no_record_and_release_leases() {
    for at in [1, 3] {
        let (cases, media, store) = fixture();
        let control = Control {
            cancel_at: at,
            ..Control::default()
        };
        assert_eq!(
            mutate(&cases, &media, &store, &control, combined())
                .unwrap_err()
                .code,
            ErrorCode::Cancelled
        );
        assert!(store.state.lock().unwrap().commits.is_empty());
        assert_eq!(cases.0.case.load(Ordering::SeqCst), 0);
        assert_eq!(cases.0.media.load(Ordering::SeqCst), 0);
    }
}
#[test]
fn post_publication_bad_receipt_remains_unknown_and_same_key_recovers_committed_body() {
    let (cases, media, store) = fixture();
    let control = Control::default();
    store.state.lock().unwrap().corrupt_return = true;
    let error = mutate(&cases, &media, &store, &control, combined()).unwrap_err();
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.details["publication_may_exist"], true);
    assert_eq!(error.details["automatic_retry"], false);
    assert_eq!(store.state.lock().unwrap().commits.len(), 1);
    let replay = mutate(&cases, &media, &store, &control, combined()).unwrap();
    assert_eq!(replay.body.association.note, "");
}
#[test]
fn malformed_committed_receipt_is_preserved_as_storage_uncertainty_not_caller_error() {
    for corrupt_digest in [true, false] {
        let (cases, media, store) = fixture();
        let control = Control::default();
        mutate(&cases, &media, &store, &control, combined()).unwrap();
        {
            let mut state = store.state.lock().unwrap();
            if corrupt_digest {
                state.commits[0].envelope_sha256 = "invalid".into();
            } else {
                state.commits[0].body.association.media.sha256 = "invalid".into();
            }
        }
        let error = mutate(&cases, &media, &store, &control, combined()).unwrap_err();
        assert_eq!(error.code, ErrorCode::EngineFailed);
        assert_eq!(error.details["delivery"], "execution_unknown");
        assert_eq!(error.details["publication_may_exist"], true);
        assert_eq!(store.state.lock().unwrap().commits.len(), 1);
        assert_eq!(media.0.media_calls.load(Ordering::SeqCst), 1);
    }
}
