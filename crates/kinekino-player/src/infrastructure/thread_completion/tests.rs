use super::*;
use std::{
    cell::RefCell,
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

fn host() -> DispatcherHost {
    DispatcherHost::start(NonZeroUsize::new(1).unwrap()).unwrap()
}
fn wake(sender: mpsc::Sender<()>) -> CompletionWake {
    Arc::new(move || sender.send(()).map_err(|error| error.to_string()))
}
fn finish(host: &mut DispatcherHost) -> DispatcherShutdown {
    host.request_shutdown();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let state = host.poll_shutdown();
        if matches!(state, DispatcherShutdown::Joined { .. }) {
            return state;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}

struct ExitGate(mpsc::Sender<()>, mpsc::Receiver<()>);
impl Drop for ExitGate {
    fn drop(&mut self) {
        let _ = self.0.send(());
        let _ = self.1.recv();
    }
}
thread_local! { static EXIT_GATE: RefCell<Option<ExitGate>> = const { RefCell::new(None) }; }

#[test]
fn wake_and_result_wait_for_actual_child_exit_and_join() {
    let mut host = host();
    let (body_done_tx, body_done_rx) = mpsc::channel();
    let (exit_tx, exit_rx) = mpsc::channel();
    let (wake_tx, wake_rx) = mpsc::channel();
    let mut ticket = host
        .handle()
        .submit(JoinRequest::new(
            move || {
                EXIT_GATE.with(|slot| *slot.borrow_mut() = Some(ExitGate(body_done_tx, exit_rx)));
                42
            },
            wake(wake_tx),
        ))
        .unwrap();
    body_done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(ticket.try_take().is_none());
    assert!(wake_rx.try_recv().is_err());
    host.request_shutdown();
    assert_eq!(host.poll_shutdown(), DispatcherShutdown::Pending);
    exit_tx.send(()).unwrap();
    wake_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(ticket.try_take(), Some(Ok(42)));
    assert!(matches!(
        finish(&mut host),
        DispatcherShutdown::Joined {
            worker_panicked: false,
            faults: 0
        }
    ));
}

#[test]
fn bounded_rejection_retains_the_unstarted_work_and_closed_host_drains_accepted_jobs() {
    let mut host = host();
    let handle = host.handle();
    let (started_tx, started_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let noop: CompletionWake = Arc::new(|| Ok(()));
    let mut first = handle
        .submit(JoinRequest::new(
            move || {
                started_tx.send(()).unwrap();
                resume_rx.recv().unwrap();
                1
            },
            noop.clone(),
        ))
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut second = handle.submit(JoinRequest::new(|| 2, noop.clone())).unwrap();
    let ran = Arc::new(AtomicBool::new(false));
    let work_ran = ran.clone();
    let owned = Arc::new(());
    let weak_owner = Arc::downgrade(&owned);
    let refused = handle
        .submit(JoinRequest::new(
            move || {
                let _owner = owned;
                work_ran.store(true, Ordering::Release);
                3
            },
            noop,
        ))
        .unwrap_err();
    assert_eq!(refused.reason, SubmitRejection::Full);
    assert!(weak_owner.upgrade().is_some());
    assert!(!ran.load(Ordering::Acquire));
    host.request_shutdown();
    let refused_again = handle.submit(refused.request).unwrap_err();
    assert_eq!(refused_again.reason, SubmitRejection::Closed);
    assert!(weak_owner.upgrade().is_some());
    assert!(!ran.load(Ordering::Acquire));
    assert_eq!(host.poll_shutdown(), DispatcherShutdown::Pending);
    resume_tx.send(()).unwrap();
    assert!(matches!(
        finish(&mut host),
        DispatcherShutdown::Joined { faults: 0, .. }
    ));
    assert_eq!(first.try_take(), Some(Ok(1)));
    assert_eq!(second.try_take(), Some(Ok(2)));
    // The rejected closure remains owned and has never been started.
    drop(refused_again);
    assert!(weak_owner.upgrade().is_none());
    assert!(!ran.load(Ordering::Acquire));
}

#[test]
fn notification_panic_keeps_the_joined_result_and_host_fault_is_separate() {
    let mut host = host();
    let mut ticket = host
        .handle()
        .submit(JoinRequest::new(
            || 7,
            Arc::new(|| panic!("Injected completion notification panic")),
        ))
        .unwrap();
    assert!(matches!(
        finish(&mut host),
        DispatcherShutdown::Joined {
            worker_panicked: false,
            faults: 1
        }
    ));
    assert_eq!(ticket.try_take(), Some(Ok(7)));
    assert!(matches!(
        ticket.notification(),
        NotificationState::Failed(_)
    ));
}

#[test]
fn dispatcher_fault_after_join_does_not_destroy_the_child_result() {
    let mut host = host();
    let mut request = JoinRequest::new(|| 9, Arc::new(|| Ok(())));
    let original = request.job.run;
    request.job.run = Box::new(move || {
        original();
        panic!("Injected dispatcher fault after joined publication")
    });
    let mut ticket = host.handle().submit(request).unwrap();
    assert!(matches!(
        finish(&mut host),
        DispatcherShutdown::Joined {
            worker_panicked: false,
            faults: 1
        }
    ));
    assert_eq!(ticket.try_take(), Some(Ok(9)));
    assert_eq!(ticket.notification(), NotificationState::Returned);
}
