use super::*;

#[test]
fn reserved_partial_temp_and_extra_files_require_manual_recovery_without_deletion() {
    for fixture in ["empty", "partial", "temp", "extra"] {
        let root = Temp::new();
        let store = ProposalFiles::open(&root.0).unwrap();
        seed(&store);
        let path = directory(&root, "decision-1");
        fs::create_dir_all(&path).unwrap();
        if fixture != "empty" {
            fs::write(path.join(format!("{}.json", "a".repeat(64))), b"{").unwrap();
        }
        if fixture == "temp" {
            fs::write(path.join(".export-fixture.tmp"), b"partial").unwrap();
        }
        if fixture == "extra" {
            fs::write(path.join("extra"), b"unexpected").unwrap();
        }
        let error = store.output_decisions("proposal-1").unwrap_err();
        assert_eq!(error.code, ErrorCode::NeedsInput);
        assert_eq!(error.remediation, "recover_output_decision_publication");
        let mut session = store.open_output_review("proposal-1").unwrap().unwrap();
        let request = prepared(session.current(), "decision-2");
        assert!(session.append(&request).is_err());
        assert!(path.exists());
        assert!(!directory(&root, "decision-2").exists());
    }
}

#[test]
fn digest_and_self_consistent_deserialized_record_tampering_are_rejected() {
    for rewrite_sha in [false, true] {
        let root = Temp::new();
        let store = ProposalFiles::open(&root.0).unwrap();
        seed(&store);
        record(&store, "decision-1");
        let old = file(&root, "decision-1");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&old).unwrap()).unwrap();
        value["counts_as_model_sample"] = serde_json::json!(true);
        let bytes = serde_json::to_vec(&value).unwrap();
        if rewrite_sha {
            let sha = crate::infrastructure::content_digest::sha256(
                &mut std::io::Cursor::new(&bytes),
                MAX_RECORD,
            )
            .unwrap()
            .1;
            fs::remove_file(&old).unwrap();
            fs::write(old.parent().unwrap().join(format!("{sha}.json")), bytes).unwrap();
        } else {
            fs::write(&old, bytes).unwrap();
        }
        assert_eq!(
            store.output_decisions("proposal-1").unwrap_err().code,
            ErrorCode::NeedsInput
        );
    }
}

#[test]
fn injected_reserved_and_published_failures_preserve_phase_and_existing_publication() {
    for fail_after_publish in [false, true] {
        let root = Temp::new();
        let store = ProposalFiles::open(&root.0).unwrap();
        seed(&store);
        let decision = fixed_decision(&store, "decision-1");
        let writer = WriterReservation::acquire(&store).unwrap();
        let error = publication::publish_with(&store, &writer, &decision, |phase| {
            if matches!(
                (phase, fail_after_publish),
                (publication::Phase::Reserved, false) | (publication::Phase::Published, true)
            ) {
                Err(ApiError::new(
                    ErrorCode::EngineFailed,
                    "Synthetic publication checkpoint",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::EngineFailed);
        assert_eq!(error.details["publication_may_exist"], true);
        assert!(directory(&root, "decision-1").exists());
        drop(writer);
        if fail_after_publish {
            assert_eq!(
                store.output_decisions("proposal-1").unwrap().records.len(),
                1
            );
            assert_eq!(record(&store, "decision-1").decision, decision);
        } else {
            assert_eq!(
                store.output_decisions("proposal-1").unwrap_err().code,
                ErrorCode::NeedsInput
            );
        }
    }
}

#[test]
fn oversized_read_and_invalid_prepublication_body_remain_bounded() {
    let root = Temp::new();
    let store = ProposalFiles::open(&root.0).unwrap();
    seed(&store);
    let mut invalid = fixed_decision(&store, "invalid");
    invalid.input.note = "x".repeat(MAX_RECORD as usize + 1);
    let writer = WriterReservation::acquire(&store).unwrap();
    assert_eq!(
        publication::publish(&store, &writer, &invalid)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(!root.0.join("output-decisions").exists());
    drop(writer);
    let dir = directory(&root, "oversized");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{}.json", "a".repeat(64))),
        vec![b' '; MAX_RECORD as usize + 1],
    )
    .unwrap();
    assert_eq!(
        store.output_decisions("proposal-1").unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
}

#[test]
fn readback_must_match_the_exact_body_and_hash_just_published() {
    let root = Temp::new();
    let store = ProposalFiles::open(&root.0).unwrap();
    seed(&store);
    let decision = fixed_decision(&store, "decision-1");
    let writer = WriterReservation::acquire(&store).unwrap();
    let error = publication::publish_with(&store, &writer, &decision, |phase| {
        if matches!(phase, publication::Phase::Published) {
            let mut exchanged = decision.clone();
            exchanged.input.note.push('!');
            let (body, sha) = records::encode(&exchanged)?;
            fs::remove_file(file(&root, "decision-1")).unwrap();
            fs::write(
                directory(&root, "decision-1").join(format!("{sha}.json")),
                body,
            )
            .unwrap();
        }
        Ok(())
    })
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert_eq!(error.details["phase"], "readback");
    assert_eq!(error.details["publication_may_exist"], true);
    assert!(directory(&root, "decision-1").exists());
}
