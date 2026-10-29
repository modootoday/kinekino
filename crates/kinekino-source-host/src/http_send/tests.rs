use super::*;
use reqwest::ResponseBuilderExt;
fn target() -> url::Url {
    url::Url::parse("https://example.invalid/media").unwrap()
}
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct PendingProbe {
    polls: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
}
impl Future for PendingProbe {
    type Output = Result<reqwest::Response, SendError>;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Poll::Pending
    }
}
impl Drop for PendingProbe {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
fn ready(status: u16) -> SendFuture {
    Box::pin(std::future::ready(Ok(http::Response::builder()
        .url(target())
        .status(status)
        .body("body")
        .unwrap()
        .into())))
}
#[test]
fn invalidation_before_poll_retains_original_future_until_abort() {
    let polls = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut owner = HttpSendOwner::from_future(
        Box::pin(PendingProbe {
            polls: polls.clone(),
            drops: drops.clone(),
        }),
        10,
        target(),
    );
    let mut cx = Context::from_waker(std::task::Waker::noop());
    assert_eq!(
        owner.poll_open(&mut cx, || Err(SendError::Invalidated)),
        Poll::Ready(Err(SendError::Invalidated))
    );
    assert_eq!(polls.load(Ordering::SeqCst), 0);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    owner.abort();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(
        owner.poll_open(&mut cx, || Ok(())),
        Poll::Ready(Err(SendError::Invalidated))
    );
}
#[test]
fn pending_poll_checks_both_sides_and_cannot_restart_after_abort() {
    let polls = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut owner = HttpSendOwner::from_future(
        Box::pin(PendingProbe {
            polls: polls.clone(),
            drops: drops.clone(),
        }),
        10,
        target(),
    );
    let mut cx = Context::from_waker(std::task::Waker::noop());
    let mut checks = 0;
    assert_eq!(
        owner.poll_open(&mut cx, || {
            checks += 1;
            Ok(())
        }),
        Poll::Pending
    );
    assert_eq!(checks, 2);
    assert_eq!(polls.load(Ordering::SeqCst), 1);
    owner.abort();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(
        owner.poll_open(&mut cx, || Ok(())),
        Poll::Ready(Err(SendError::Cancelled))
    );
    assert_eq!(polls.load(Ordering::SeqCst), 1);
}
#[test]
fn response_is_owned_before_post_poll_invalidation_or_validation_failure() {
    let mut cx = Context::from_waker(std::task::Waker::noop());
    let mut owner = HttpSendOwner::from_future(ready(200), 10, target());
    let mut checks = 0;
    assert_eq!(
        owner.poll_open(&mut cx, || {
            checks += 1;
            if checks == 2 {
                Err(SendError::Invalidated)
            } else {
                Ok(())
            }
        }),
        Poll::Ready(Err(SendError::Invalidated))
    );
    assert!(owner.response.is_some());
    assert!(owner.pending.is_none());
    assert_eq!(owner.response_mut().err(), Some(SendError::Invalidated));
    owner.abort();
    assert!(owner.response.is_some());
    let mut rejected = HttpSendOwner::from_future(ready(206), 10, target());
    assert_eq!(
        rejected.poll_open(&mut cx, || Ok(())),
        Poll::Ready(Err(SendError::Response(ResponseError::Status)))
    );
    assert!(rejected.response.is_some());
}
#[test]
fn post_poll_panic_preserves_response_and_normal_completion_is_single_use() {
    let mut cx = Context::from_waker(std::task::Waker::noop());
    let mut owner = HttpSendOwner::from_future(ready(200), 10, target());
    let mut checks = 0;
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        owner.poll_open(&mut cx, || {
            checks += 1;
            assert!(checks != 2, "post-poll fixture panic");
            Ok(())
        })
    }));
    assert!(panic.is_err());
    assert!(owner.response.is_some());
    owner.abort();
    let mut success = HttpSendOwner::from_future(ready(200), 10, target());
    assert_eq!(success.poll_open(&mut cx, || Ok(())), Poll::Ready(Ok(())));
    assert!(success.response_mut().is_ok());
    assert_eq!(
        success.poll_open(&mut cx, || Ok(())),
        Poll::Ready(Err(SendError::Order))
    );
}
