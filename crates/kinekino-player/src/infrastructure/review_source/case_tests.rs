//! Real ReviewFiles custody; synthetic package, no original media or decisions.
use super::UserReviewFiles;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::{
            CaseResolver, MediaResolver, Mutation, MutationControl, MutationRequest, ReviewCases,
            ReviewPackageMedia, inspect_association, mutate,
        },
    },
    domain::{
        api_values::DecimalU64,
        review_source::{AssociationRevision, MediaReference, ReviewCase, TimelineStatus},
    },
    infrastructure::model_review::ReviewFiles,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    fs::OpenOptions,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn case_retains_exact_item_source_and_releases_real_file_guards() {
    let root = std::env::temp_dir().join(format!(
        "review-case-custody-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&root).unwrap();
    for name in ["packages", "objects", "decisions"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    // Paths inside source request bytes are evidence only; the resolver never opens them.
    let source = br#"{"path":"Z:/never-open-this/private.mp4"}"#;
    let candidate = b"independent candidate";
    // Synthetic bytes test custody only; no real decoder or PTS claim.
    let video = b"synthetic declared video custody fixture\0\xff";
    let source_sha = sha(source);
    let candidate_sha = sha(candidate);
    let video_sha = sha(video);
    fs::write(root.join("objects").join(&source_sha), source).unwrap();
    fs::write(root.join("objects").join(&candidate_sha), candidate).unwrap();
    fs::write(root.join("objects").join(&video_sha), video).unwrap();
    let mut package_json = serde_json::json!({
        "schema_version":"1","package_id":"fixture","items":[{
            "item_id":"item","task":"asr",
            "source":{"asset_id":"request","sha256":source_sha,"bytes":source.len().to_string(),"media_type":"application/json"},
            "range":null,"baseline":null,
            "candidate":{
                "condition":{"model_id":"fixture","revision":"r1","precision":"fp32","engine_id":"fixture","engine_revision":"r1","input_conditions":{"sample":"synthetic"}},
                "assets":[{"asset_id":"candidate","sha256":candidate_sha,"bytes":candidate.len().to_string(),"media_type":"text/plain"},
                    {"asset_id":"video","sha256":video_sha,"bytes":video.len().to_string(),"media_type":"video/mp4"}],
                "automated_checks":[{"check_id":"quality","outcome":"not_run","summary":"Not quality approved","evidence":[]}],
                "measured_resources":{"kind":"synthetic"}
            },
            "criteria":[{"criterion_id":"compare","how_to_review":"Compare source","expected":"Faithful result"}],"known_issues":[]
        }]
    });
    let other_bytes = b"only referenced by the other item";
    let other_sha = sha(other_bytes);
    fs::write(root.join("objects").join(&other_sha), other_bytes).unwrap();
    let mut other = package_json["items"][0].clone();
    other["item_id"] = "other".into();
    other["candidate"]["assets"] = serde_json::json!([{"asset_id":"other-evidence","sha256":other_sha,"bytes":other_bytes.len().to_string(),"media_type":"application/octet-stream"}]);
    package_json["items"].as_array_mut().unwrap().push(other);
    let body = serde_json::to_vec(&package_json).unwrap();
    let package_sha = sha(&body);
    let package_path = root.join("packages").join(format!("{package_sha}.json"));
    fs::write(&package_path, body).unwrap();
    {
        use crate::application::model_review::{MAX_BINARY_EVIDENCE_BYTES, ReviewStore};
        let store = ReviewFiles::open(&root).unwrap();
        let session = store.open(&package_sha).unwrap();
        assert_eq!(
            session
                .read_evidence_asset("video", &video_sha, 1024)
                .unwrap(),
            video
        );
        assert_eq!(
            session
                .read_evidence_asset("request", &source_sha, 1024)
                .unwrap(),
            source
        );
        assert!(
            matches!(session.read_evidence_asset("foreign",&video_sha,1024),Err(e) if e.code==ErrorCode::NotFound)
        );
        assert!(
            matches!(session.read_evidence_asset("video",&candidate_sha,1024),Err(e) if e.code==ErrorCode::RevisionConflict)
        );
        assert!(
            matches!(session.read_evidence_asset("video",&video_sha,1),Err(e) if e.code==ErrorCode::ResourceExhausted)
        );
        for bound in [0, MAX_BINARY_EVIDENCE_BYTES + 1] {
            assert!(
                matches!(session.read_evidence_asset("video",&video_sha,bound),Err(e) if e.code==ErrorCode::InvalidArgument)
            );
        }
        assert_eq!(
            OpenOptions::new()
                .write(true)
                .open(root.join("objects").join(&video_sha))
                .is_err(),
            crate::infrastructure::os_policy::has_mandatory_share_modes()
        );
    }
    let cases = ReviewCases::new(ReviewFiles::open(&root).unwrap());
    let case = ReviewCase {
        package_sha256: package_sha,
        item_id: "item".into(),
        source_request_sha256: source_sha.clone(),
    };
    {
        use crate::{
            application::review_source::ReviewCaseEvidence,
            domain::review_source::EvidenceReference,
        };
        let resolver = ReviewCaseEvidence::new(ReviewFiles::open(&root).unwrap());
        let retained = resolver.retain(&case).unwrap();
        assert_eq!(retained.case(), &case);
        assert_eq!(
            retained
                .read(
                    &EvidenceReference {
                        asset_id: "video".into(),
                        sha256: video_sha.clone()
                    },
                    1024
                )
                .unwrap(),
            video
        );
        let foreign = EvidenceReference {
            asset_id: "other-evidence".into(),
            sha256: other_sha.clone(),
        };
        assert!(matches!(retained.read(&foreign,1024),Err(e) if e.code==ErrorCode::NotFound));
        let other_case = ReviewCase {
            item_id: "other".into(),
            ..case.clone()
        };
        let other_retained = resolver.retain(&other_case).unwrap();
        assert_eq!(other_retained.read(&foreign, 1024).unwrap(), other_bytes);
        assert_eq!(
            OpenOptions::new()
                .write(true)
                .open(root.join("objects").join(&other_sha))
                .is_err(),
            crate::infrastructure::os_policy::has_mandatory_share_modes()
        );
        assert!(
            matches!(retained.read(&EvidenceReference{asset_id:"video".into(),sha256:other_sha.clone()},1024),Err(e) if e.code==ErrorCode::RevisionConflict)
        );
        assert!(
            matches!(retained.read(&EvidenceReference{asset_id:"../outside".into(),sha256:video_sha.clone()},1024),Err(e) if e.code==ErrorCode::InvalidArgument)
        );
    }
    let media = ReviewPackageMedia::new(ReviewFiles::open(&root).unwrap());
    let reference = MediaReference {
        asset_id: "video".into(),
        sha256: video_sha.clone(),
    };
    let media_lease = media.retain(&case, &reference).unwrap();
    assert_eq!(media_lease.identity(), &reference);
    assert_eq!(media_lease.verified_duration_us(), None);
    media_lease.check_current().unwrap();
    let video_path = root.join("objects").join(&video_sha);
    let mandatory = crate::infrastructure::os_policy::has_mandatory_share_modes();
    assert_eq!(
        OpenOptions::new().write(true).open(&video_path).is_err(),
        mandatory
    );
    if mandatory {
        assert!(fs::remove_file(&video_path).is_err());
    } else {
        // POSIX: the retained original handle, not the name, is the authority.
        media_lease.check_current().unwrap();
    }
    let wrong_media = MediaReference {
        asset_id: "video".into(),
        sha256: source_sha.clone(),
    };
    assert!(
        matches!(media.retain(&case, &wrong_media), Err(e) if e.code == ErrorCode::RevisionConflict)
    );
    let not_media = MediaReference {
        asset_id: "request".into(),
        sha256: source_sha.clone(),
    };
    assert!(matches!(media.retain(&case, &not_media), Err(e) if e.code == ErrorCode::Unsupported));
    let missing_media = MediaReference {
        asset_id: "foreign".into(),
        sha256: video_sha.clone(),
    };
    assert!(matches!(media.retain(&case, &missing_media), Err(e) if e.code == ErrorCode::NotFound));
    drop(media_lease);
    drop(OpenOptions::new().write(true).open(&video_path).unwrap());
    drop(media.retain(&case, &reference).unwrap());
    let lease = cases.retain(&case).unwrap();
    assert_eq!(lease.case(), &case);
    lease.check_current().unwrap();
    for path in [
        &package_path,
        &root.join("objects").join(&source_sha),
        &root.join("objects").join(&candidate_sha),
    ] {
        assert_eq!(
            OpenOptions::new().write(true).open(path).is_err(),
            crate::infrastructure::os_policy::has_mandatory_share_modes()
        );
        if crate::infrastructure::os_policy::has_mandatory_share_modes() {
            assert!(fs::remove_file(path).is_err());
        }
    }
    // POSIX: names are not protected; the retained original handles stay the authority.
    lease.check_current().unwrap();
    let mut wrong = case.clone();
    wrong.source_request_sha256 = candidate_sha;
    assert!(matches!(cases.retain(&wrong), Err(e) if e.code == ErrorCode::RevisionConflict));
    wrong = case.clone();
    wrong.item_id = "missing".into();
    assert!(matches!(cases.retain(&wrong), Err(e) if e.code == ErrorCode::NotFound));
    wrong = case.clone();
    wrong.package_sha256 = "a".repeat(64);
    assert!(matches!(cases.retain(&wrong), Err(e) if e.code == ErrorCode::NotFound));
    wrong = case.clone();
    wrong.item_id = "../item".into();
    assert!(matches!(cases.retain(&wrong), Err(e) if e.code == ErrorCode::InvalidArgument));
    drop(lease);
    let released = OpenOptions::new().write(true).open(&package_path).unwrap();
    drop(released);
    let again = cases.retain(&case).unwrap();
    again.check_current().unwrap();
    drop(again);
    let user_root = root.with_extension("user-review");
    fs::create_dir(&user_root).unwrap();
    let store = UserReviewFiles::open(&user_root, true).unwrap();
    struct Protected<'a>(&'a std::path::Path, AtomicUsize);
    impl MutationControl for Protected<'_> {
        fn checkpoint(&self) -> Result<(), ApiError> {
            assert_eq!(
                OpenOptions::new().write(true).open(self.0).is_err(),
                crate::infrastructure::os_policy::has_mandatory_share_modes()
            );
            self.1.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    let control = Protected(&video_path, AtomicUsize::new(0));
    let association = AssociationRevision {
        association_id: "real-custody".into(),
        case: case.clone(),
        media: reference,
        revision: DecimalU64(1),
        timeline_status: TimelineStatus::Unverified,
        evidence_refs: vec![],
        note: "Explicit manual association; no timeline proof".into(),
    };
    let request = MutationRequest {
        case: case.clone(),
        idempotency_key: "create-real-custody".into(),
        expected_revision: None,
        mutation: Mutation::CreateAssociation(Box::new(association.clone())),
    };
    let receipt = mutate(&cases, &media, &store, &control, request.clone()).unwrap();
    assert!(control.1.load(Ordering::SeqCst) >= 3);
    assert_eq!(receipt.body.association, association);
    assert!(!receipt.body.association.playback_ready());
    assert_eq!(
        inspect_association(&cases, &store, &case, "real-custody", None).unwrap(),
        association
    );
    let before_replay = control.1.load(Ordering::SeqCst);
    assert_eq!(
        mutate(&cases, &media, &store, &control, request).unwrap(),
        receipt
    );
    assert_eq!(control.1.load(Ordering::SeqCst), before_replay);
    drop(OpenOptions::new().write(true).open(&video_path).unwrap());
    {
        use crate::application::{api::Request, review_source::api::ReviewSourceApi};
        assert!(
            matches!(super::open_api(&root,&root,true), Err(e) if e.code == ErrorCode::InvalidArgument)
        );
        let nested = root.join("forbidden-user-root");
        fs::create_dir(&nested).unwrap();
        assert!(
            matches!(super::open_api(&root,&nested,true), Err(e) if e.code == ErrorCode::InvalidArgument)
        );
        assert!(!nested.join("cases").exists());
        let service = super::open_api(&root, &user_root, true).unwrap();
        let mut command = Request::new("reviews.association.create");
        command.context.idempotency_key = Some("transport-create".into());
        command.payload = serde_json::json!({"case":case,"association_id":"transport","media":{"asset_id":"video","sha256":video_sha},"note":"manual"});
        let first = service.execute(&command).unwrap();
        assert_eq!(first["receipt"]["body"]["association"]["revision"], "1");
        assert_eq!(service.execute(&command).unwrap(), first);
        let mut invalid = command.clone();
        invalid.payload["timeline_status"] = "verified".into();
        assert!(
            matches!(service.execute(&invalid), Err(e) if e.code == ErrorCode::InvalidArgument)
        );
        invalid = command.clone();
        invalid.context.idempotency_key = None;
        assert!(
            matches!(service.execute(&invalid), Err(e) if e.code == ErrorCode::InvalidArgument)
        );
        invalid = command.clone();
        invalid.context.timeout_ms = Some(120001);
        assert!(
            matches!(service.execute(&invalid), Err(e) if e.code == ErrorCode::InvalidArgument)
        );
        command.command = "reviews.association.revise".into();
        command.context.expected_revision = Some("1".into());
        command.context.idempotency_key = Some("transport-revise".into());
        command.payload["note"] = "revised".into();
        assert_eq!(
            service.execute(&command).unwrap()["receipt"]["body"]["association"]["revision"],
            "2"
        );
        command = Request::new("reviews.association.inspect");
        command.payload = serde_json::json!({"case":case,"association_id":"transport"});
        let view = service.execute(&command).unwrap();
        assert_eq!(view["stored"]["association"]["revision"], "2");
        assert_eq!(view["media_check"], "not_checked");
        assert_eq!(view["playback_ready"], false);
        command.context.idempotency_key = Some("forbidden-read".into());
        assert!(
            matches!(service.execute(&command), Err(e) if e.code == ErrorCode::InvalidArgument)
        );
        command = Request::new("reviews.range_draft.save");
        command.context.idempotency_key = Some("transport-draft".into());
        command.payload = serde_json::json!({"case":case,"draft":{"draft_id":"transport-draft","association_id":"transport","association_revision":"1","draft_revision":"1","authored_by":"user","ranges":[{"start_us":"1","end_us":"3"}],"note":"manual draft"}});
        let saved = service.execute(&command).unwrap();
        assert_eq!(service.execute(&command).unwrap(), saved);
        command = Request::new("reviews.range_draft.read");
        command.payload = serde_json::json!({"case":case,"draft_id":"transport-draft"});
        assert_eq!(
            service.execute(&command).unwrap()["stored"]["association"]["revision"],
            "1"
        );
        let readonly = super::open_api(&root, &user_root, false).unwrap();
        assert_eq!(
            readonly.execute(&command).unwrap(),
            service.execute(&command).unwrap()
        );
        command = Request::new("reviews.association.create");
        command.context.idempotency_key = Some("readonly-denied".into());
        command.payload = serde_json::json!({"case":case,"association_id":"denied","media":{"asset_id":"video","sha256":video_sha},"note":"manual"});
        assert!(matches!(readonly.execute(&command), Err(e) if e.code == ErrorCode::Unsupported));
        let sdk = crate::application::api::PlayerSdk::default().with_review_source(service);
        for name in crate::application::review_source::api::COMMANDS {
            let mut schema = Request::new("schema");
            schema.payload = serde_json::json!({"command":name});
            let result = sdk.execute(schema);
            assert!(result.ok, "{name}: {:?}", result.error);
            assert_eq!(result.data.unwrap()["status"], "supported");
        }
        let capabilities = sdk.execute(Request::new("capabilities")).data.unwrap();
        assert_eq!(capabilities["review_source"]["write_available"], true);
        assert_eq!(
            capabilities["review_source"]["strong_cancel_available"],
            false
        );
        let read = Request {
            payload: serde_json::json!({"case":case,"draft_id":"transport-draft"}),
            ..Request::new("reviews.range_draft.read")
        };
        assert!(sdk.execute(read).ok);
        let unavailable = crate::application::api::PlayerSdk::default();
        assert!(
            !unavailable
                .execute(Request::new("reviews.range_draft.read"))
                .ok
        );
    }
    // Detect post-lease tampering on a fresh open; never reuse old membership proof.
    fs::write(root.join("objects").join(&source_sha), b"tampered").unwrap();
    assert!(matches!(cases.retain(&case), Err(e) if e.code == ErrorCode::RevisionConflict));
}
