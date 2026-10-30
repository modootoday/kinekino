use super::*;
use crate::application::api::ErrorCode;
use serde_json::Value;

fn edit_bound_task(
    lease: &AsyncProposalRegistration,
    registry: &Rc<RefCell<PluginRegistry>>,
    context: &Rc<RefCell<RequestContext>>,
    imported: &Value,
) -> Value {
    use crate::domain::proposals::{ProposalTask, StoredProposal};
    let stored: StoredProposal = serde_json::from_value(imported.clone()).unwrap();
    let id = &stored.document.proposal_id;
    let (draft, foreign) = match stored.document.generated.task {
        ProposalTask::Correction => {
            let source = stored.document.source.as_document().unwrap();
            let reference = source
                .text_reference(&source.texts[0].inner_pointer)
                .unwrap();
            let draft = json!({"task":"document_correction","edits":[{"source_ref":reference,"text":"Technical manual repair; no quality approval."}]});
            let mut foreign = draft.clone();
            foreign["edits"][0]["source_ref"]["source_asset_sha256"] = json!("0".repeat(64));
            (draft, foreign)
        }
        ProposalTask::Cards => {
            let cue = &stored.document.source.cues()[0].cue_id;
            (
                json!({"task":"cards","cards":[{"front":"Technical question","back":"Technical answer","source_refs":[cue]}]}),
                json!({"task":"cards","cards":[{"front":"Invalid","back":"Invalid","source_refs":["foreign-cue"]}]}),
            )
        }
        ProposalTask::Highlights => {
            let cue = &stored.document.source.cues()[0].cue_id;
            (
                json!({"task":"highlights","highlights":[{"title":"Technical highlight","source_refs":[cue]}]}),
                json!({"task":"highlights","highlights":[{"title":"Invalid","source_refs":["foreign-cue"]}]}),
            )
        }
    };
    context.borrow_mut().expected_revision = Some("1".into());
    assert_eq!(
        invoke(
            lease,
            registry,
            "proposals.edit",
            json!({"proposal_id":id,"draft":foreign})
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        invoke(
            lease,
            registry,
            "proposals.edit",
            json!({"proposal_id":id,"draft":draft})
        )
        .unwrap()["draft_revision"],
        "2"
    );
    assert_eq!(
        invoke(
            lease,
            registry,
            "proposals.edit",
            json!({"proposal_id":id,"draft":draft})
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    context.borrow_mut().expected_revision = Some("2".into());
    let accepted = invoke(
        lease,
        registry,
        "proposals.accept",
        json!({"proposal_id":id,"target_source_revision":"1"}),
    )
    .unwrap();
    assert_eq!(
        invoke(
            lease,
            registry,
            "proposals.accept",
            json!({"proposal_id":id,"target_source_revision":"1"})
        )
        .unwrap(),
        accepted
    );
    assert_eq!(accepted["automatic_application_enabled"], false);
    assert_eq!(accepted["output"], draft);
    context.borrow_mut().expected_revision = None;
    let latest = invoke(
        lease,
        registry,
        "proposals.inspect",
        json!({"proposal_id":id}),
    )
    .unwrap();
    assert_eq!(latest["document"]["source"], imported["document"]["source"]);
    assert_eq!(
        latest["document"]["generated"],
        imported["document"]["generated"]
    );
    assert_eq!(latest["historical"], imported["historical"]);
    assert_eq!(latest["acceptance"], accepted);
    latest
}

fn invoke(
    lease: &AsyncProposalRegistration,
    registry: &Rc<RefCell<PluginRegistry>>,
    command: &str,
    input: Value,
) -> Result<Value, ApiError> {
    let registration = lease.handle().unwrap().clone();
    let descriptor = registry
        .borrow()
        .manifest(&registration)
        .unwrap()
        .capabilities[command]
        .clone();
    let receipt = registry
        .borrow_mut()
        .dispatch(
            &registration,
            command,
            PluginPayload::encode(descriptor.request_schema, &input).unwrap(),
        )
        .unwrap();
    assert!(matches!(receipt.outcome, PluginDispatch::Pending));
    match drain(lease, registry, &receipt.ticket).event {
        PluginEventKind::Completed(payload) => {
            Ok(payload.decode(&descriptor.reply_schema).unwrap())
        }
        PluginEventKind::Failed(error) => Err(error
            .details
            .unwrap()
            .decode(&descriptor.error_schema.unwrap())
            .unwrap()),
        _ => panic!("S1 SDK work must retain its actual success or API failure"),
    }
}

#[test]
fn native_pending_three_tasks_preserve_cas_receipts_raw_and_fresh_store_state() {
    let temp = Temp::new();
    let sdk: ApplicationClientHandle = Arc::new(
        PlayerSdk::new(SubtitleFiles).with_proposal_store(ProposalFiles::open(&temp.0).unwrap()),
    );
    let record = Arc::new(Record {
        inner: sdk,
        threads: Mutex::new(Vec::new()),
    });
    let owner = execution(record.clone());
    let (_program, scope, registry) = scope();
    let context = Rc::new(RefCell::new(RequestContext::default()));
    let mut lease = AsyncProposalRegistration::register(
        registry.clone(),
        &scope,
        context.clone(),
        owner.handle(),
        codec(),
    )
    .unwrap();
    let source = json!({"source_id":"technical-source","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"explicit-cue","text":"unchanged source","start_us":null,"end_us":null}]});
    for (task, draft, foreign) in [
        (
            "correction",
            json!({"task":"correction","edits":[{"cue_id":"explicit-cue","text":"manual edit"}]}),
            json!({"task":"correction","edits":[{"cue_id":"foreign-cue","text":"invalid"}]}),
        ),
        (
            "cards",
            json!({"task":"cards","cards":[{"front":"question","back":"answer","source_refs":["explicit-cue"]}]}),
            json!({"task":"cards","cards":[{"front":"question","back":"answer","source_refs":["foreign-cue"]}]}),
        ),
        (
            "highlights",
            json!({"task":"highlights","highlights":[{"title":"manual highlight","source_refs":["explicit-cue"]}]}),
            json!({"task":"highlights","highlights":[{"title":"invalid","source_refs":["foreign-cue"]}]}),
        ),
    ] {
        context.borrow_mut().expected_revision = None;
        let id = format!("native-{task}");
        let raw = format!("preserved malformed {task} raw");
        let created = invoke(
            &lease,
            &registry,
            "proposals.create",
            json!({"proposal_id":id,"source":source,"generated":{"task":task,"raw_text":raw}}),
        )
        .unwrap();
        assert_eq!(created["state"], "needs_repair");
        let original = invoke(
            &lease,
            &registry,
            "proposals.inspect",
            json!({"proposal_id":id}),
        )
        .unwrap();
        context.borrow_mut().expected_revision = Some("1".into());
        assert_eq!(
            invoke(
                &lease,
                &registry,
                "proposals.edit",
                json!({"proposal_id":id,"draft":foreign})
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        let edited = invoke(
            &lease,
            &registry,
            "proposals.edit",
            json!({"proposal_id":id,"draft":draft}),
        )
        .unwrap();
        assert_eq!(edited["draft_revision"], "2");
        assert_eq!(
            invoke(
                &lease,
                &registry,
                "proposals.edit",
                json!({"proposal_id":id,"draft":draft})
            )
            .unwrap_err()
            .code,
            ErrorCode::RevisionConflict
        );
        context.borrow_mut().expected_revision = Some("2".into());
        let accepted = invoke(
            &lease,
            &registry,
            "proposals.accept",
            json!({"proposal_id":id,"target_source_revision":"1"}),
        )
        .unwrap();
        let retried = invoke(
            &lease,
            &registry,
            "proposals.accept",
            json!({"proposal_id":id,"target_source_revision":"1"}),
        )
        .unwrap();
        assert_eq!(accepted, retried);
        assert_eq!(accepted["automatic_application_enabled"], false);
        assert_eq!(accepted["output"], draft);
        context.borrow_mut().expected_revision = None;
        let latest = invoke(
            &lease,
            &registry,
            "proposals.inspect",
            json!({"proposal_id":id}),
        )
        .unwrap();
        assert_eq!(latest["document"]["source"], original["document"]["source"]);
        assert_eq!(latest["document"]["generated"]["raw_text"], raw);
        assert_eq!(latest["acceptance"], accepted);
        let fresh = PlayerSdk::new(SubtitleFiles)
            .with_proposal_store(ProposalFiles::open(&temp.0).unwrap());
        let mut request = Request::new("proposals.inspect");
        request.payload = json!({"proposal_id":id});
        let fresh = fresh.execute(request);
        assert!(fresh.ok);
        assert_eq!(fresh.data.unwrap(), latest);
    }
    // The fifth capability must report missing configuration through the canonical SDK.
    let error = invoke(&lease, &registry, "proposals.import_historical", json!({"proposal_id":"historical-unconfigured","source_id":"historical-source","source_revision":"1","reference":{"sha256":"b".repeat(64),"bytes":20,"item_id":"explicit-item"}})).unwrap_err();
    assert_eq!(error.code, ErrorCode::Unsupported);
    let threads = record.threads.lock().unwrap();
    assert_eq!(threads.len(), 25);
    assert!(
        threads
            .iter()
            .all(|id| *id == threads[0] && *id != thread::current().id())
    );
    drop(threads);
    lease.unbind().unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        if let Ok(receipt) = owner.close() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(windows)]
#[test]
#[ignore = "requires explicitly pinned verifier and preserved actual-nine package"]
fn native_pending_imports_nine_originals_with_same_sdk_and_pinned_verifier() {
    use crate::infrastructure::historical_evidence::HistoricalVerifier;
    use sha2::{Digest, Sha256};
    let executable =
        std::env::var_os("HISTORICAL_VERIFIER_EXE").expect("explicit verifier path required");
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let reader = Arc::new(
        HistoricalVerifier::open(
            std::path::Path::new(&executable),
            "e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41".into(),
            root,
            "sdk-text-results-r3".into(),
        )
        .unwrap(),
    );
    let temp = Temp::new();
    let store = ProposalFiles::open(&temp.0)
        .unwrap()
        .with_historical_reader(reader.clone());
    let sdk: ApplicationClientHandle =
        Arc::new(PlayerSdk::new(SubtitleFiles).with_proposal_store(store));
    let record = Arc::new(Record {
        inner: sdk,
        threads: Mutex::new(Vec::new()),
    });
    let owner = execution(record.clone());
    let (_program, scope, registry) = scope();
    let context = Rc::new(RefCell::new(RequestContext::default()));
    let mut lease = AsyncProposalRegistration::register(
        registry.clone(),
        &scope,
        context.clone(),
        owner.handle(),
        codec(),
    )
    .unwrap();
    let companion_path = std::env::var_os("HISTORICAL_COMPANION_PATH");
    let companion = crate::test_support::historical_fixture::read_companion(
        companion_path.as_deref().map(std::path::Path::new),
        "7a7cd05eafa6294e846c8080c976600ea135aa91e9da315ac9f770e45f624b10",
        16566,
    )
    .expect("explicit preserved companion required");
    let rows = companion["items"].as_array().unwrap();
    assert_eq!(rows.len(), 9);
    for row in rows {
        let item = row["item_id"].as_str().unwrap();
        let id = format!("native-import-{item}");
        let reference = json!({"sha256":"7a7cd05eafa6294e846c8080c976600ea135aa91e9da315ac9f770e45f624b10","bytes":16566,"item_id":item});
        let imported = invoke(&lease, &registry, "proposals.import_historical", json!({"proposal_id":id,"source_id":"actual-historical-source","source_revision":"1","reference":reference})).unwrap();
        assert_eq!(imported["document"]["schema_version"], "3");
        assert_eq!(imported["historical"]["reference"], reference);
        let raw = imported["document"]["generated"]["raw_text"]
            .as_str()
            .unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(raw.as_bytes())),
            row["candidate"]["sha256"].as_str().unwrap()
        );
        assert_eq!(
            imported["document"]["source"]["source_sha256"],
            row["source"]["sha256"]
        );
        let inspected = invoke(
            &lease,
            &registry,
            "proposals.inspect",
            json!({"proposal_id":id}),
        )
        .unwrap();
        assert_eq!(inspected, imported);
        let latest = if item.starts_with("gemma270-") {
            edit_bound_task(&lease, &registry, &context, &imported)
        } else {
            imported
        };
        let fresh = PlayerSdk::new(SubtitleFiles).with_proposal_store(
            ProposalFiles::open(&temp.0)
                .unwrap()
                .with_historical_reader(reader.clone()),
        );
        let mut request = Request::new("proposals.inspect");
        request.payload = json!({"proposal_id":id});
        let response = fresh.execute(request);
        assert!(response.ok);
        assert_eq!(response.data.unwrap(), latest);
    }
    let threads = record.threads.lock().unwrap();
    assert_eq!(threads.len(), 36);
    assert!(
        threads
            .iter()
            .all(|id| *id == threads[0] && *id != thread::current().id())
    );
    drop(threads);
    lease.unbind().unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        if let Ok(receipt) = owner.close() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(1));
    }
}
