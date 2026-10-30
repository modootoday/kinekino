use super::*;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
#[derive(Default)]
struct Calls {
    kill: usize,
    wait: usize,
}
struct FakeChild {
    calls: Rc<RefCell<Calls>>,
    kill_error: bool,
    waits: VecDeque<bool>,
}
impl ChildProcess for FakeChild {
    fn id(&self) -> u32 {
        77
    }
    fn terminate(&mut self) -> io::Result<()> {
        self.calls.borrow_mut().kill += 1;
        if self.kill_error {
            Err(io::Error::other("kill denied"))
        } else {
            Ok(())
        }
    }
    fn wait(&mut self) -> io::Result<Option<i32>> {
        self.calls.borrow_mut().wait += 1;
        if self.waits.pop_front().unwrap_or(true) {
            Ok(Some(0))
        } else {
            Err(io::Error::other("wait unavailable"))
        }
    }
}
#[derive(Default)]
struct FakeSink {
    fail_start: bool,
    fail_terminal: bool,
    fail_failure: bool,
    failure_reaped: Option<bool>,
}
impl Sink for FakeSink {
    fn started(&mut self, _: u32, _: (usize, u32)) -> io::Result<()> {
        if self.fail_start {
            Err(io::Error::other("write start"))
        } else {
            Ok(())
        }
    }
    fn completed(&mut self, _: i32) -> io::Result<()> {
        if self.fail_terminal {
            Err(io::Error::other("write terminal"))
        } else {
            Ok(())
        }
    }
    fn failed(&mut self, failure: &Failure) -> io::Result<()> {
        self.failure_reaped = Some(failure.cleanup.reaped);
        if self.fail_failure {
            Err(io::Error::other("write failure"))
        } else {
            Ok(())
        }
    }
}
fn child() -> (FakeChild, Rc<RefCell<Calls>>) {
    let calls = Rc::new(RefCell::new(Calls::default()));
    (
        FakeChild {
            calls: calls.clone(),
            kill_error: false,
            waits: VecDeque::new(),
        },
        calls,
    )
}
fn budget(_: &FakeChild) -> io::Result<(usize, u32)> {
    Ok((3, 0x4000))
}
#[test]
fn started_write_failure_kills_and_waits_before_returning_original_error() {
    let (c, counts) = child();
    let mut sink = FakeSink {
        fail_start: true,
        ..FakeSink::default()
    };
    let error = supervise(c, budget, &mut sink).unwrap_err();
    assert_eq!(error.stage, "started_receipt");
    assert_eq!(error.original_error.to_string(), "write start");
    assert!(error.cleanup.reaped);
    assert_eq!(sink.failure_reaped, Some(true));
    assert_eq!((counts.borrow().kill, counts.borrow().wait), (1, 1));
}
#[test]
fn verify_and_kill_errors_are_both_preserved_even_when_wait_reaps() {
    let (mut c, counts) = child();
    c.kill_error = true;
    let error = supervise(
        c,
        |_| Err(io::Error::other("verify mismatch")),
        &mut FakeSink::default(),
    )
    .unwrap_err();
    assert_eq!(error.stage, "budget_verify");
    assert_eq!(error.original_error.to_string(), "verify mismatch");
    assert_eq!(error.cleanup.kill_error.as_deref(), Some("kill denied"));
    assert!(error.cleanup.reaped);
    assert_eq!((counts.borrow().kill, counts.borrow().wait), (1, 1));
}
#[test]
fn wait_failure_keeps_ownership_until_cleanup_reaps() {
    let (mut c, counts) = child();
    c.waits = [false, true].into();
    let error = supervise(c, budget, &mut FakeSink::default()).unwrap_err();
    assert_eq!(error.stage, "child_wait");
    assert!(error.cleanup.reaped);
    assert_eq!((counts.borrow().kill, counts.borrow().wait), (1, 2));
}
#[test]
fn unresolved_cleanup_is_not_reported_terminal_and_guard_makes_final_attempt() {
    let (mut c, counts) = child();
    c.kill_error = true;
    c.waits = [false, false].into();
    let mut sink = FakeSink {
        fail_start: true,
        ..FakeSink::default()
    };
    let error = supervise(c, budget, &mut sink).unwrap_err();
    assert!(!error.cleanup.reaped);
    assert_eq!(sink.failure_reaped, Some(false));
    assert_eq!(
        error.cleanup.wait_error.as_deref(),
        Some("wait unavailable")
    );
    assert_eq!((counts.borrow().kill, counts.borrow().wait), (2, 2));
}
#[test]
fn terminal_write_failure_does_not_kill_an_already_reaped_child() {
    let (c, counts) = child();
    let mut sink = FakeSink {
        fail_terminal: true,
        ..FakeSink::default()
    };
    let error = supervise(c, budget, &mut sink).unwrap_err();
    assert!(error.cleanup.reaped);
    assert!(!error.cleanup.attempted);
    assert_eq!(error.stage, "terminal_receipt");
    assert_eq!((counts.borrow().kill, counts.borrow().wait), (0, 1));
}
#[test]
fn failure_receipt_error_does_not_hide_original_and_cleanup_outcome() {
    let (c, _) = child();
    let mut sink = FakeSink {
        fail_start: true,
        fail_failure: true,
        ..FakeSink::default()
    };
    let error = supervise(c, budget, &mut sink).unwrap_err();
    assert_eq!(error.receipt_error.as_deref(), Some("write failure"));
    assert!(error.to_string().contains("write start"));
    assert!(error.to_string().contains("reaped: true"));
}
#[test]
fn normal_exit_waits_once_without_termination() {
    let (c, counts) = child();
    assert_eq!(supervise(c, budget, &mut FakeSink::default()).unwrap(), 0);
    assert_eq!((counts.borrow().kill, counts.borrow().wait), (0, 1));
}
#[test]
fn real_owned_child_is_reaped_when_started_receipt_fails() {
    let child = std::process::Command::new("rustc")
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("trusted Rust toolchain child");
    let pid = child.id();
    let mut sink = FakeSink {
        fail_start: true,
        ..FakeSink::default()
    };
    let error = supervise(child, |_| Ok((3, 0x4000)), &mut sink).unwrap_err();
    assert_eq!(error.pid, pid);
    assert!(error.cleanup.reaped);
    assert_eq!(sink.failure_reaped, Some(true));
}
