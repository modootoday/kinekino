use super::*;
use std::time::{Duration, Instant};

fn deadline() -> Deadline {
    Deadline::new(
        Instant::now(),
        Duration::from_secs(5),
        Duration::from_secs(1),
    )
    .unwrap()
}

#[test]
fn second_pair_failure_drops_first_handles_without_starting_io() {
    let mut calls = 0;
    let mut first_pipe = None;
    let result = PreparedSourceIpc::with_pair(deadline(), |maximum, cutoffs, deadline| {
        calls += 1;
        if calls == 2 {
            return Err(io::Error::other("original second pair setup failure"));
        }
        let (lane, child) = pair(maximum, cutoffs, deadline)?;
        assert!(lane.connect.is_none());
        assert!(!lane.connected);
        first_pipe = Some(Arc::downgrade(lane.pipe.as_ref().unwrap()));
        Ok((lane, child))
    });
    assert_eq!(calls, 2);
    assert_eq!(
        result.err().unwrap().to_string(),
        "original second pair setup failure"
    );
    assert!(first_pipe.unwrap().upgrade().is_none());
}

#[test]
fn unstarted_owner_rejects_io_and_closes_without_connecting() {
    let mut prepared = PreparedSourceIpc::new(deadline()).unwrap();
    assert!(prepared.parent.control.start_read(1).is_err());
    assert!(prepared.parent.data.start_write(b"x").is_err());
    assert!(prepared.parent.control.connect.is_none());
    assert!(prepared.parent.data.connect.is_none());
    let closed = prepared.poll_close();
    assert!(closed.io_reaped && closed.local_endpoints_closed);
    assert!(!closed.control_eof && !closed.data_eof);
    assert!(prepared.poll_connected().is_err());
}

#[test]
fn retained_owner_connects_both_original_lanes_before_use() {
    let mut prepared = PreparedSourceIpc::new(deadline()).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(2);
    while !prepared.poll_connected().unwrap() {
        assert!(Instant::now() < cutoff);
        std::thread::yield_now();
    }
    assert!(prepared.parent.control.connected && prepared.parent.data.connected);
    let closed = prepared.poll_close();
    assert!(closed.io_reaped && closed.local_endpoints_closed);
    assert!(!closed.had_error);
}
