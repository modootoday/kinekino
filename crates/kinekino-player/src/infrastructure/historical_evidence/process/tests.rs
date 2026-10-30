use super::*;
use std::io::Write;

#[test]
#[ignore = "subprocess fixture invoked only by process ownership tests"]
fn verifier_fault_child() {
    match std::env::var("PLAYER_HISTORICAL_PROCESS_FAULT")
        .unwrap()
        .as_str()
    {
        "stdout" => {
            let _ = std::io::stdout().write_all(&vec![b'x'; 2_097_153]);
            thread::sleep(Duration::from_secs(10));
        }
        "stderr" => {
            let _ = std::io::stderr().write_all(&vec![b'x'; 8_193]);
            thread::sleep(Duration::from_secs(10));
        }
        "timeout" => thread::sleep(Duration::from_secs(10)),
        "nonzero" => std::process::exit(23),
        "normal" => println!("fixture-complete"),
        _ => panic!("unknown explicit subprocess fixture"),
    }
}
fn command(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "infrastructure::historical_evidence::process::tests::verifier_fault_child",
            "--ignored",
            "--nocapture",
        ])
        .env("PLAYER_HISTORICAL_PROCESS_FAULT", mode);
    command
}
fn settle(runner: &Runner, expected: ErrorCode) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while runner.0.lock().unwrap().is_some() {
        assert!(
            Instant::now() < deadline,
            "cleanup must retain ownership until terminal"
        );
        thread::sleep(Duration::from_millis(10));
        let error = runner
            .run(command("normal"), Duration::from_secs(2), |_| {
                panic!("cleanup must not spawn another process")
            })
            .unwrap_err();
        assert_eq!(error.code, expected);
    }
}
#[test]
fn both_pipe_overflows_timeout_and_nonzero_preserve_error_and_reap() {
    for (mode, code) in [
        ("stdout", ErrorCode::ResourceExhausted),
        ("stderr", ErrorCode::ResourceExhausted),
        ("timeout", ErrorCode::Timeout),
        ("nonzero", ErrorCode::EngineFailed),
    ] {
        let runner = Runner::default();
        let timeout = if mode == "timeout" {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(5)
        };
        let error = runner
            .run(command(mode), timeout, |child| Box::new(child))
            .unwrap_err();
        assert_eq!(error.code, code, "{mode}");
        settle(&runner, code);
        assert!(runner.0.lock().unwrap().is_none());
    }
}
struct FaultControl {
    child: Child,
    kill_once: bool,
    wait_once: bool,
}
impl Control for FaultControl {
    fn id(&self) -> u32 {
        self.child.id()
    }
    fn poll(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }
    fn kill(&mut self) -> std::io::Result<()> {
        if std::mem::take(&mut self.kill_once) {
            return Err(std::io::ErrorKind::PermissionDenied.into());
        }
        self.child.kill()
    }
    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        if std::mem::take(&mut self.wait_once) {
            return Err(std::io::ErrorKind::PermissionDenied.into());
        }
        self.child.wait()
    }
}
#[test]
fn failed_kill_or_wait_retains_owner_and_retry_never_spawns_replacement() {
    for kill_once in [true, false] {
        let runner = Runner::default();
        let error = runner
            .run(
                command(if kill_once { "timeout" } else { "normal" }),
                if kill_once {
                    Duration::from_millis(100)
                } else {
                    Duration::from_secs(5)
                },
                |child| {
                    Box::new(FaultControl {
                        child,
                        kill_once,
                        wait_once: !kill_once,
                    })
                },
            )
            .unwrap_err();
        let code = if kill_once {
            ErrorCode::Timeout
        } else {
            ErrorCode::EngineFailed
        };
        assert_eq!(error.code, code);
        assert_eq!(error.details["cleanup_complete"], false);
        assert!(runner.0.lock().unwrap().is_some());
        settle(&runner, code);
    }
}
#[test]
fn pipe_limit_and_read_failure_are_distinct() {
    assert_eq!(bounded(&b"abc"[..], 3).unwrap(), b"abc");
    assert_eq!(
        bounded(&b"abcd"[..], 3).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
    }
    assert_eq!(
        bounded(Broken, 3).unwrap_err().code,
        ErrorCode::EngineFailed
    );
}
