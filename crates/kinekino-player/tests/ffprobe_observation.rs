#![cfg(windows)]
//! Fixed local tool observations, separate from owned media and product support.
#[path = "../src/infrastructure/immutable_storage_paths.rs"]
pub mod immutable_storage_paths;
#[path = "support/owned_process_evidence.rs"]
pub mod owned_process_evidence;

use kinekino_process::domain::observation::FamilyObservation;
use owned_process_evidence::{CliFailure, run_process_with_arguments, write_json};
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const TOOL: &str = "C:/ProgramData/chocolatey/lib/ffmpeg/tools/ffmpeg/bin/ffprobe.exe";
const TOOL_BYTES: u64 = 82_620_928;
const TOOL_SHA: &str = "efb014c8f39091c90f3616154fe20ab8470f5f943f6a48ec4fd30c479c7dd697";
const MEDIA: &str = "D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/results/production-export-76968/portrait.mp4";
const MEDIA_BYTES: u64 = 932_830;
const MEDIA_SHA: &str = "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483";

// Existing mounted helper's inline contract tests use this root hash adapter.
pub fn hash(bytes: &[u8]) -> String {
    kinekino_player::infrastructure::content_digest::sha256(
        &mut std::io::Cursor::new(bytes),
        bytes.len() as u64,
    )
    .unwrap()
    .1
}
fn pin(path: &str, bytes: u64, sha256: &str, guards: &mut Vec<fs::File>) {
    let path = Path::new(path);
    let (_, mut ancestors) = immutable_storage_paths::root(path.parent().unwrap()).unwrap();
    guards.append(&mut ancestors);
    let mut file = immutable_storage_paths::file(path).unwrap();
    assert_eq!(file.metadata().unwrap().len(), bytes);
    let observed =
        kinekino_player::infrastructure::content_digest::sha256(&mut file, bytes).unwrap();
    assert_eq!(observed, (bytes, sha256.to_owned()));
    guards.push(file);
}
#[derive(Clone, Copy)]
enum Observation {
    Version,
    Configuration,
    License,
    Protocols,
    Metadata,
}
impl Observation {
    fn label(&self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::Configuration => "configuration",
            Self::License => "license",
            Self::Protocols => "protocols",
            Self::Metadata => "portrait-metadata",
        }
    }
    fn arguments(&self) -> Vec<OsString> {
        let args = match self {
            Self::Version => vec!["-version"],
            Self::Configuration => vec!["-buildconf"],
            Self::License => vec!["-L"],
            Self::Protocols => vec!["-v", "error", "-protocols"],
            Self::Metadata => vec![
                "-v",
                "error",
                "-protocol_whitelist",
                "file",
                "-f",
                "mov",
                "-enable_drefs",
                "0",
                "-show_entries",
                "stream=index,codec_type,codec_name,width,height,channels,sample_rate:format=format_name,duration,size",
                "-of",
                "json",
                "-i",
                MEDIA,
            ],
        };
        args.into_iter().map(OsString::from).collect()
    }
}
struct FailedObservation {
    failure: Box<CliFailure>,
    _original_guards: Vec<fs::File>,
}
fn exercise(observation: Observation) {
    let parent = PathBuf::from(
        std::env::var_os("FFPROBE_OBSERVATION_EVIDENCE_ROOT")
            .expect("Allocated existing evidence parent required"),
    );
    let (_, mut guards) = immutable_storage_paths::root(&parent).unwrap();
    pin(TOOL, TOOL_BYTES, TOOL_SHA, &mut guards);
    if matches!(observation, Observation::Metadata) {
        pin(MEDIA, MEDIA_BYTES, MEDIA_SHA, &mut guards);
    }
    let root = parent.join(observation.label());
    fs::create_dir(&root).unwrap();
    let arguments = observation.arguments();
    let total = if matches!(observation, Observation::Metadata) {
        15
    } else {
        10
    };
    write_json(&root.join("request.json"), &json!({
        "executable":TOOL,"tool_sha256":TOOL_SHA,"tool_bytes":TOOL_BYTES,
        "arguments":arguments.iter().map(|value| value.to_str().unwrap()).collect::<Vec<_>>(),
        "total_ms":total * 1000,"cleanup_reserve_ms":2000,
        "pin_admission_excluded_from_child_deadline":true,
        "media":if matches!(observation, Observation::Metadata) { json!({"path":MEDIA,"bytes":MEDIA_BYTES,"sha256":MEDIA_SHA}) } else { Value::Null },
    })).unwrap();
    let output = match run_process_with_arguments(
        PathBuf::from(TOOL),
        &root,
        b"",
        arguments,
        Duration::from_secs(total),
        true,
    ) {
        Ok(output) => output,
        Err(failure) => {
            // Keep the original normal-finish or partial-launch failure intact.
            // Retain the WHOLE original failure and guards before any diagnostic.
            // This is an incomplete test-process boundary, never a cleanup ACK.
            let retained = Box::leak(Box::new(FailedObservation {
                failure,
                _original_guards: guards,
            }));
            let result = write_json(
                &root.join("failure-final.json"),
                &retained.failure.observation(),
            );
            panic!(
                "Original observation failure retained: {:?}; diagnostic={result:?}",
                retained.failure
            );
        }
    };
    // Save actual process/raw/receipt observations before semantic assertions.
    write_json(&root.join("observation.json"), &json!({
        "receipt":owned_process_evidence::receipt_json(&output.receipt),
        "stdout":output.stdout_evidence,"stderr":output.stderr_evidence,
        "elapsed_ms":output.elapsed_ms,"semantic_support_verdict":"not_assigned_by_process_success",
    })).unwrap();
    output.check_deadline(&root).unwrap();
    assert_eq!(output.receipt.child_exit, Some(0));
    assert_eq!(output.receipt.family, FamilyObservation::Empty);
    assert!(!output.receipt.forced);
    assert!(output.receipt.stdin_eof_requested);
    for stream in [&output.stdout_evidence, &output.stderr_evidence] {
        assert_eq!(stream["complete"], true);
        assert_eq!(stream["limit_exceeded"], false);
    }
    assert!(!output.stdout.is_empty() || !output.stderr.is_empty());
    match observation {
        Observation::Metadata => {
            let metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
            write_json(&root.join("observed-metadata.json"), &metadata).unwrap();
            // Track and duration pins belong to a separate post-observation review.
        }
        Observation::Protocols => {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let lines: Vec<_> = stdout.lines().collect();
            let input = lines.iter().position(|line| line.trim() == "Input:");
            let output = lines.iter().position(|line| line.trim() == "Output:");
            let input_fd = match (input, output) {
                (Some(start), Some(end)) if start < end => {
                    Some(lines[start + 1..end].iter().any(|line| line.trim() == "fd"))
                }
                _ => None,
            };
            write_json(
                &root.join("observed-capability.json"),
                &json!({
                    "input_fd_observed":input_fd,"media_validity_proven":false,
                    "unsupported_fd_is_not_replaced_with_pipe_or_path":true,
                }),
            )
            .unwrap();
        }
        _ => {}
    }
    // The original immutable guards remain alive through receipt and raw checks.
    drop(guards);
}
#[test]
#[ignore = "Allocated fixed local ffprobe observation gate only"]
fn ffprobe_version_owned() {
    kinekino_platform::install();
    exercise(Observation::Version);
}
#[test]
#[ignore = "Allocated fixed local ffprobe observation gate only"]
fn ffprobe_build_configuration_owned() {
    kinekino_platform::install();
    exercise(Observation::Configuration);
}
#[test]
#[ignore = "Allocated fixed local ffprobe observation gate only"]
fn ffprobe_license_owned() {
    kinekino_platform::install();
    exercise(Observation::License);
}
#[test]
#[ignore = "Allocated fixed local ffprobe observation gate only"]
fn ffprobe_protocols_owned() {
    kinekino_platform::install();
    exercise(Observation::Protocols);
}
#[test]
#[ignore = "Allocated fixed local ffprobe and existing pinned portrait only"]
fn ffprobe_existing_portrait_metadata_owned() {
    kinekino_platform::install();
    exercise(Observation::Metadata);
}
mod infrastructure {
    pub(crate) use kinekino_player::infrastructure::os_policy;
}
