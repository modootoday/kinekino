//! Snapshot observations; child-family ownership is a separate adapter.
#[path = "owned_process_evidence.rs"]
mod owned_process_evidence;
use kinekino_player::{application::api::Request, infrastructure::content_digest};
pub use owned_process_evidence::CliFailure;
use owned_process_evidence::{receipt_json, run_process, write_json};
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, Cursor, Read},
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
pub fn hash(bytes: &[u8]) -> String {
    content_digest::sha256(
        &mut Cursor::new(bytes),
        kinekino_player::domain::frame_capture::MAX_FRAME_PNG_BYTES,
    )
    .unwrap()
    .1
}
static EVIDENCE_RUN: OnceLock<Result<PathBuf, String>> = OnceLock::new();
static NEXT_CALL: AtomicUsize = AtomicUsize::new(0);
fn try_evidence_call() -> io::Result<PathBuf> {
    let run = EVIDENCE_RUN
        .get_or_init(|| {
            let create = || -> io::Result<PathBuf> {
                let root = std::env::var_os("KINEKINO_SNAPSHOT_CLI_EVIDENCE_ROOT")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
                            .join("runtime/quality-review/snapshot-export-cli-evidence")
                    });
                if !root.is_absolute() {
                    return Err(io::Error::other("Evidence root must be absolute"));
                }
                fs::create_dir_all(&root)?;
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(io::Error::other)?
                    .as_nanos();
                let run = root.join(format!("run-{}-{nanos}", std::process::id()));
                fs::create_dir(&run)?;
                Ok(run)
            };
            create().map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|error| io::Error::other(error.clone()))?;
    let call = run.join(format!(
        "call-{:04}",
        NEXT_CALL.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&call)?;
    Ok(call)
}
/// Utility used outside process ownership; process calls use the fallible boundary.
pub fn evidence_call() -> PathBuf {
    try_evidence_call().expect("Create snapshot evidence directory")
}
pub fn save_json(root: &Path, name: &str, value: &Value) {
    write_json(&root.join(name), value).expect("Save snapshot workflow evidence");
}
fn capture_file(root: &Path, name: &str, path: Option<&str>) -> io::Result<Value> {
    let Some(path) = path else {
        return Ok(json!({"state":"not_applicable"}));
    };
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(json!({"state":"absent","path":path}));
        }
        Err(error) => {
            return Ok(json!({"state":"read_error","path":path,"error":error.to_string()}));
        }
    };
    let maximum = kinekino_player::domain::frame_capture::MAX_FRAME_PNG_BYTES + 1;
    let length = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            return Ok(
                json!({"state":"read_error","stage":"metadata","path":path,"error":error.to_string()}),
            );
        }
    };
    let mut bytes = Vec::new();
    if let Err(error) = file.take(maximum).read_to_end(&mut bytes) {
        return Ok(json!({"state":"read_error","path":path,"error":error.to_string()}));
    }
    fs::write(root.join(name), &bytes)?;
    let digest = content_digest::sha256(&mut Cursor::new(&bytes), maximum)
        .map_err(|error| io::Error::other(format!("{error:?}")))?
        .1;
    Ok(
        json!({"state":"present","path":path,"observed_length":length,"preserved_file":name,"preserved_bytes":bytes.len(),"preserved_sha256":digest,"complete":length==bytes.len() as u64}),
    )
}
pub struct HostObservation<'a> {
    pub injected: bool,
    pub calls: Option<&'a AtomicUsize>,
}
pub struct CliOptions {
    pub wall_timeout: Duration,
    pub clear_path: bool,
}
impl Default for CliOptions {
    fn default() -> Self {
        Self {
            wall_timeout: Duration::from_secs(20),
            clear_path: true,
        }
    }
}
pub struct CliObservation {
    pub code: i32,
    pub response: Value,
    pub evidence: PathBuf,
}
impl CliObservation {
    fn into_tuple(self) -> (i32, Value) {
        (self.code, self.response)
    }
}
pub fn cli(request: &Request, endpoint: Option<&str>) -> Result<(i32, Value), Box<CliFailure>> {
    cli_observed(request, endpoint, None)
}
pub fn cli_observed(
    request: &Request,
    endpoint: Option<&str>,
    host: Option<HostObservation<'_>>,
) -> Result<(i32, Value), Box<CliFailure>> {
    run(request, endpoint, host, CliOptions::default()).map(CliObservation::into_tuple)
}
pub fn cli_recorded(
    request: &Request,
    endpoint: Option<&str>,
    options: CliOptions,
) -> Result<CliObservation, Box<CliFailure>> {
    run(request, endpoint, None, options)
}
fn source_name(request: &Request, phase: &str) -> String {
    let extension = if request.payload["source_path"]
        .as_str()
        .is_some_and(|path| path.to_ascii_lowercase().ends_with(".png"))
    {
        "png"
    } else {
        "bin"
    };
    format!("source.{phase}.{extension}")
}
fn run(
    request: &Request,
    endpoint: Option<&str>,
    host: Option<HostObservation<'_>>,
    options: CliOptions,
) -> Result<CliObservation, Box<CliFailure>> {
    let evidence = try_evidence_call()
        .map_err(|error| CliFailure::not_spawned("evidence directory", error, Path::new("")))?;
    let before = |stage, error| CliFailure::not_spawned(stage, error, &evidence);
    let request_bytes = serde_json::to_vec(request)
        .map_err(|error| before("serialize request", io::Error::other(error)))?;
    fs::write(evidence.join("request.json"), &request_bytes)
        .map_err(|error| before("request evidence", error))?;
    let request_sha256 = content_digest::sha256(
        &mut Cursor::new(&request_bytes),
        kinekino_player::domain::frame_capture::MAX_FRAME_PNG_BYTES,
    )
    .map_err(|error| before("request digest", io::Error::other(format!("{error:?}"))))?
    .1;
    let source_before = capture_file(
        &evidence,
        &source_name(request, "before"),
        request.payload["source_path"].as_str(),
    )
    .map_err(|error| before("source before evidence", error))?;
    let output_before = capture_file(
        &evidence,
        "output.before.png",
        request.payload["output_path"].as_str(),
    )
    .map_err(|error| before("output before evidence", error))?;
    let calls = || {
        host.as_ref()
            .and_then(|host| host.calls)
            .map(|calls| calls.load(Ordering::SeqCst))
    };
    write_json(&evidence.join("before.json"), &json!({"source":source_before,"output":output_before,"endpoint":endpoint,"host_port_injected":host.as_ref().map(|host|host.injected),"host_calls":calls()})).map_err(|error| before("before evidence", error))?;
    let process = run_process(
        PathBuf::from(crate::cli_exe::kinekino_cli_exe()),
        &evidence,
        &request_bytes,
        endpoint,
        options.wall_timeout,
        options.clear_path,
    );
    let mut secondary = Vec::new();
    let mut after_file = |name: &str, path| match capture_file(&evidence, name, path) {
        Ok(value) => value,
        Err(error) => {
            secondary.push(format!("{name}: {error}"));
            json!({"state":"preservation_error","error":error.to_string()})
        }
    };
    let source_after = after_file(
        &source_name(request, "after"),
        request.payload["source_path"].as_str(),
    );
    let output_after = after_file("output.after.png", request.payload["output_path"].as_str());
    if let Err(error) = write_json(
        &evidence.join("after.json"),
        &json!({"source":source_after,"output":output_after,"host_port_injected":host.as_ref().map(|host|host.injected),"host_calls":calls()}),
    ) {
        secondary.push(format!("after evidence: {error}"));
    }
    let mut process_json = json!({"schema_version":2,"fixture_timeout_ms":options.wall_timeout.as_millis(),"operation_timeout_ms":options.wall_timeout.checked_sub(owned_process_evidence::CLEANUP_RESERVE).map(|value|value.as_millis()),"cleanup_reserve_ms":2000,"child_path_cleared":options.clear_path,"request_timeout_ms":request.context.timeout_ms,"request_sha256":request_sha256,"endpoint":endpoint,"stdin_result":"not_applicable_file_transport","io_workers":[],"kill_result":"not_applicable_owned_job","descendants_observed":false});
    match &process {
        Ok(output) => {
            process_json["child_pid"] = json!(output.receipt.identity.pid);
            process_json["owned_family"] = receipt_json(&output.receipt);
            process_json["owned_job_termination"] = json!({"requested":output.receipt.forced});
            process_json["terminal_observed"] = json!(output.receipt.child_exit.is_some());
            process_json["fixture_timed_out"] = json!(output.receipt.forced);
            process_json["exit_code"] = json!(output.receipt.child_exit);
            process_json["reap_unresolved"] = json!(false);
            process_json["wait_error"] = Value::Null;
            process_json["reap_error"] = Value::Null;
            process_json["elapsed_ms"] = json!(output.elapsed_ms);
            process_json["stdout"] = output.stdout_evidence.clone();
            process_json["stderr"] = output.stderr_evidence.clone();
        }
        Err(error) => {
            process_json["failure"] = error.observation();
            let observation = error.observation();
            process_json["child_pid"] = observation["owned_family"]["identity"]["pid"].clone();
            process_json["owned_family"] = observation["owned_family"].clone();
            process_json["terminal_observed"] =
                json!(!observation["owned_family"]["child_exit"].is_null());
            process_json["exit_code"] = observation["owned_family"]["child_exit"].clone();
            process_json["fixture_timed_out"] = observation["owned_family"]["forced"].clone();
            process_json["owned_job_termination"] =
                json!({"requested":observation["owned_family"]["forced"]});
            process_json["reap_unresolved"] =
                json!(observation["ownership"] == "live_owner_retained");
            process_json["stdout"] = observation["streams"]["stdout"].clone();
            process_json["stderr"] = observation["streams"]["stderr"].clone();
        }
    }
    if let Err(error) = write_json(&evidence.join("process.json"), &process_json) {
        secondary.push(format!("process evidence: {error}"));
    }
    let output = match process {
        Ok(output) => output,
        Err(mut error) => {
            error.secondary.extend(secondary);
            return Err(error);
        }
    };
    if !secondary.is_empty() {
        let mut error = output.failure(
            "post-process evidence",
            io::Error::other(secondary.remove(0)),
            &evidence,
        );
        error.secondary = secondary;
        return Err(error);
    }
    let (code, response) = parse_response(&output, &evidence)?;
    eprintln!(
        "snapshot CLI pid={} exit={code} request_id={}",
        output.receipt.identity.pid, response["request_id"]
    );
    eprintln!("snapshot CLI evidence={}", evidence.display());
    Ok(CliObservation {
        code,
        response,
        evidence,
    })
}
fn parse_response(
    output: &owned_process_evidence::ProcessOutput,
    evidence: &Path,
) -> Result<(i32, Value), Box<CliFailure>> {
    output.check_deadline(evidence)?;
    let invalid = |message| {
        output.failure(
            "CLI response",
            io::Error::new(io::ErrorKind::InvalidData, message),
            evidence,
        )
    };
    if !output.stderr.is_empty() {
        return Err(invalid("CLI stderr is not empty"));
    }
    if output.stdout.len() > kinekino_player::application::local_host::MAX_RESPONSE_BYTES {
        return Err(invalid("CLI response exceeds limit"));
    }
    if String::from_utf8_lossy(&output.stdout).lines().count() != 1 {
        return Err(invalid("CLI response must contain one JSON line"));
    }
    let response: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| output.failure("CLI JSON", io::Error::other(error), evidence))?;
    let code = output
        .receipt
        .child_exit
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| invalid("CLI exit code unavailable or outside i32"))?;
    output.check_deadline(evidence)?;
    Ok((code, response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_process::domain::{
        identity::ProcessIdentity,
        observation::{FamilyObservation, ProcessReceipt},
    };
    fn output(stdout: &[u8], exit: u32) -> owned_process_evidence::ProcessOutput {
        owned_process_evidence::ProcessOutput {
            deadline: kinekino_process::domain::deadline::Deadline::new(
                std::time::Instant::now(),
                Duration::from_secs(2),
                Duration::from_secs(1),
            )
            .unwrap(),
            receipt: ProcessReceipt {
                identity: ProcessIdentity {
                    pid: 123,
                    creation_filetime: Some(456),
                    executable: "fixture-only".into(),
                },
                child_exit: Some(exit),
                family: FamilyObservation::Empty,
                forced: false,
                stdin_eof_requested: true,
            },
            stdout: stdout.into(),
            stderr: Vec::new(),
            stdout_evidence: Value::Null,
            stderr_evidence: Value::Null,
            elapsed_ms: 0,
        }
    }
    #[test]
    fn missing_or_malformed_response_keeps_original_terminal_receipt() {
        kinekino_platform::install();
        for bytes in [
            b"".as_slice(),
            b"not-json\n".as_slice(),
            b"{}\n{}\n".as_slice(),
        ] {
            let error = parse_response(&output(bytes, 9), Path::new("fixture-only")).unwrap_err();
            assert!(
                matches!(error.ownership, owned_process_evidence::Ownership::ObservedTerminal(ref receipt) if receipt.child_exit == Some(9))
            );
        }
    }
    #[test]
    fn nonzero_cli_json_remains_an_error_response_not_an_invented_success() {
        kinekino_platform::install();
        let (code, response) = parse_response(
            &output(b"{\"ok\":false,\"error\":{\"code\":\"unsupported\"}}\n", 4),
            Path::new("fixture-only"),
        )
        .unwrap();
        assert_eq!(code, 4);
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "unsupported");
    }
    #[test]
    fn process_evidence_and_failure_record_errors_preserve_original_terminal_outcome() {
        kinekino_platform::install();
        let root = evidence_call();
        fs::create_dir(root.join("process.json")).unwrap();
        fs::create_dir(root.join("failure.json")).unwrap();
        let original =
            write_json(&root.join("process.json"), &json!({"state":"observed"})).unwrap_err();
        let failure = output(b"{}\n", 7)
            .failure("process evidence", original, &root)
            .closeout();
        assert_eq!(failure.stage, "process evidence");
        assert!(
            matches!(failure.ownership, owned_process_evidence::Ownership::ObservedTerminal(ref receipt) if receipt.child_exit == Some(7))
        );
        assert!(
            failure
                .secondary
                .iter()
                .any(|error| error.contains("failure evidence"))
        );
    }
}
