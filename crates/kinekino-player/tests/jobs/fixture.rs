use kinekino_player::{
    application::api::{Request, RequestContext},
    infrastructure::{
        content_digest,
        local_host::{EndpointClient, TransportOptions},
    },
};
use serde_json::Value;
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
#[path = "fixture/request_evidence.rs"]
mod request_evidence;
use request_evidence::{request_link, request_settlement};

pub struct Fixture {
    pub root: PathBuf,
    pub cli: PathBuf,
    persistent: bool,
}
impl Fixture {
    pub(super) fn keep_evidence(&mut self) {
        self.persistent = true;
    }
    pub(super) fn try_call_until(
        &self,
        label: &str,
        request: &Request,
        endpoint: &str,
        original: kinekino_process::domain::deadline::Deadline,
        cap: std::time::Duration,
        exit: i32,
    ) -> Result<Value, Box<super::kinekino_process::CliFailure>> {
        use super::kinekino_process::{CliFailure, evidence};
        use std::io;
        let failed = |stage, error| CliFailure::not_spawned(stage, error, &self.root);
        let request_deadline = super::kinekino_process::phase_deadline(original, cap)
            .map_err(|error| failed("request phase budget", error))?;
        if !kinekino_player::domain::identifiers::valid_id(label) {
            return Err(failed(
                "request label",
                io::Error::other("Invalid owned request evidence label"),
            ));
        }
        let root = self.root.join("logs/owned-requests").join(label);
        fs::create_dir_all(root.parent().expect("label has parent"))
            .and_then(|_| fs::create_dir(&root))
            .map_err(|error| failed("request directory", error))?;
        let bytes = serde_json::to_vec(request)
            .map_err(|error| failed("request encode", io::Error::other(error)))?;
        fs::write(self.root.join(format!("logs/{label}.request.json")), &bytes)
            .map_err(|error| failed("request bytes", error))?;
        let link = request_link(label, request, endpoint, &bytes, exit)
            .map_err(|error| failed("request link", error))?;
        evidence::write_json(&root.join("attempt.json"), &link)
            .map_err(|error| failed("request attempt evidence", error))?;
        let output = match evidence::run_process_until(
            self.cli.clone(),
            &root,
            &bytes,
            Some(endpoint),
            request_deadline,
            cap,
            false,
        ) {
            Ok(output) => output,
            Err(mut error) => {
                let observation = error.observation();
                let actual = observation["owned_family"]["child_exit"].as_u64();
                let settled = request_settlement(&link, actual, false, Some(observation));
                if let Err(secondary) = evidence::write_json(&root.join("settled.json"), &settled) {
                    error
                        .secondary
                        .push(format!("request settlement evidence: {secondary}"));
                }
                return Err(error);
            }
        };
        let result: Result<Value, Box<CliFailure>> = (|| {
            let terminal = |stage, error| output.failure(stage, error, &root);
            fs::write(
                self.root.join(format!("logs/{label}.stdout")),
                &output.stdout,
            )
            .and_then(|_| {
                fs::write(
                    self.root.join(format!("logs/{label}.stderr")),
                    &output.stderr,
                )
            })
            .and_then(|_| {
                fs::write(
                    self.root.join(format!("logs/{label}.exit")),
                    format!(
                        "{:?}",
                        output
                            .receipt
                            .child_exit
                            .and_then(|code| i32::try_from(code).ok())
                    ),
                )
            })
            .map_err(|error| terminal("request legacy evidence", error))?;
            evidence::write_json(&root.join("process.json"), &serde_json::json!({"schema_version":2,"owned_family":evidence::receipt_json(&output.receipt),"request_bytes":bytes.len(),"elapsed_ms":output.elapsed_ms,"stdout":output.stdout_evidence,"stderr":output.stderr_evidence,"io_workers":[],"stdin_result":"not_applicable_file_transport"}))
            .map_err(|error|terminal("request receipt",error))?;
            if output.receipt.child_exit != u32::try_from(exit).ok() {
                return Err(terminal(
                    "request exit",
                    io::Error::other(format!(
                        "{label}: unexpected exit {:?}, expected {exit}",
                        output.receipt.child_exit
                    )),
                ));
            }
            if output.stdout_evidence["limit_exceeded"] == true
                || output.stderr_evidence["limit_exceeded"] == true
            {
                return Err(terminal(
                    "request stream bound",
                    io::Error::other("CLI response prefix exceeded limit"),
                ));
            }
            let value = serde_json::from_slice(&output.stdout)
                .map_err(|error| terminal("request JSON", io::Error::other(error)))?;
            output.check_deadline(&root)?;
            Ok(value)
        })();
        let failure = result.as_ref().err().map(|error| error.observation());
        let settled = request_settlement(
            &link,
            output.receipt.child_exit.map(u64::from),
            result.is_ok(),
            failure,
        );
        if let Err(secondary) = evidence::write_json(&root.join("settled.json"), &settled) {
            return match result {
                Err(mut original) => {
                    original
                        .secondary
                        .push(format!("request settlement evidence: {secondary}"));
                    Err(original)
                }
                Ok(_) => Err(output.failure("request settlement evidence", secondary, &root)),
            };
        }
        if result.is_ok()
            && let Err(mut error) = output.check_deadline(&root)
        {
            let corrected = request_settlement(
                &link,
                output.receipt.child_exit.map(u64::from),
                false,
                Some(error.observation()),
            );
            if let Err(secondary) = evidence::write_json(&root.join("settled.json"), &corrected) {
                error
                    .secondary
                    .push(format!("late request settlement evidence: {secondary}"));
            }
            return Err(error);
        }
        result
    }
    pub fn new() -> Self {
        Self::named("lifecycle")
    }
    pub fn named(name: &str) -> Self {
        let configured =
            std::env::var_os("JOB_EVIDENCE_DIR").map(|root| PathBuf::from(root).join(name));
        let persistent = configured.is_some();
        let root = configured.unwrap_or_else(|| {
            // Windows clocks tick in 100 ns, so parallel fixtures can read the same instant;
            // the per-process sequence keeps every root unique.
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            std::env::temp_dir().join(format!(
                "player-jobs-cli-{}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
        });
        assert!(root.is_absolute());
        if let Some(parent) = root.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::create_dir(&root).unwrap();
        for directory in ["logs", "store", "forced-store"] {
            fs::create_dir(root.join(directory)).unwrap();
        }
        let source = std::env::var_os("JOB_CLI_EXE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(crate::cli_exe::kinekino_cli_exe()));
        let digest = |path: &Path| {
            content_digest::sha256(&mut fs::File::open(path).unwrap(), 1024 * 1024 * 1024)
                .unwrap()
                .1
        };
        let before = digest(&source);
        if let Ok(expected) = std::env::var("JOB_CLI_SHA256") {
            assert_eq!(before, expected);
        }
        let cli = root.join(kinekino_player::domain::product::PRODUCT.cli_executable);
        fs::copy(&source, &cli).unwrap();
        assert_eq!(digest(&source), before);
        assert_eq!(digest(&cli), before);
        fs::write(root.join("cli-sha256.txt"), before).unwrap();
        crate::fixture_plugin::install_fixture_plugin(&root, &source);
        Self {
            root,
            cli,
            persistent,
        }
    }
    pub fn call(
        &self,
        label: &str,
        request: &Request,
        endpoint: Option<&str>,
        root: Option<&Path>,
        exit: i32,
    ) -> Value {
        self.call_root(
            label,
            request,
            endpoint,
            root.map(|root| ("--job-root", root)),
            exit,
        )
    }
    pub fn call_catalog(&self, label: &str, request: &Request, root: &Path, exit: i32) -> Value {
        self.call_root(label, request, None, Some(("--catalog-root", root)), exit)
    }
    fn call_root(
        &self,
        label: &str,
        request: &Request,
        endpoint: Option<&str>,
        root: Option<(&str, &Path)>,
        exit: i32,
    ) -> Value {
        let bytes = serde_json::to_vec(request).unwrap();
        fs::write(self.root.join(format!("logs/{label}.request.json")), &bytes).unwrap();
        let mut command = Command::new(&self.cli);
        command
            .creation_flags(0x08000000)
            .env("KINEKINO_PLUGIN_DIR", self.root.join("plugins"))
            .args(["--input", "-"]);
        if let Some(endpoint) = endpoint {
            command.args(["--endpoint", endpoint]);
        }
        if let Some((option, root)) = root {
            command.arg(option).arg(root);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&bytes).unwrap();
        let output = child.wait_with_output().unwrap();
        fs::write(
            self.root.join(format!("logs/{label}.stdout")),
            &output.stdout,
        )
        .unwrap();
        fs::write(
            self.root.join(format!("logs/{label}.stderr")),
            &output.stderr,
        )
        .unwrap();
        fs::write(
            self.root.join(format!("logs/{label}.exit")),
            format!("{:?}", output.status.code()),
        )
        .unwrap();
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{label}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    pub fn endpoint(&self, host: &Host) -> EndpointClient {
        EndpointClient::new(&host.endpoint, TransportOptions::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.persistent {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
pub struct Host {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    pub endpoint: String,
    log: PathBuf,
    active: bool,
    provider: bool,
}
impl Host {
    pub fn start(fixture: &Fixture, name: &str, root: &Path) -> Self {
        Self::start_inner(fixture, name, root, None, &[])
    }
    pub fn start_with_options(
        fixture: &Fixture,
        name: &str,
        root: &Path,
        options: &[String],
    ) -> Self {
        Self::start_inner(fixture, name, root, None, options)
    }
    pub fn start_provider(
        fixture: &Fixture,
        name: &str,
        root: &Path,
        catalog: &Path,
        profile: &Path,
    ) -> Self {
        Self::start_inner(fixture, name, root, Some((catalog, profile)), &[])
    }
    fn start_inner(
        fixture: &Fixture,
        name: &str,
        root: &Path,
        provider: Option<(&Path, &Path)>,
        options: &[String],
    ) -> Self {
        let mut command = Command::new(&fixture.cli);
        command
            .creation_flags(0x08000000)
            .env("KINEKINO_PLUGIN_DIR", fixture.root.join("plugins"))
            .arg(if provider.is_some() {
                "--serve-provider"
            } else {
                "--serve-jobs"
            })
            .arg("--job-root")
            .arg(root);
        command.args(options);
        if let Some((catalog, profile)) = provider {
            command
                .arg("--catalog-root")
                .arg(catalog)
                .arg("--browser-profile-root")
                .arg(profile);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(
                fs::File::create(fixture.root.join(format!("logs/{name}-host.stderr"))).unwrap(),
            ))
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        fs::write(
            fixture.root.join(format!("logs/{name}-ready.stdout")),
            &line,
        )
        .unwrap();
        let ready: Value = serde_json::from_str(&line).unwrap();
        if ready["event"]
            != if provider.is_some() {
                "provider_host_ready"
            } else {
                "job_host_ready"
            }
        {
            drop(input);
            let status = child.wait().unwrap();
            panic!("host failed: {ready} {status}");
        }
        Self {
            child,
            input,
            output,
            endpoint: ready["endpoint"].as_str().unwrap().into(),
            log: fixture.root.join(format!("logs/{name}-stop.stdout")),
            active: true,
            provider: provider.is_some(),
        }
    }
    pub fn stop(mut self) {
        drop(self.input.take());
        let status = self.child.wait().unwrap();
        self.active = false;
        let mut rest = String::new();
        self.output.read_to_string(&mut rest).unwrap();
        fs::write(&self.log, &rest).unwrap();
        assert!(status.success(), "{rest}");
        let stopped: Value = serde_json::from_str(rest.trim()).unwrap();
        assert_eq!(
            stopped["event"],
            if self.provider {
                "provider_host_stopped"
            } else {
                "job_host_stopped"
            }
        );
        if self.provider {
            assert_eq!(stopped["receipt"]["browser_worker_exited"], true);
        }
    }
    pub fn kill_owned(mut self) {
        self.child.kill().unwrap();
        let status = self.child.wait().unwrap();
        self.active = false;
        fs::write(&self.log, format!("owned forced stop: {status}")).unwrap();
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        if self.active {
            drop(self.input.take());
            if self.child.try_wait().ok().flatten().is_none() {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
        }
    }
}
pub fn request(command: &str, payload: Value) -> Request {
    Request {
        command: command.into(),
        payload,
        context: RequestContext {
            request_id: "jobs-fixture".into(),
            ..Default::default()
        },
    }
}
