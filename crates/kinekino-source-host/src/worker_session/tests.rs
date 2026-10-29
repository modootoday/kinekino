use super::*;
use reqwest::ResponseBuilderExt;
fn target() -> url::Url {
    url::Url::parse("https://example.invalid/media").unwrap()
}
fn control(sequence: u64, kind: Kind, payload: Vec<u8>) -> Frame {
    Frame {
        binding: [7; 32],
        sequence,
        offset: 0,
        kind,
        payload,
    }
}
fn open(session: &mut WorkerSession) {
    let mut cutoffs = 1000u64.to_le_bytes().to_vec();
    cutoffs.extend_from_slice(&2000u64.to_le_bytes());
    session
        .control(&control(0, Kind::Open, cutoffs), 1)
        .unwrap();
    let mut cx = Context::from_waker(std::task::Waker::noop());
    assert_eq!(
        session.poll_open(&mut cx, 2, || Ok(())),
        Poll::Ready(Ok(()))
    );
}
fn fixture(body: Vec<u8>) -> WorkerSession {
    let bootstrap = HttpBootstrap::new([7; 32], 100000, target().as_str(), None).unwrap();
    let mut session = WorkerSession::new(bootstrap, Cutoffs::new(1000, 2000).unwrap()).unwrap();
    let response: reqwest::Response = http::Response::builder()
        .url(target())
        .header("content-length", body.len())
        .body(body)
        .unwrap()
        .into();
    session.bootstrap = None;
    session.send = Some(HttpSendOwner::response_fixture(response, 100000, target()));
    session
}
fn drain(session: &mut WorkerSession) -> Frame {
    let mut bytes = Vec::new();
    while let Some(offered) = session.offer_write(8192).unwrap() {
        let completed = offered.len().min(1111);
        bytes.extend_from_slice(&offered[..completed]);
        session.complete_write(completed).unwrap();
    }
    crate::codec::decode_exact(&bytes).unwrap()
}
#[test]
fn full_worker_memory_flow_obeys_credit_partial_writes_and_real_response_eof() {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            let original: Vec<u8> = (0..70000).map(|i| (i % 251) as u8).collect();
            let mut session = fixture(original.clone());
            open(&mut session);
            assert!(!session.can_read());
            assert_eq!(drain(&mut session).kind, Kind::Metadata);
            assert!(!session.can_read());
            let mut received = Vec::new();
            for sequence in 1..=7 {
                session
                    .control(
                        &control(sequence, Kind::Credit, 10000u64.to_le_bytes().to_vec()),
                        4,
                    )
                    .unwrap();
                if sequence == 1 {
                    assert!(session.can_read());
                    session.read_body(|| Ok(())).await.unwrap();
                }
                assert!(!session.can_read());
                assert!(session.pump_body(4).unwrap());
                let frame = drain(&mut session);
                assert_eq!(frame.kind, Kind::Data);
                assert_eq!(frame.offset, received.len() as u64);
                received.extend_from_slice(&frame.payload);
            }
            assert_eq!(received, original);
            assert!(!session.body_end_written());
            // EOF discovery is also forbidden until a fresh credit admits a body poll.
            assert!(!session.can_read());
            session
                .control(&control(8, Kind::Credit, 1u64.to_le_bytes().to_vec()), 5)
                .unwrap();
            session.read_body(|| Ok(())).await.unwrap();
            assert!(session.pump_body(6).unwrap());
            assert!(!session.body_end_written());
            assert_eq!(drain(&mut session).kind, Kind::BodyEnd);
            assert!(session.body_end_written());
        });
}
#[test]
fn control_abort_is_available_during_pending_wire_write_and_retains_owners() {
    let mut session = fixture(vec![1; 10]);
    open(&mut session);
    assert!(session.offer_write(8).unwrap().is_some());
    session
        .control(&control(1, Kind::Abort, Vec::new()), 3)
        .unwrap();
    assert!(session.outgoing.is_some());
    assert!(session.send.is_some());
    assert!(session.offer_write(8).is_err());
    assert!(!session.can_read());
    assert!(!session.body_end_written());
}
#[test]
fn original_body_guard_invalidation_is_sticky_and_keeps_same_http_owner() {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            let mut session = fixture(vec![1; 10]);
            open(&mut session);
            drain(&mut session);
            session
                .control(&control(1, Kind::Credit, 10u64.to_le_bytes().to_vec()), 3)
                .unwrap();
            let mut checks = 0;
            assert_eq!(
                session
                    .read_body(|| {
                        checks += 1;
                        if checks == 2 {
                            Err(SendError::Invalidated)
                        } else {
                            Ok(())
                        }
                    })
                    .await,
                Err(WorkerError::Http(SendError::Invalidated))
            );
            assert!(session.send.is_some());
            assert!(!session.can_read());
            session.abort();
            assert!(session.send.is_some());
            assert_eq!(
                session.pump_body(4),
                Err(WorkerError::Http(SendError::Invalidated))
            );
        });
}

