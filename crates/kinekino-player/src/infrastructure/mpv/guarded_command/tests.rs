use super::*;
use crate::application::{api::ErrorCode, playback_session::GuardedSeekState as State};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
fn identity() -> Identity {
    Identity {
        engine: 7,
        resolution: [3; 32],
        connection: 1,
        lifecycle: 2,
        entry: Some(9),
    }
}
fn test_host() -> GuardedQueue {
    GuardedQueue::standalone(HostScope::testing(), 7).unwrap()
}
fn reply(id: i64, data: Value) -> Value {
    json!({"request_id":id,"error":"success","data":data})
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn playlist() -> Value {
    json!([{"id":9,"playing":true}])
}
fn properties(barrier: &mut Barrier, ids: &mut RequestIds, before: i64) -> Vec<Value> {
    match barrier
        .receive(&reply(before, playlist()), &identity(), Instant::now(), ids)
        .unwrap()
    {
        BarrierUpdate::Send(values) => values,
        _ => panic!("expected six queries"),
    }
}
#[test]
fn eight_queries_are_strictly_before_six_replies_then_after() {
    let mut ids = RequestIds::default();
    let (mut barrier, before) = Barrier::start(identity(), deadline(), &mut ids).unwrap();
    let properties = properties(
        &mut barrier,
        &mut ids,
        before["request_id"].as_i64().unwrap(),
    );
    assert_eq!(properties.len(), 6);
    let values = [
        json!(3.5),
        json!(60.),
        json!(false),
        json!(false),
        json!(true),
        json!(false),
    ];
    let mut after = None;
    for (index, (query, value)) in properties.iter().zip(values).enumerate() {
        let result = barrier
            .receive(
                &reply(query["request_id"].as_i64().unwrap(), value),
                &identity(),
                Instant::now(),
                &mut ids,
            )
            .unwrap();
        if index < 5 {
            assert!(matches!(result, BarrierUpdate::Ignored));
        } else if let BarrierUpdate::Send(queries) = result {
            assert_eq!(queries.len(), 1);
            after = Some(queries[0]["request_id"].as_i64().unwrap());
        } else {
            panic!("after query missing");
        }
    }
    assert!(
        matches!(barrier.receive(&reply(after.unwrap(),playlist()),&identity(),Instant::now(),&mut ids).unwrap(),BarrierUpdate::Complete(sample) if sample.position==3.5 && sample.duration==60. && !sample.paused && !sample.idle)
    );
    assert!(matches!(
        barrier
            .receive(
                &reply(after.unwrap(), playlist()),
                &identity(),
                Instant::now(),
                &mut ids
            )
            .unwrap(),
        BarrierUpdate::Ignored
    ));
}
#[test]
fn timeout_wrong_ids_duplicates_and_old_barrier_cannot_complete_new_barrier() {
    let now = Instant::now();
    let mut ids = RequestIds::default();
    let (mut old, first) = Barrier::start(identity(), now, &mut ids).unwrap();
    assert!(matches!(
        old.receive(
            &reply(first["request_id"].as_i64().unwrap(), playlist()),
            &identity(),
            now,
            &mut ids
        )
        .unwrap(),
        BarrierUpdate::Rejected
    ));
    let (mut new, second) = Barrier::start(identity(), deadline(), &mut ids).unwrap();
    assert!(matches!(
        new.receive(
            &reply(first["request_id"].as_i64().unwrap(), playlist()),
            &identity(),
            now,
            &mut ids
        )
        .unwrap(),
        BarrierUpdate::Ignored
    ));
    let queries = properties(&mut new, &mut ids, second["request_id"].as_i64().unwrap());
    let response = reply(queries[0]["request_id"].as_i64().unwrap(), json!(1.));
    assert!(matches!(
        new.receive(&response, &identity(), now, &mut ids).unwrap(),
        BarrierUpdate::Ignored
    ));
    assert!(matches!(
        new.receive(&response, &identity(), now, &mut ids).unwrap(),
        BarrierUpdate::Ignored
    ));
    assert!(matches!(
        new.receive(&json!({"event":"end-file"}), &identity(), now, &mut ids)
            .unwrap(),
        BarrierUpdate::Rejected
    ));
}
#[test]
fn missing_partial_and_wrong_playing_entry_close_barrier() {
    for data in [
        json!([]),
        json!([{"id":10,"playing":true}]),
        json!([{"id":9,"playing":true},{"id":10,"playing":true}]),
        json!([{"id":9,"playing":null}]),
        json!(vec![json!({"id":9,"playing":true}); 65]),
    ] {
        let mut ids = RequestIds::default();
        let (mut barrier, query) = Barrier::start(identity(), deadline(), &mut ids).unwrap();
        assert!(matches!(
            barrier
                .receive(
                    &reply(query["request_id"].as_i64().unwrap(), data),
                    &identity(),
                    Instant::now(),
                    &mut ids
                )
                .unwrap(),
            BarrierUpdate::Rejected
        ));
    }
    let mut ids = RequestIds::default();
    let (mut barrier, query) = Barrier::start(identity(), deadline(), &mut ids).unwrap();
    let queries = properties(
        &mut barrier,
        &mut ids,
        query["request_id"].as_i64().unwrap(),
    );
    assert!(matches!(
        barrier
            .receive(
                &reply(queries[5]["request_id"].as_i64().unwrap(), Value::Null),
                &identity(),
                Instant::now(),
                &mut ids
            )
            .unwrap(),
        BarrierUpdate::Rejected
    ));
    assert!(std::mem::size_of::<Barrier>() <= 4096);
}
#[test]
fn registry_capacity_pending_consume_and_terminal_consumption_preserve_rows() {
    let registry = Registry::default();
    let tickets: Vec<_> = (0..64)
        .map(|_| registry.reserve(identity()).unwrap())
        .collect();
    assert!(
        matches!(registry.reserve(identity()),Err(error) if error.code==ErrorCode::ResourceExhausted)
    );
    assert_eq!(
        registry.read(&tickets[0], true).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        registry.read(&tickets[0].clone(), false).unwrap().state,
        State::Queued
    );
    registry
        .transition(&tickets[0], State::RejectedBeforeWrite, None)
        .unwrap();
    registry.read(&tickets[0], true).unwrap();
    assert_eq!(
        registry.read(&tickets[0], false).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert!(
        !registry
            .transition(&tickets[0], State::CommandAccepted, None)
            .unwrap()
    );
    registry.reserve(identity()).unwrap();
}
#[test]
fn terminal_consume_race_has_exactly_one_winner_and_foreign_registry_is_rejected() {
    let registry = Registry::default();
    let ticket = registry.reserve(identity()).unwrap();
    registry
        .transition(&ticket, State::RejectedBeforeWrite, None)
        .unwrap();
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let registry = registry.clone();
            let ticket = ticket.clone();
            std::thread::spawn(move || registry.read(&ticket, true))
        })
        .collect();
    let mut wins = 0;
    for worker in workers {
        match worker.join().unwrap() {
            Ok(_) => wins += 1,
            Err(error) => assert_eq!(error.code, ErrorCode::NotFound),
        }
    }
    assert_eq!(wins, 1);
    let foreign = Registry::default();
    assert_eq!(
        foreign.read(&ticket, false).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
}
#[test]
fn close_keeps_original_ticket_result_without_retargeting_or_eviction() {
    let registry = Registry::default();
    let queued = registry.reserve(identity()).unwrap();
    let committed = registry.reserve(identity()).unwrap();
    registry
        .transition(&committed, State::WriteCommitted, None)
        .unwrap();
    registry
        .close(Failure::text(FailureStage::Connection, "disconnected"))
        .unwrap();
    let original = registry.read(&committed, false).unwrap();
    assert!(original.committed);
    assert_eq!(original.identity, identity());
    assert_eq!(original.state, State::OutcomeUnobserved);
    assert_eq!(
        registry.read(&queued, false).unwrap().state,
        State::RejectedBeforeWrite
    );
    assert!(
        !registry
            .transition(&committed, State::CommandAccepted, None)
            .unwrap()
    );
    drop(registry);
    assert_eq!(
        Registry::default()
            .read(&committed, false)
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn failure_prefix_hash_and_native_error_metadata_are_bounded() {
    let text = "가".repeat(10000);
    let failure = Failure::text(FailureStage::Acknowledgement, &text);
    assert!(failure.prefix().len() <= results::ROW_PAYLOAD_BYTES - 512);
    assert!(failure.truncated);
    assert_eq!(failure.original_bytes, text.len());
    use sha2::{Digest, Sha256};
    assert_eq!(
        failure.sha256,
        <[u8; 32]>::from(Sha256::digest(text.as_bytes()))
    );
    let error = std::io::Error::from_raw_os_error(5);
    let failure = Failure::io(FailureStage::Write, &error);
    assert_eq!(failure.raw_os_code, Some(5));
    assert_eq!(failure.kind, Some(error.kind()));
    assert_eq!(failure.stage, FailureStage::Write);
}
#[test]
fn queue_full_rollback_closed_receiver_and_revoke_never_accept_new_packets() {
    let host = test_host();
    let receiver = host.take_receiver().unwrap();
    let mut waiters = Vec::new();
    for _ in 0..64 {
        waiters.push(host.observe(identity(), false, deadline()).unwrap());
    }
    assert!(
        matches!(host.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::ResourceExhausted)
    );
    drop(waiters.remove(0));
    host.registry.reap_observations(Instant::now()).unwrap();
    assert!(
        matches!(host.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::ResourceExhausted)
    );
    let probe = host.registry.reserve(identity()).unwrap();
    host.registry.rollback(&probe).unwrap();
    drop(receiver);
    assert!(
        matches!(host.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::EngineFailed)
    );
    host.close().unwrap();
    assert!(
        matches!(host.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::Cancelled)
    );
}
#[test]
fn checked_wire_allocator_never_reuses_subscription_legacy_or_guarded_ids() {
    let mut ids = RequestIds::default();
    assert_eq!(ids.next().unwrap(), 1);
    assert_eq!(ids.reserve(6).unwrap(), vec![2, 3, 4, 5, 6, 7]);
    assert_eq!(ids.next().unwrap(), 8);
    ids.0 = i64::MAX - 1;
    assert!(ids.reserve(6).is_err());
    assert_eq!(ids.next().unwrap(), i64::MAX);
    assert!(ids.next().is_err());
    assert!(ids.reserve(0).is_err());
}

fn ready_seek(host: &GuardedQueue, worker: &mut Worker, ids: &mut RequestIds) -> (Ticket, Packet) {
    let (ticket, packet, _receiver) = ready_seek_with_receiver(host, worker, ids);
    (ticket, packet)
}
fn observe_guard(
    host: &GuardedQueue,
    worker: &mut Worker,
    ids: &mut RequestIds,
    receiver: &mut tokio::sync::mpsc::Receiver<queue::Envelope>,
) -> crate::application::playback_session::NativeObservationGuard {
    let waiter = host.observe(identity(), false, deadline()).unwrap();
    let before = worker
        .begin(receiver.try_recv().unwrap(), ids, host, Instant::now())
        .unwrap()
        .remove(0);
    assert!(before.permit.is_none());
    let properties = worker
        .receive(
            &reply(before.wire["request_id"].as_i64().unwrap(), playlist()),
            Some((1, 2)),
            ids,
            host,
            Instant::now(),
        )
        .unwrap();
    assert_eq!(properties.len(), 6);
    let mut after = Vec::new();
    for (query, value) in properties.into_iter().zip([
        json!(3.),
        json!(60.),
        json!(false),
        json!(false),
        json!(true),
        json!(false),
    ]) {
        assert!(query.permit.is_none());
        after = worker
            .receive(
                &reply(query.wire["request_id"].as_i64().unwrap(), value),
                Some((1, 2)),
                ids,
                host,
                Instant::now(),
            )
            .unwrap();
    }
    assert_eq!(after.len(), 1);
    assert!(
        worker
            .receive(
                &reply(after[0].wire["request_id"].as_i64().unwrap(), playlist()),
                Some((1, 2)),
                ids,
                host,
                Instant::now()
            )
            .unwrap()
            .is_empty()
    );
    assert_eq!(host.fence.in_flight(), 0);
    assert!(receiver.try_recv().is_err());
    waiter.wait().unwrap()
}
fn evaluate_target(
    guard: &crate::application::playback_session::NativeObservationGuard,
) -> crate::domain::api_values::DecimalU64 {
    use crate::domain::{api_values::DecimalU64, skip_segments::*};
    let segment = ManualSegment {
        id: "opening-1".into(),
        revision: DecimalU64(1),
        kind: SegmentKind::Opening,
        in_us: DecimalU64(1_000_000),
        out_us: DecimalU64(4_000_000),
        origin: SegmentOrigin::Manual,
        timebase: SegmentTimebase::EnginePlayback,
        verified_revision: Some(DecimalU64(1)),
    };
    let view = guard.frozen_view();
    let playback = view.playback();
    let decision = evaluate(
        &segment,
        std::slice::from_ref(&segment),
        &SkipPolicy::default(),
        &PassState::default(),
        &Evaluation {
            position_us: playback.position_us,
            duration_us: playback.duration_us,
            playing: playback.paused == Some(false) && playback.idle == Some(false),
            observation_authorized: view.observation_authorized(),
            seekability: view.seekability(),
        },
        Intent::Manual,
    );
    match decision {
        Decision::Eligible { target_us } => target_us,
        other => panic!("{other:?}"),
    }
}
fn queued_seek(
    host: &GuardedQueue,
    worker: &mut Worker,
    ids: &mut RequestIds,
    receiver: &mut tokio::sync::mpsc::Receiver<queue::Envelope>,
) -> (
    crate::application::playback_session::GuardedSeekSubmission,
    queue::SubmitInput,
) {
    let guard = observe_guard(host, worker, ids, receiver);
    let target = evaluate_target(&guard);
    let submission = guard.submit_native(target).unwrap();
    assert_eq!(submission.receipt().engine_instance_id.0, 7);
    let queue::Envelope::Submit(input) = receiver.try_recv().unwrap() else {
        panic!("Submit must not start another observation")
    };
    (submission, input)
}
fn ready_seek_with_receiver(
    host: &GuardedQueue,
    worker: &mut Worker,
    ids: &mut RequestIds,
) -> (Ticket, Packet, tokio::sync::mpsc::Receiver<queue::Envelope>) {
    let mut receiver = host.take_receiver().unwrap();
    let (_, input) = queued_seek(host, worker, ids, &mut receiver);
    let ticket = input.ticket.clone();
    let mut packets = worker
        .begin(queue::Envelope::Submit(input), ids, host, Instant::now())
        .unwrap();
    assert_eq!(packets.len(), 1);
    (ticket, packets.remove(0), receiver)
}
#[test]
fn worker_preserves_enqueue_then_write_then_ack_and_never_reports_landing() {
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let (ticket, packet) = ready_seek(&host, &mut worker, &mut ids);
    assert_eq!(
        host.registry.read(&ticket, false).unwrap().state,
        State::WriteCommitted
    );
    assert_eq!(host.fence.in_flight(), 1);
    assert_eq!(
        packet.wire["command"],
        json!(["seek", 4., "absolute+exact"])
    );
    let id = packet.wire["request_id"].as_i64().unwrap();
    host.registry
        .transition(&ticket, State::WrittenAwaitingAck, None)
        .unwrap();
    drop(packet);
    assert_eq!(host.fence.in_flight(), 0);
    worker
        .receive(
            &reply(id + 100, Value::Null),
            Some((1, 2)),
            &mut ids,
            &host,
            Instant::now(),
        )
        .unwrap();
    assert_eq!(
        host.registry.read(&ticket, false).unwrap().state,
        State::WrittenAwaitingAck
    );
    worker
        .receive(
            &reply(id, Value::Null),
            Some((1, 2)),
            &mut ids,
            &host,
            Instant::now(),
        )
        .unwrap();
    assert_eq!(
        host.registry.read(&ticket, false).unwrap().state,
        State::CommandAccepted
    );
    host.registry.read(&ticket, true).unwrap();
    worker
        .receive(
            &reply(id, Value::Null),
            Some((1, 2)),
            &mut ids,
            &host,
            Instant::now(),
        )
        .unwrap();
    assert_eq!(
        host.registry.read(&ticket, false).unwrap_err().code,
        ErrorCode::NotFound
    );
}
#[test]
fn generic_writer_partial_failure_drops_permit_and_does_not_append_next_frame() {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };
    struct Partial {
        bytes: Vec<u8>,
        polls: usize,
    }
    impl tokio::io::AsyncWrite for Partial {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
            bytes: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.polls += 1;
            if self.polls == 1 {
                let count = bytes.len().min(3);
                self.bytes.extend_from_slice(&bytes[..count]);
                Poll::Ready(Ok(count))
            } else {
                Poll::Ready(Err(std::io::Error::from_raw_os_error(5)))
            }
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let (ticket, packet) = ready_seek(&host, &mut worker, &mut ids);
    let mut writer = Partial {
        bytes: Vec::new(),
        polls: 0,
    };
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut interval = tokio::time::interval(Duration::from_millis(1));
            let second = Packet {
                wire: json!({"command":["must-not-write"]}),
                deadline: deadline(),
                permit: None,
                ticket: None,
            };
            assert!(
                super::super::ipc::write_packets(
                    &mut writer,
                    vec![packet, second],
                    &host,
                    &mut interval
                )
                .await
                .is_err()
            );
        });
    assert_eq!(writer.bytes.len(), 3);
    assert_eq!(writer.polls, 2);
    assert_eq!(host.fence.in_flight(), 0);
    let result = host.registry.read(&ticket, false).unwrap();
    assert_eq!(result.state, State::OutcomeUnobserved);
    assert_eq!(result.failure.unwrap().raw_os_code, Some(5));
}

#[test]
fn application_reader_uses_original_weak_issuer_and_actual_receipt() {
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let mut receiver = host.take_receiver().unwrap();
    let (submission, _input) = queued_seek(&host, &mut worker, &mut ids, &mut receiver);
    assert_eq!(submission.receipt().engine_instance_id.0, 7);
    let (_, read) = submission.into_parts();
    assert_eq!(read.read_original(false).unwrap().state(), State::Queued);
    assert_eq!(
        read.read_original(true).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    host.close().unwrap();
    assert_eq!(
        read.read_original(false)
            .unwrap()
            .original_identity()
            .engine_instance_id()
            .0,
        7
    );
    assert_eq!(
        read.clone().read_original(true).unwrap().state(),
        State::RejectedBeforeWrite
    );
    assert_eq!(
        read.read_original(false).unwrap_err().code,
        ErrorCode::NotFound
    );
    drop(host);
    assert_eq!(
        read.read_original(false).unwrap_err().code,
        ErrorCode::SourceExpired
    );
}

#[test]
fn bounded_error_response_including_json_escaping_is_below_row_budget() {
    let registry = Registry::default();
    let ticket = registry.reserve(identity()).unwrap();
    registry
        .transition(
            &ticket,
            State::RejectedBeforeWrite,
            Some(Failure::text(
                FailureStage::BeforeWrite,
                &"\u{0001}".repeat(20000),
            )),
        )
        .unwrap();
    let submission = ticket
        .submission(crate::application::playback_control::ControlReceipt::enqueued(7))
        .unwrap();
    let result = submission.ticket.read_original(false).unwrap();
    let error = result.failure().unwrap().api_error();
    assert!(serde_json::to_vec(error).unwrap().len() < results::ROW_PAYLOAD_BYTES);
}

#[test]
fn legacy_channel_preserves_fifo_without_creating_or_retargeting_guarded_results() {
    use crate::domain::playback_control::PlaybackControl;
    let (sender, mut legacy) = tokio::sync::mpsc::channel(64);
    let handle = super::super::control::MpvControlHandle::new(sender).unwrap();
    handle
        .state
        .connected
        .store(true, std::sync::atomic::Ordering::Release);
    handle.submit(PlaybackControl::Play).unwrap();
    handle.submit(PlaybackControl::Pause).unwrap();
    let mut original = identity();
    original.engine = handle.instance_id();
    assert!(
        matches!(handle.state.guarded.observe(original,false,deadline()),
        Err(error) if error.code==ErrorCode::Unsupported)
    );
    assert_eq!(
        legacy.try_recv().unwrap(),
        json!(["set_property", "pause", false])
    );
    assert_eq!(
        legacy.try_recv().unwrap(),
        json!(["set_property", "pause", true])
    );
    assert!(legacy.try_recv().is_err());
    handle.state.close();
    assert_eq!(
        handle.submit(PlaybackControl::Play).unwrap_err().code,
        ErrorCode::NotFound
    );
}

#[test]
fn close_and_logical_commit_race_preserves_original_row_classification() {
    for _ in 0..32 {
        let host = std::sync::Arc::new(test_host());
        let mut worker = Worker::new();
        let mut ids = RequestIds::default();
        let mut receiver = host.take_receiver().unwrap();
        let (_, input) = queued_seek(&host, &mut worker, &mut ids, &mut receiver);
        let ticket = input.ticket;
        let other = host.clone();
        let original = ticket.clone();
        let generation = host.fence.generation();
        let gate = std::sync::Arc::new(std::sync::Barrier::new(2));
        let rendezvous = gate.clone();
        let commit = std::thread::spawn(move || {
            rendezvous.wait();
            other
                .registry
                .commit(&original, other.fence.word(), generation)
        });
        gate.wait();
        host.close().unwrap();
        let row = host.registry.read(&ticket, false).unwrap();
        match commit.join().unwrap() {
            Ok(permit) => {
                assert!(row.committed);
                assert_eq!(row.state, State::OutcomeUnobserved);
                drop(permit);
            }
            Err(error) => {
                assert_eq!(error.code, ErrorCode::Cancelled);
                assert!(!row.committed);
                assert_eq!(row.state, State::RejectedBeforeWrite);
            }
        }
        host.fence.word().close_and_drain().unwrap();
    }
}

#[test]
fn deadline_elapsed_while_waiting_for_row_lock_never_commits_or_obtains_writer() {
    let host = std::sync::Arc::new(test_host());
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let mut receiver = host.take_receiver().unwrap();
    let (_, mut input) = queued_seek(&host, &mut worker, &mut ids, &mut receiver);
    let ticket = input.ticket.clone();
    let dequeued_at = Instant::now();
    let deadline = dequeued_at + Duration::from_millis(100);
    input.deadline = deadline;
    host.registry.set_deadline_for_test(&ticket, deadline);
    let other = host.clone();
    let (started, receive) = std::sync::mpsc::channel();
    let attempt = host.registry.with_row_lock_for_test(|| {
        let attempt = std::thread::spawn(move || {
            started.send(()).unwrap();
            worker.begin(
                queue::Envelope::Submit(input),
                &mut ids,
                &other,
                dequeued_at,
            )
        });
        receive.recv_timeout(Duration::from_secs(1)).unwrap();
        std::thread::sleep(
            deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(1),
        );
        assert!(Instant::now() >= deadline);
        attempt
    });
    let packets = attempt.join().unwrap().unwrap();
    assert!(packets.is_empty());
    assert_eq!(host.fence.in_flight(), 0);
    let row = host.registry.read(&ticket, false).unwrap();
    assert_eq!(row.state, State::RejectedBeforeWrite);
    assert!(!row.committed);
    let failure = row.failure.unwrap();
    assert_eq!(failure.stage, FailureStage::BeforeWrite);
    assert_eq!(failure.code, ErrorCode::NeedsInput);
    assert!(receiver.try_recv().is_err());
}

#[test]
fn acknowledgement_timeout_closes_original_result_and_late_ack_cannot_revive_it() {
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let (ticket, packet) = ready_seek(&host, &mut worker, &mut ids);
    host.registry
        .transition(&ticket, State::WrittenAwaitingAck, None)
        .unwrap();
    let id = packet.wire["request_id"].as_i64().unwrap();
    let expired = packet.deadline + Duration::from_secs(1);
    drop(packet);
    assert!(worker.tick(&host, expired).unwrap());
    host.close().unwrap();
    worker
        .receive(
            &reply(id, Value::Null),
            Some((1, 2)),
            &mut ids,
            &host,
            expired,
        )
        .unwrap();
    assert_eq!(
        host.registry.read(&ticket, false).unwrap().state,
        State::OutcomeUnobserved
    );
}

#[test]
fn write_deadline_keeps_partial_future_and_never_restarts_frame() {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };
    struct Stalled {
        bytes: Vec<u8>,
        remaining: Option<usize>,
    }
    impl tokio::io::AsyncWrite for Stalled {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
            bytes: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            if let Some(remaining) = self.remaining {
                assert_eq!(bytes.len(), remaining);
                Poll::Pending
            } else {
                self.bytes.extend_from_slice(&bytes[..3]);
                self.remaining = Some(bytes.len() - 3);
                Poll::Ready(Ok(3))
            }
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let (ticket, mut packet) = ready_seek(&host, &mut worker, &mut ids);
    let mut writer = Stalled {
        bytes: Vec::new(),
        remaining: None,
    };
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut interval = tokio::time::interval(Duration::from_millis(1));
            interval.tick().await;
            packet.deadline = Instant::now() + Duration::from_millis(5);
            assert!(
                super::super::ipc::write_packets(&mut writer, vec![packet], &host, &mut interval)
                    .await
                    .is_err()
            );
        });
    assert_eq!(writer.bytes.len(), 3);
    assert_eq!(host.fence.in_flight(), 0);
    let failure = host.registry.read(&ticket, false).unwrap().failure.unwrap();
    assert_eq!(failure.kind, Some(std::io::ErrorKind::TimedOut));
}

