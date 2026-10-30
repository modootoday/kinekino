#![cfg(all(windows, feature = "card-review-export"))]
#[path = "accepted_card_export_cli/artifact.rs"]
mod artifact;
#[path = "accepted_card_export_cli/fixture.rs"]
mod fixture;
#[path = "accepted_card_export_cli/invocation.rs"]
mod invocation;
#[allow(dead_code)]
#[path = "support/owned_process_evidence.rs"]
mod owned_process_evidence;

use kinekino_player::{
    application::{api::RequestContext, proposals::card_export::LIMITATIONS},
    domain::api_values::DecimalU64,
    infrastructure::content_digest,
};
use serde_json::json;
use std::{fs, path::PathBuf};

// Existing shared owned-process helper's inline tests require the parent digest adapter.
fn hash(bytes: &[u8]) -> String {
    content_digest::sha256(&mut std::io::Cursor::new(bytes), bytes.len() as u64)
        .unwrap()
        .1
}

#[test]
#[ignore = "Fixed feature-enabled CLI, fresh evidence root and central CPU/OS allocation required"]
fn accepted_card_review_cli_publishes_real_png_and_preserves_binding_boundaries() {
    kinekino_platform::install();
    let root = PathBuf::from(std::env::var_os("ACCEPTED_CARD_EXPORT_EVIDENCE_DIR").unwrap());
    let executable = PathBuf::from(std::env::var_os("ACCEPTED_CARD_EXPORT_CLI").unwrap());
    let pin = std::env::var("ACCEPTED_CARD_EXPORT_CLI_SHA256").unwrap();
    assert!(root.is_absolute() && !root.exists());
    assert!(executable.is_absolute());
    let (_, actual) =
        content_digest::sha256(&mut fs::File::open(&executable).unwrap(), 512 * 1024 * 1024)
            .unwrap();
    assert!(pin.eq_ignore_ascii_case(&actual));
    fs::create_dir(&root).unwrap();
    let fixture = fixture::Fixture::create(&root);
    let before = fixture::store_snapshot(&fixture.store_path);
    fs::write(
        root.join("store-before.json"),
        serde_json::to_vec_pretty(&before).unwrap(),
    )
    .unwrap();
    let frame_before = fs::read(&fixture.frame).unwrap();
    let target_before = fixture.logical_snapshot();
    fs::write(root.join("accepted-before.json"), &target_before).unwrap();
    let output = root.join("review.png");
    let valid = fixture.input(&output);
    let run = |label, input, exit| {
        invocation::invoke(
            &executable,
            &root,
            &fixture.store_path,
            label,
            input,
            RequestContext::default(),
            exit,
        )
    };
    let response = run("new-png", valid.clone(), 0);
    let data = &response["data"];
    assert_eq!(
        data["target"],
        serde_json::to_value(&fixture.target).unwrap()
    );
    assert_eq!(data["position"], "0");
    assert_eq!(data["source_refs"], json!(["cue-1"]));
    assert_eq!(data["state"], "review_artifact");
    for name in [
        "quality_approved",
        "template_approved",
        "overwrite",
        "cancellation_supported",
        "timeout_supported",
    ] {
        assert_eq!(data[name], false, "{name}");
    }
    assert_eq!(data["limitations"], json!(LIMITATIONS));
    let pixels = artifact::inspect(&output, data, &valid);
    fs::write(
        root.join("pixel-observations.json"),
        serde_json::to_vec_pretty(&pixels).unwrap(),
    )
    .unwrap();
    let original_png = fs::read(&output).unwrap();
    assert_eq!(
        run("existing-output", valid.clone(), 5)["error"]["code"],
        "revision_conflict"
    );
    assert_eq!(fs::read(&output).unwrap(), original_png);

    for (label, mutate, exit, code) in [
        ("target-pin", 0, 5, "revision_conflict"),
        ("position", 1, 2, "invalid_argument"),
        ("frame-pin", 2, 5, "revision_conflict"),
        ("template-pin", 3, 5, "revision_conflict"),
        ("review-ack", 4, 3, "needs_input"),
    ] {
        let path = root.join(format!("{label}.png"));
        let mut input = fixture.input(&path);
        match mutate {
            0 => input.target.output_sha256 = "0".repeat(64),
            1 => input.position = DecimalU64(1),
            2 => input.frame_sha256 = "0".repeat(64),
            3 => input.template.declaration_sha256 = "0".repeat(64),
            4 => input.review_artifact_only = false,
            _ => unreachable!(),
        }
        assert_eq!(run(label, input, exit)["error"]["code"], code);
        assert!(!path.exists(), "failed request must not publish {label}");
    }
    let invalid_frame = root.join("invalid-frame.png");
    fs::write(&invalid_frame, b"not-a-png").unwrap();
    let invalid_output = root.join("inspection-failure.png");
    let mut invalid = fixture.input(&invalid_output);
    invalid.frame_path = invalid_frame.to_str().unwrap().into();
    invalid.frame_sha256 = hash(b"not-a-png");
    assert_eq!(
        run("inspection-failure", invalid, 2)["error"]["code"],
        "invalid_argument"
    );
    assert!(!invalid_output.exists());
    let overflow_path = root.join("layout-failure.png");
    let mut overflow = fixture.input(&overflow_path);
    overflow.target = fixture.overflow.clone();
    assert_eq!(
        run("layout-failure", overflow, 3)["error"]["code"],
        "needs_input"
    );
    assert!(!overflow_path.exists());
    let deadline_path = root.join("deadline-unsupported.png");
    let deadline = invocation::invoke(
        &executable,
        &root,
        &fixture.store_path,
        "deadline-unsupported",
        fixture.input(&deadline_path),
        RequestContext {
            timeout_ms: Some(1),
            ..RequestContext::default()
        },
        3,
    );
    assert_eq!(deadline["error"]["code"], "unsupported");
    assert!(!deadline_path.exists());
    let after = fixture::store_snapshot(&fixture.store_path);
    fs::write(
        root.join("store-after.json"),
        serde_json::to_vec_pretty(&after).unwrap(),
    )
    .unwrap();
    let target_after = fixture.logical_snapshot();
    fs::write(root.join("accepted-after.json"), &target_after).unwrap();
    assert_eq!(before, after, "export must not mutate any owned store file");
    assert_eq!(target_before, target_after);
    assert_eq!(frame_before, fs::read(&fixture.frame).unwrap());
    assert_eq!(original_png, fs::read(&output).unwrap());
}
