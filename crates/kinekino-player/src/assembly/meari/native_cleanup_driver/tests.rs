use super::*;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Complete,
    BudgetExhausted,
    ForcedIncomplete,
    Faulted,
}
struct Fake {
    polls: VecDeque<CleanupPoll<Status>>,
    owners: Rc<()>,
    evidence: Rc<()>,
    events: Rc<RefCell<Vec<&'static str>>>,
    acceptance: Result<(), Rc<str>>,
}
impl CleanupAdapter for Fake {
    type Owners = Rc<()>;
    type Evidence = Rc<()>;
    type Status = Status;
    type Error = Rc<str>;
    fn poll(&mut self) -> CleanupPoll<Status> {
        self.events.borrow_mut().push("poll");
        self.polls
            .pop_front()
            .expect("driver must not poll past terminal")
    }
    fn accept_terminal_for_shutdown(&mut self) -> Result<(), Rc<str>> {
        self.events.borrow_mut().push("accept");
        self.acceptance.clone()
    }
    fn acceptance_failure_status(&self) -> Status {
        Status::Faulted
    }
    fn into_parts(self) -> (Rc<()>, Rc<()>) {
        (self.owners, self.evidence)
    }
}
fn fake(polls: Vec<CleanupPoll<Status>>) -> Fake {
    Fake {
        polls: polls.into(),
        owners: Rc::new(()),
        evidence: Rc::new(()),
        events: Rc::new(RefCell::new(Vec::new())),
        acceptance: Ok(()),
    }
}

#[test]
fn pending_delays_and_explicit_acceptance_preserve_original_carrier() {
    let adapter = fake(vec![
        CleanupPoll::Pending(HostRetry::After(Duration::from_millis(7))),
        CleanupPoll::Pending(HostRetry::OnWake),
        CleanupPoll::NeedsTerminalAcceptance,
        CleanupPoll::Terminal(Status::Complete),
    ]);
    let owners = adapter.owners.clone();
    let evidence = adapter.evidence.clone();
    let events = adapter.events.clone();
    let mut sleeps = Vec::new();
    let result = drive_cleanup(adapter, Duration::from_millis(5), |delay| {
        sleeps.push(delay)
    });
    assert_eq!(sleeps, [Duration::from_millis(7), Duration::from_millis(5)]);
    assert_eq!(*events.borrow(), ["poll", "poll", "poll", "accept", "poll"]);
    assert_eq!(result.status, Status::Complete);
    assert!(result.acceptance_error.is_none());
    assert!(Rc::ptr_eq(&owners, &result.owners));
    assert!(Rc::ptr_eq(&evidence, &result.evidence));
}

#[test]
fn acceptance_failure_returns_original_error_and_owners_without_retry() {
    let mut adapter = fake(vec![CleanupPoll::NeedsTerminalAcceptance]);
    let error: Rc<str> = "original acceptance failure".into();
    adapter.acceptance = Err(error.clone());
    let owners = adapter.owners.clone();
    let evidence = adapter.evidence.clone();
    let events = adapter.events.clone();
    let result = drive_cleanup(adapter, Duration::from_millis(5), |_| {
        panic!("acceptance failure must not sleep")
    });
    assert_eq!(result.status, Status::Faulted);
    assert_eq!(*events.borrow(), ["poll", "accept"]);
    assert!(Rc::ptr_eq(
        &error,
        result.acceptance_error.as_ref().unwrap()
    ));
    assert!(Rc::ptr_eq(&owners, &result.owners));
    assert!(Rc::ptr_eq(&evidence, &result.evidence));
}

#[test]
fn budget_and_forced_terminals_are_not_accepted_or_retried() {
    for status in [
        Status::BudgetExhausted,
        Status::ForcedIncomplete,
        Status::Faulted,
    ] {
        let adapter = fake(vec![CleanupPoll::Terminal(status)]);
        let events = adapter.events.clone();
        let owners = adapter.owners.clone();
        let result = drive_cleanup(adapter, Duration::from_millis(5), |_| {
            panic!("terminal must not sleep")
        });
        assert_eq!(result.status, status);
        assert_eq!(*events.borrow(), ["poll"]);
        assert!(result.acceptance_error.is_none());
        assert!(Rc::ptr_eq(&owners, &result.owners));
    }
}

#[test]
fn pending_then_adapter_budget_exhaustion_is_terminal_without_budget_reset() {
    let adapter = fake(vec![
        CleanupPoll::Pending(HostRetry::OnWake),
        CleanupPoll::Terminal(Status::BudgetExhausted),
    ]);
    let events = adapter.events.clone();
    let mut sleeps = 0;
    let result = drive_cleanup(adapter, Duration::from_millis(5), |_| sleeps += 1);
    assert_eq!(sleeps, 1);
    assert_eq!(*events.borrow(), ["poll", "poll"]);
    assert_eq!(result.status, Status::BudgetExhausted);
}