fn stalled_legacy_case(close_on_poll: bool) {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };
    struct PendingLegacy<'a> {
        polls: usize,
        host: &'a GuardedQueue,
        close_on_poll: bool,
        queued: Option<issuer::ObservationWaiter>,
    }
    impl tokio::io::AsyncWrite for PendingLegacy<'_> {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.polls += 1;
            if self.queued.is_none() {
                self.queued = Some(
                    self.host
                        .observe(
                            identity(),
                            false,
                            Instant::now() + Duration::from_millis(100),
                        )
                        .unwrap(),
                );
                if self.close_on_poll {
                    self.host.close().unwrap();
                }
            }
            Poll::Pending
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let (acknowledgement, seek, _receiver) = ready_seek_with_receiver(&host, &mut worker, &mut ids);
    host.registry
        .transition(&acknowledgement, State::WrittenAwaitingAck, None)
        .unwrap();
    drop(seek);
    let mut writer = PendingLegacy {
        polls: 0,
        host: &host,
        close_on_poll,
        queued: None,
    };
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
        let mut lifetime=tokio::time::interval(Duration::from_millis(1));lifetime.tick().await;
        let legacy=Packet {wire:json!({"command":["set_property","pause",true],"request_id":ids.next().unwrap()}),deadline:Instant::now()+Duration::from_secs(5),permit:None,ticket:None};
        let failure=tokio::time::timeout(Duration::from_secs(2),super::super::ipc::write_packets(&mut writer,vec![legacy],&host,&mut lifetime)).await.expect("legacy Pending must not block guarded deadline or close").unwrap_err();
        assert_eq!(failure.downcast_ref::<std::io::Error>().unwrap().kind(),if close_on_poll {std::io::ErrorKind::Interrupted}else{std::io::ErrorKind::TimedOut});
        assert!(writer.queued.take().unwrap().wait().is_err());
        let original=host.registry.read(&acknowledgement,false).unwrap();assert_eq!(original.state,State::OutcomeUnobserved);assert!(original.committed);assert_eq!(original.identity,identity());
        if !close_on_poll {assert_eq!(original.failure.unwrap().stage,FailureStage::Deadline);}
        assert!(host.fence.is_closed());assert_eq!(host.fence.in_flight(),0);
    });
    assert!(writer.polls > 0);
}
#[test]
fn pending_legacy_write_cannot_starve_queued_or_awaiting_ack_deadline() {
    stalled_legacy_case(false);
}
#[test]
fn pending_legacy_write_observes_original_close_and_terminalizes_pending_results() {
    stalled_legacy_case(true);
}