#[test]
fn bounded_failure_at_frame_boundary_preserves_first_error_after_partial_writes() {
    let mut session = fixture(vec![1; 10]);
    open(&mut session);
    drain(&mut session);
    let primary = WorkerError::Http(SendError::Invalidated);
    assert_eq!(session.begin_failure(primary, 3), Ok(true));
    assert!(!session.failure_written());
    assert_eq!(session.begin_failure(WorkerError::Order, 3), Ok(false));
    let frame = drain(&mut session);
    assert_eq!(frame.kind, Kind::Failure);
    assert_eq!(frame.sequence, 1);
    assert_eq!(frame.payload, b"source invalidated");
    assert!(session.failure_written());
    assert!(!session.body_end_written());
    assert_eq!(session.first_failure(), Some(primary));
    session.abort();
    assert_eq!(session.first_failure(), Some(primary));
    assert!(session.send.is_some());
}
#[test]
fn failure_never_splices_a_partially_written_frame_or_extends_end_cutoff() {
    let mut session = fixture(vec![1; 10]);
    open(&mut session);
    let offered = session.offer_write(8).unwrap().unwrap().to_vec();
    assert_eq!(session.begin_failure(WorkerError::Order, 3), Ok(false));
    assert_eq!(session.outgoing.as_ref().unwrap().kind, Kind::Metadata);
    assert!(session.outgoing.is_some());
    assert_eq!(offered.len(), 8);
    assert!(session.offer_write(8).is_err());
    assert!(!session.failure_written());
    let mut expired = fixture(vec![1]);
    open(&mut expired);
    drain(&mut expired);
    assert_eq!(
        expired.begin_failure(WorkerError::Protocol(ProtocolError::Cutoff), 2000),
        Ok(false)
    );
    assert!(expired.outgoing.is_none());
    assert!(expired.send.is_some());
}
#[test]
fn poisoned_exchange_secondary_error_does_not_replace_original_primary() {
    let mut session = fixture(vec![1]);
    open(&mut session);
    drain(&mut session);
    let mut wrong = control(1, Kind::Credit, 1u64.to_le_bytes().to_vec());
    wrong.binding = [0; 32];
    assert_eq!(
        session.control(&wrong, 3),
        Err(WorkerError::Protocol(ProtocolError::Binding))
    );
    assert_eq!(
        session.begin_failure(WorkerError::Order, 4),
        Err(WorkerError::Protocol(ProtocolError::Order))
    );
    assert_eq!(
        session.first_failure(),
        Some(WorkerError::Protocol(ProtocolError::Binding))
    );
    assert!(!session.failure_written());
}

#[cfg(feature = "parent-stream")]
#[test]
fn parent_and_worker_share_metadata_credit_offsets_and_only_explicit_end() {
    use crate::parent_stream::{ParentStream, ReadStatus};
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            let original: Vec<u8> = (0..70000).map(|n| (n % 251) as u8).collect();
            let mut worker = fixture(original.clone());
            let request = HttpBootstrap::new([7; 32], 100000, target().as_str(), None).unwrap();
            let mut parent =
                ParentStream::new(&request, Cutoffs::new(1000, 2000).unwrap()).unwrap();
            worker.control(&parent.open_frame(1).unwrap(), 1).unwrap();
            let mut cx = Context::from_waker(std::task::Waker::noop());
            assert_eq!(worker.poll_open(&mut cx, 2, || Ok(())), Poll::Ready(Ok(())));
            parent.receive(drain(&mut worker), 2).unwrap();
            assert_eq!(parent.metadata().unwrap().response_length(), Some(70000));
            worker
                .control(&parent.credit_frame(100000, 3).unwrap(), 3)
                .unwrap();
            let mut data_frames = 0;
            while !worker.body_end_written() {
                if worker.can_read() {
                    worker.read_body(|| Ok(())).await.unwrap();
                }
                assert!(worker.can_pump());
                assert!(worker.pump_body(4).unwrap());
                let frame = drain(&mut worker);
                if frame.kind == Kind::Data {
                    data_frames += 1;
                    assert!(frame.payload.len() <= crate::domain::MAX_DATA_BYTES);
                }
                assert!(parent.can_receive());
                parent.receive(frame, 4).unwrap();
            }
            assert_eq!(data_frames, 2);
            let mut received = Vec::new();
            let mut output = [0; 5000];
            loop {
                match parent.read(&mut output).unwrap() {
                    ReadStatus::Data(count) => received.extend_from_slice(&output[..count]),
                    ReadStatus::End => break,
                    ReadStatus::Pending => panic!("BodyEnd was already delivered"),
                }
            }
            assert_eq!(received, original);
            assert_eq!(parent.transport_eof(), Ok(()));
        });
}
