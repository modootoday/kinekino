#![cfg(windows)]
#[allow(dead_code)]
#[path = "support/owned_process_evidence.rs"]
mod owned_process_evidence;
use kinekino_player::{
    application::{
        api::{Request, RequestContext},
        proposals::{
            self, AcceptProposal, CreateProposal, InspectProposal,
            subtitle_export::{COMMAND, ExportAcceptedSubtitles},
        },
    },
    domain::{
        api_values::DecimalU64,
        proposals::{
            CueSourceSnapshot, GeneratedResult, ProposalTask, SourceCue, SourceSnapshot,
            output_review::OutputReviewTarget,
        },
        subtitles::SubtitleFormat,
    },
    infrastructure::{content_digest, proposals::ProposalFiles},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

// Same adapter as download_cli: unchanged shared inline tests expect this parent function.
fn hash(bytes: &[u8]) -> String {
    content_digest::sha256(&mut std::io::Cursor::new(bytes), bytes.len() as u64)
        .unwrap()
        .1
}

fn accepted(store: &ProposalFiles, id: &str, times: &[(u64, u64)]) -> OutputReviewTarget {
    let source = SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: format!("{id}-source"),
        source_revision: DecimalU64(1),
        source_sha256: "a".repeat(64),
        cues: times
            .iter()
            .enumerate()
            .map(|(index, &(start, end))| SourceCue {
                cue_id: format!("cue-{index}"),
                text: format!("원문-{index}"),
                start_us: Some(DecimalU64(start)),
                end_us: Some(DecimalU64(end)),
            })
            .collect(),
    });
    proposals::create(
        store,
        CreateProposal {
            proposal_id: id.into(),
            source,
            generated: GeneratedResult {
                task: ProposalTask::Correction,
                raw_text:
                    json!({"task":"correction","edits":[{"cue_id":"cue-0","text":"교정 & <보존>"}]})
                        .to_string(),
            },
        },
    )
    .unwrap();
    proposals::accept(
        store,
        AcceptProposal {
            proposal_id: id.into(),
            target_source_revision: DecimalU64(1),
        },
        1,
    )
    .unwrap();
    let state = proposals::inspect(
        store,
        InspectProposal {
            proposal_id: id.into(),
        },
    )
    .unwrap();
    OutputReviewTarget::from_stored(&state).unwrap()
}
fn invoke(
    executable: &Path,
    root: &Path,
    store: &Path,
    label: &str,
    input: ExportAcceptedSubtitles,
    expected_exit: i32,
) -> Value {
    let mut request = Request::new(COMMAND);
    request.context = RequestContext {
        request_id: label.into(),
        ..RequestContext::default()
    };
    request.payload = serde_json::to_value(input).unwrap();
    let input_path = root.join(format!("{label}.request.json"));
    fs::write(&input_path, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    let call_root = root.join(label);
    fs::create_dir(&call_root).unwrap();
    let output = owned_process_evidence::run_process_with_arguments(
        executable.to_path_buf(),
        &call_root,
        &[],
        vec![
            "--input".into(),
            input_path.into_os_string(),
            "--proposal-root".into(),
            store.as_os_str().into(),
            "--json".into(),
        ],
        Duration::from_secs(15),
        false,
    );
    let output = match output {
        Ok(output) => output,
        Err(failure) => {
            // Retain the original owner/first cause before fallible diagnostics or unwind.
            // No fresh deadline, second finish, or replacement process is introduced.
            let retained = Box::leak(failure);
            let diagnostic = owned_process_evidence::write_json(
                &call_root.join("failure.json"),
                &retained.observation(),
            );
            panic!("original CLI failure retained: {retained:?}; diagnostic={diagnostic:?}");
        }
    };
    fs::write(root.join(format!("{label}.stdout.json")), &output.stdout).unwrap();
    fs::write(root.join(format!("{label}.stderr.raw")), &output.stderr).unwrap();
    fs::write(
        root.join(format!("{label}.status.json")),
        serde_json::to_vec(&json!({"receipt":owned_process_evidence::receipt_json(&output.receipt),"elapsed_ms":output.elapsed_ms,"stdout":output.stdout_evidence,"stderr":output.stderr_evidence}))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        output.receipt.child_exit,
        Some(expected_exit as u32),
        "{label}: original response preserved"
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["request_id"], label);
    response
}
fn request(
    target: &OutputReviewTarget,
    path: &Path,
    format: SubtitleFormat,
    allow_loss: bool,
) -> ExportAcceptedSubtitles {
    ExportAcceptedSubtitles {
        target: target.clone(),
        output_path: path.to_str().unwrap().into(),
        output_format: format,
        allow_loss,
    }
}

#[test]
#[ignore = "Explicit fixed CLI and fresh private fixture root; OS/CLI allocation required"]
fn accepted_subtitle_export_cli_preserves_binding_loss_and_new_file_boundaries() {
    kinekino_platform::install();
    let root = PathBuf::from(
        std::env::var_os("ACCEPTED_SUBTITLE_EXPORT_EVIDENCE_DIR")
            .expect("explicit fresh evidence root required"),
    );
    let executable = PathBuf::from(
        std::env::var_os("ACCEPTED_SUBTITLE_EXPORT_CLI").expect("explicit fixed CLI required"),
    );
    let expected_sha =
        std::env::var("ACCEPTED_SUBTITLE_EXPORT_CLI_SHA256").expect("explicit CLI SHA required");
    assert!(root.is_absolute() && !root.exists());
    assert!(executable.is_absolute());
    let (_, actual_sha) =
        content_digest::sha256(&mut fs::File::open(&executable).unwrap(), 512 * 1024 * 1024)
            .unwrap();
    assert!(actual_sha.eq_ignore_ascii_case(&expected_sha));
    fs::create_dir(&root).unwrap();
    let store_path = root.join("store");
    fs::create_dir(&store_path).unwrap();
    let store = ProposalFiles::open(&store_path).unwrap();
    let plain = accepted(
        &store,
        "cli-plain",
        &[(9_000_000, 10_000_000), (1_000_000, 2_000_000)],
    );
    let maximum = accepted(&store, "cli-max", &[(u64::MAX - 5_000, u64::MAX)]);
    let collapsed = accepted(&store, "cli-collapse", &[(1001, 1999)]);
    let original_states: Vec<_> = [&plain, &maximum, &collapsed]
        .into_iter()
        .map(|target| {
            serde_json::to_vec(
                &proposals::inspect(
                    &store,
                    InspectProposal {
                        proposal_id: target.proposal_id.clone(),
                    },
                )
                .unwrap(),
            )
            .unwrap()
        })
        .collect();
    let srt = root.join("plain.srt");
    let response = invoke(
        &executable,
        &root,
        &store_path,
        "plain-srt",
        request(&plain, &srt, SubtitleFormat::Srt, false),
        0,
    );
    assert_eq!(response["ok"], true);
    assert_eq!(
        response["data"]["target"],
        serde_json::to_value(&plain).unwrap()
    );
    let expected = "1\n00:00:09,000 --> 00:00:10,000\n교정 &amp; &lt;보존&gt;\n\n2\n00:00:01,000 --> 00:00:02,000\n원문-1\n\n";
    assert_eq!(fs::read_to_string(&srt).unwrap(), expected);
    let (_, output_sha) =
        content_digest::sha256(&mut fs::File::open(&srt).unwrap(), 8 * 1024 * 1024).unwrap();
    assert_eq!(response["data"]["output_sha256"], output_sha);
    assert_eq!(response["data"]["overwrite"], false);
    let vtt = root.join("plain.vtt");
    assert_eq!(
        invoke(
            &executable,
            &root,
            &store_path,
            "plain-vtt",
            request(&plain, &vtt, SubtitleFormat::Vtt, false),
            0
        )["ok"],
        true
    );
    assert!(
        fs::read_to_string(vtt)
            .unwrap()
            .contains("00:00:09.000 --> 00:00:10.000")
    );
    let conflict = invoke(
        &executable,
        &root,
        &store_path,
        "existing-output",
        request(&plain, &srt, SubtitleFormat::Srt, false),
        5,
    );
    assert_eq!(conflict["error"]["code"], "revision_conflict");
    assert_eq!(fs::read_to_string(&srt).unwrap(), expected);
    let max_vtt = root.join("maximum.vtt");
    let declined = invoke(
        &executable,
        &root,
        &store_path,
        "precision-unconfirmed",
        request(&maximum, &max_vtt, SubtitleFormat::Vtt, false),
        3,
    );
    assert_eq!(declined["error"]["code"], "needs_input");
    assert_eq!(
        declined["error"]["details"]["losses"][0]["code"],
        "timestamp_precision_reduced"
    );
    assert!(!max_vtt.exists());
    let acknowledged = invoke(
        &executable,
        &root,
        &store_path,
        "precision-acknowledged",
        request(&maximum, &max_vtt, SubtitleFormat::Vtt, true),
        0,
    );
    assert_eq!(
        acknowledged["data"]["losses"][0]["code"],
        "timestamp_precision_reduced"
    );
    assert!(
        fs::read_to_string(max_vtt)
            .unwrap()
            .contains("5124095576:01:49.546 --> 5124095576:01:49.551")
    );
    let collapsed_path = root.join("collapsed.srt");
    let rejected = invoke(
        &executable,
        &root,
        &store_path,
        "collapsed-even-with-loss",
        request(&collapsed, &collapsed_path, SubtitleFormat::Srt, true),
        2,
    );
    assert_eq!(rejected["error"]["code"], "invalid_argument");
    assert!(!collapsed_path.exists());
    for (target, original) in [&plain, &maximum, &collapsed]
        .into_iter()
        .zip(original_states)
    {
        assert_eq!(
            serde_json::to_vec(
                &proposals::inspect(
                    &store,
                    InspectProposal {
                        proposal_id: target.proposal_id.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            original
        );
    }
}