#[test]
fn disabled_scope_never_issues_observation_authority() {
    let host = GuardedQueue::default();
    assert!(
        matches!(host.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::Unsupported)
    );
}

#[test]
fn scoped_control_close_retires_only_original_connection() {
    let scope = HostScope::testing();
    let (sender, _legacy) = tokio::sync::mpsc::channel(64);
    let handle =
        super::super::control::MpvControlHandle::new_scoped(sender, scope.clone()).unwrap();
    let mut original = identity();
    original.engine = handle.instance_id();
    let waiter = handle
        .state
        .guarded
        .observe(original, false, deadline())
        .unwrap();
    handle.state.close();
    assert_eq!(waiter.wait().unwrap_err().code, ErrorCode::Cancelled);
    let replacement = GuardedQueue::scoped(scope.clone(), 8).unwrap();
    assert!(
        replacement
            .fence
            .validate(replacement.fence.generation())
            .is_ok()
    );
    drop(scope);
    assert!(
        matches!(replacement.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::SourceExpired)
    );
}

#[test]
fn conditional_generation_and_claim_nonce_exhaustion_never_wrap() {
    let word = Fence::new();
    word.exhaust_generation_for_test();
    let last = word.generation();
    assert_eq!(
        word.advance_if(last).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert!(word.is_closed());
    assert_eq!(word.generation(), last);
    assert_eq!(word.advance().unwrap_err().code, ErrorCode::Cancelled);
    let host = test_host();
    host.registry.exhaust_nonce_for_test();
    for _ in 0..2 {
        assert!(
            matches!(host.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::ResourceExhausted)
        );
    }
}

#[test]
fn revision_exhaustion_blocks_ready_and_queued_authority() {
    for queued in [false, true] {
        let host = test_host();
        let mut worker = Worker::new();
        let mut ids = RequestIds::default();
        let mut receiver = host.take_receiver().unwrap();
        let guard = observe_guard(&host, &mut worker, &mut ids, &mut receiver);
        let target = evaluate_target(&guard);
        if queued {
            guard.submit_native(target).unwrap();
            let input = receiver.try_recv().unwrap();
            host.registry.exhaust_revision_for_test();
            assert_eq!(
                host.registry
                    .invalidate_observations(false)
                    .unwrap_err()
                    .code,
                ErrorCode::ResourceExhausted
            );
            assert!(
                worker
                    .begin(input, &mut ids, &host, Instant::now())
                    .unwrap()
                    .is_empty()
            );
        } else {
            host.registry.exhaust_revision_for_test();
            assert_eq!(
                host.registry
                    .invalidate_observations(false)
                    .unwrap_err()
                    .code,
                ErrorCode::ResourceExhausted
            );
            assert!(
                matches!(guard.submit_native(target),Err(error) if error.code==ErrorCode::ResourceExhausted)
            );
        }
        assert_eq!(host.fence.in_flight(), 0);
    }
}

#[test]
fn changed_sample_rejects_same_guard_before_admission_without_packets() {
    for event in [
        json!({"event":"property-change","name":"pause"}),
        json!({"event":"end-file"}),
    ] {
        let host = test_host();
        let mut worker = Worker::new();
        let mut ids = RequestIds::default();
        let mut receiver = host.take_receiver().unwrap();
        let guard = observe_guard(&host, &mut worker, &mut ids, &mut receiver);
        let target = evaluate_target(&guard);
        worker
            .receive(&event, Some((1, 2)), &mut ids, &host, Instant::now())
            .unwrap();
        assert!(
            matches!(guard.submit_native(target),Err(error) if error.code==ErrorCode::RevisionConflict)
        );
        assert!(receiver.try_recv().is_err());
        assert_eq!(host.fence.in_flight(), 0);
    }
}

#[test]
fn public_diagnostic_mutation_cannot_retarget_original_guard_or_view() {
    use crate::application::playback_session::test_guarded_observation;
    use crate::domain::api_values::DecimalU64;
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let mut receiver = host.take_receiver().unwrap();
    let guard = observe_guard(&host, &mut worker, &mut ids, &mut receiver);
    let mut diagnostic = test_guarded_observation(Default::default());
    diagnostic.guard = Some(guard);
    diagnostic.engine_instance_id = DecimalU64(999);
    diagnostic.resolution_artifact_id = "foreign".into();
    diagnostic.native.position_seconds.value = Some(999.);
    diagnostic.observation_authorized = false;
    let (view, guard) = diagnostic.into_guarded_parts().unwrap();
    assert_eq!(view.identity().engine_instance_id(), DecimalU64(7));
    assert_eq!(view.playback().position_us, Some(DecimalU64(3_000_000)));
    let target = evaluate_target(&guard);
    let submission = guard.submit_native(target).unwrap();
    assert_eq!(submission.receipt().engine_instance_id.0, 7);
    let queue::Envelope::Submit(input) = receiver.try_recv().unwrap() else {
        panic!("same guard Submit expected")
    };
    assert_eq!(input.identity, identity());
    assert_eq!(input.position_us, 4_000_000);
    assert!(receiver.try_recv().is_err());
}

#[test]
fn changed_sample_after_enqueue_rejects_at_original_commit() {
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let mut receiver = host.take_receiver().unwrap();
    let (_, input) = queued_seek(&host, &mut worker, &mut ids, &mut receiver);
    let ticket = input.ticket.clone();
    worker
        .receive(
            &json!({"event":"property-change","name":"time-pos"}),
            Some((1, 2)),
            &mut ids,
            &host,
            Instant::now(),
        )
        .unwrap();
    assert!(
        worker
            .begin(
                queue::Envelope::Submit(input),
                &mut ids,
                &host,
                Instant::now()
            )
            .unwrap()
            .is_empty()
    );
    let result = host.registry.read(&ticket, false).unwrap();
    assert!(!result.committed);
    assert_eq!(result.state, State::RejectedBeforeWrite);
    assert_eq!(host.fence.in_flight(), 0);
}

#[test]
fn grants_and_commands_share_capacity_without_evicting_terminal_result() {
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let mut receiver = host.take_receiver().unwrap();
    let (_, input) = queued_seek(&host, &mut worker, &mut ids, &mut receiver);
    let ticket = input.ticket;
    host.registry
        .transition(&ticket, State::RejectedBeforeWrite, None)
        .unwrap();
    let mut waiters = Vec::new();
    for _ in 0..63 {
        waiters.push(host.observe(identity(), false, deadline()).unwrap());
    }
    assert!(
        matches!(host.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::ResourceExhausted)
    );
    drop(waiters.pop());
    host.registry.reap_observations(Instant::now()).unwrap();
    let replacement = host.observe(identity(), false, deadline()).unwrap();
    assert_eq!(
        host.registry.read(&ticket, false).unwrap().state,
        State::RejectedBeforeWrite
    );
    drop(replacement);
}

#[test]
fn abandoned_ready_guard_releases_only_its_original_grant() {
    let host = test_host();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let mut receiver = host.take_receiver().unwrap();
    let guard = observe_guard(&host, &mut worker, &mut ids, &mut receiver);
    let tickets: Vec<_> = (0..63)
        .map(|_| host.registry.reserve(identity()).unwrap())
        .collect();
    assert!(host.registry.reserve(identity()).is_err());
    drop(guard);
    host.registry.reap_observations(Instant::now()).unwrap();
    host.registry.reserve(identity()).unwrap();
    for ticket in tickets {
        assert_eq!(
            host.registry.read(&ticket, false).unwrap().state,
            State::Queued
        );
    }
}

#[test]
fn host_expiry_invalidates_original_ticket_despite_live_registry_and_connection() {
    let scope = HostScope::testing();
    let host = GuardedQueue::scoped(scope.clone(), 7).unwrap();
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    let mut receiver = host.take_receiver().unwrap();
    let (submission, _) = queued_seek(&host, &mut worker, &mut ids, &mut receiver);
    let (_, ticket) = submission.into_parts();
    let registry = host.registry.clone();
    drop(scope);
    assert_eq!(
        ticket.read_original(false).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    assert!(
        matches!(host.observe(identity(),false,deadline()),Err(error) if error.code==ErrorCode::SourceExpired)
    );
    drop(registry);
}

#[test]
fn late_old_connection_close_preserves_new_generation_and_original_permit_count() {
    let scope = HostScope::testing();
    let old = GuardedQueue::scoped(scope.clone(), 7).unwrap();
    let permit = scope.word.commit(old.fence.generation()).unwrap();
    let current = GuardedQueue::scoped(scope.clone(), 8).unwrap();
    let generation = current.fence.generation();
    old.close().unwrap();
    assert_eq!(scope.word.generation(), generation);
    assert!(current.fence.validate(generation).is_ok());
    assert_eq!(scope.word.in_flight(), 1);
    drop(permit);
    assert_eq!(scope.word.in_flight(), 0);
    current.close().unwrap();
    let reopened = GuardedQueue::scoped(scope.clone(), 9).unwrap();
    assert!(reopened.fence.validate(reopened.fence.generation()).is_ok());
}
