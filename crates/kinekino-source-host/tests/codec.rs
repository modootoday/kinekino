use kinekino_source_host::domain::*;
use kinekino_source_host::{
    codec,
    domain::state::{BodyStatus, Direction, Exchange},
};
use std::io::{self, Cursor, Read};

fn frame(kind: Kind, sequence: u64, offset: u64, payload: Vec<u8>) -> Frame {
    Frame {
        binding: [7; 32],
        kind,
        sequence,
        offset,
        payload,
    }
}
fn open() -> Frame {
    let mut payload = 100u64.to_le_bytes().to_vec();
    payload.extend_from_slice(&120u64.to_le_bytes());
    frame(Kind::Open, 0, 0, payload)
}
fn streaming() -> Exchange {
    let mut state = Exchange::new([7; 32], Cutoffs::new(100, 120).unwrap()).unwrap();
    state.accept(Direction::Parent, &open(), 1).unwrap();
    state
        .accept(Direction::Child, &frame(Kind::Metadata, 0, 0, vec![1]), 2)
        .unwrap();
    state
}

#[test]
fn kind_and_length_limits_are_explicit() {
    assert_eq!(Kind::from_byte(0), Err(ProtocolError::Framing));
    assert_eq!(Kind::from_byte(8), Err(ProtocolError::Framing));
    assert!(Kind::Data.check_length(MAX_DATA_BYTES).is_ok());
    assert_eq!(
        Kind::Data.check_length(MAX_DATA_BYTES + 1),
        Err(ProtocolError::Limit)
    );
    assert!(Kind::Metadata.check_length(MAX_METADATA_BYTES).is_ok());
    assert!(Kind::Metadata.check_length(MAX_METADATA_BYTES + 1).is_err());
    assert!(Kind::Data.check_length(0).is_err());
    assert!(Kind::Open.check_length(16).is_ok());
    assert!(Kind::Open.check_length(17).is_err());
    assert!(Kind::Credit.check_length(8).is_ok());
    assert!(Kind::Abort.check_length(0).is_ok());
    assert!(Kind::BodyEnd.check_length(1).is_err());
}

#[test]
fn cutoff_values_cannot_be_renewed_by_intersection() {
    let original = Cutoffs::new(100, 120).unwrap();
    assert_eq!(
        original
            .constrained_by(Cutoffs::new(200, 240).unwrap())
            .unwrap(),
        original
    );
    assert_eq!(
        original
            .constrained_by(Cutoffs::new(80, 110).unwrap())
            .unwrap(),
        Cutoffs::new(80, 110).unwrap()
    );
    assert!(Cutoffs::new(0, 10).is_err());
    assert!(Cutoffs::new(10, 10).is_err());
}

#[test]
fn fixed_header_roundtrip_rejects_version_flags_and_trailing_bytes() {
    let original = frame(Kind::Data, 1, 0, vec![3; MAX_DATA_BYTES]);
    let bytes = codec::encode(&original).unwrap();
    assert_eq!(codec::decode_exact(&bytes).unwrap(), original);
    for index in [0, 4, 6, 7] {
        let mut changed = bytes.clone();
        changed[index] = 255;
        assert!(codec::decode_exact(&changed).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(codec::decode_exact(&trailing), Err(ProtocolError::Framing));
}

struct HeaderOnly {
    header: Cursor<Vec<u8>>,
}
impl Read for HeaderOnly {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        assert!(
            self.header.position() < HEADER_BYTES as u64,
            "oversized declaration must not read body"
        );
        self.header.read(output)
    }
}
#[test]
fn oversized_prefix_fails_before_reading_or_allocating_the_declared_body() {
    let mut bytes = codec::encode(&frame(Kind::Data, 1, 0, vec![1])).unwrap();
    bytes.truncate(HEADER_BYTES);
    bytes[56..60].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        codec::read_frame(&mut HeaderOnly {
            header: Cursor::new(bytes)
        }),
        Err(ProtocolError::Limit)
    );
}

#[test]
fn transport_eof_truncation_and_body_end_are_distinct() {
    assert_eq!(
        codec::read_frame(&mut Cursor::new(Vec::<u8>::new())),
        Ok(None)
    );
    let bytes = codec::encode(&open()).unwrap();
    for length in 1..bytes.len() {
        assert_eq!(
            codec::decode_exact(&bytes[..length]),
            Err(ProtocolError::Truncated)
        );
    }
    let mut state = streaming();
    assert_eq!(
        codec::read_frame(&mut Cursor::new(Vec::<u8>::new())),
        Ok(None)
    );
    assert_eq!(state.status(), BodyStatus::Streaming);
    state
        .accept(Direction::Child, &frame(Kind::BodyEnd, 1, 0, vec![]), 3)
        .unwrap();
    assert_eq!(state.status(), BodyStatus::Ended); // Only body status, no physical ACK exists.
}

#[test]
fn sequence_credit_and_offsets_are_consumed_once() {
    let mut state = streaming();
    state
        .accept(
            Direction::Parent,
            &frame(Kind::Credit, 1, 0, 3u64.to_le_bytes().to_vec()),
            3,
        )
        .unwrap();
    state
        .accept(Direction::Child, &frame(Kind::Data, 1, 0, vec![1, 2, 3]), 4)
        .unwrap();
    assert_eq!(state.credit(), 0);
    assert_eq!(state.offset(), 3);
    assert_eq!(
        state.accept(Direction::Child, &frame(Kind::Data, 2, 3, vec![4]), 5),
        Err(ProtocolError::Credit)
    );
    assert_eq!(state.offset(), 3);
    assert!(
        state
            .accept(
                Direction::Parent,
                &frame(Kind::Credit, 2, 0, 1u64.to_le_bytes().to_vec()),
                6
            )
            .is_err()
    );
}

#[test]
fn credit_overflow_and_replay_fail_closed() {
    for amount in [0, MAX_CREDIT_BYTES + 1, u64::MAX] {
        assert_eq!(
            streaming().accept(
                Direction::Parent,
                &frame(Kind::Credit, 1, 0, amount.to_le_bytes().to_vec()),
                3
            ),
            Err(ProtocolError::Credit)
        );
    }
    let mut state = streaming();
    let credit = frame(Kind::Credit, 1, 0, MAX_CREDIT_BYTES.to_le_bytes().to_vec());
    state.accept(Direction::Parent, &credit, 3).unwrap();
    assert_eq!(
        state.accept(Direction::Parent, &credit, 4),
        Err(ProtocolError::Order)
    );
}

#[test]
fn wrong_binding_direction_offset_and_order_are_rejected() {
    let mut other = frame(Kind::Credit, 1, 0, 1u64.to_le_bytes().to_vec());
    other.binding = [8; 32];
    assert_eq!(
        streaming().accept(Direction::Parent, &other, 3),
        Err(ProtocolError::Binding)
    );
    assert!(streaming().accept(Direction::Child, &open(), 3).is_err());
    assert!(
        streaming()
            .accept(Direction::Child, &frame(Kind::Metadata, 1, 0, vec![1]), 3)
            .is_err()
    );
    assert!(
        streaming()
            .accept(Direction::Child, &frame(Kind::BodyEnd, 1, 1, vec![]), 3)
            .is_err()
    );
    let mut unopened = Exchange::new([7; 32], Cutoffs::new(100, 120).unwrap()).unwrap();
    assert!(
        unopened
            .accept(Direction::Child, &frame(Kind::Data, 0, 0, vec![1]), 1)
            .is_err()
    );
}

#[test]
fn absolute_cutoff_never_restarts_at_receive_and_control_needs_no_credit() {
    let mut state = Exchange::new([7; 32], Cutoffs::new(90, 110).unwrap()).unwrap();
    assert_eq!(
        state.accept(Direction::Parent, &open(), 1),
        Err(ProtocolError::Cutoff)
    );
    assert_eq!(
        streaming().accept(Direction::Child, &frame(Kind::BodyEnd, 1, 0, vec![]), 100),
        Err(ProtocolError::Cutoff)
    );
    let mut aborted = streaming();
    aborted
        .accept(Direction::Parent, &frame(Kind::Abort, 1, 0, vec![]), 121)
        .unwrap();
    assert_eq!(aborted.status(), BodyStatus::Aborted);
    assert_eq!(aborted.cutoffs(), Cutoffs::new(100, 120).unwrap());
}

#[test]
fn failure_is_bounded_utf8_and_does_not_authorize_late_success() {
    assert!(codec::encode(&frame(Kind::Failure, 0, 0, vec![255])).is_err());
    assert!(
        codec::encode(&frame(
            Kind::Failure,
            0,
            0,
            vec![b'x'; MAX_FAILURE_BYTES + 1]
        ))
        .is_err()
    );
    let mut state = streaming();
    state
        .accept(
            Direction::Child,
            &frame(Kind::Failure, 1, 0, b"original failure".to_vec()),
            3,
        )
        .unwrap();
    assert_eq!(state.status(), BodyStatus::Failed);
    assert!(
        state
            .accept(Direction::Child, &frame(Kind::BodyEnd, 2, 0, vec![]), 4)
            .is_err()
    );
}

#[test]
fn two_bounded_frames_can_carry_a_source_chunk_larger_than_64kib() {
    let mut state = streaming();
    state
        .accept(
            Direction::Parent,
            &frame(
                Kind::Credit,
                1,
                0,
                (MAX_DATA_BYTES as u64 + 1).to_le_bytes().to_vec(),
            ),
            3,
        )
        .unwrap();
    for data in [
        frame(Kind::Data, 1, 0, vec![1; MAX_DATA_BYTES]),
        frame(Kind::Data, 2, MAX_DATA_BYTES as u64, vec![2]),
    ] {
        let decoded = codec::decode_exact(&codec::encode(&data).unwrap()).unwrap();
        state.accept(Direction::Child, &decoded, 4).unwrap();
    }
    state
        .accept(
            Direction::Child,
            &frame(Kind::BodyEnd, 3, MAX_DATA_BYTES as u64 + 1, vec![]),
            5,
        )
        .unwrap();
    assert_eq!(state.offset(), MAX_DATA_BYTES as u64 + 1);
}

#[test]
fn clock_rollback_and_terminal_reopening_are_rejected() {
    assert_eq!(
        streaming().accept(
            Direction::Parent,
            &frame(Kind::Credit, 1, 0, 1u64.to_le_bytes().to_vec()),
            1
        ),
        Err(ProtocolError::Cutoff)
    );
    assert_eq!(
        streaming().accept(
            Direction::Child,
            &frame(Kind::Data, u64::MAX, 0, vec![1]),
            3
        ),
        Err(ProtocolError::Order)
    );
    let mut state = streaming();
    state
        .accept(Direction::Child, &frame(Kind::BodyEnd, 1, 0, vec![]), 3)
        .unwrap();
    assert!(
        state
            .accept(
                Direction::Parent,
                &frame(Kind::Credit, 1, 0, 1u64.to_le_bytes().to_vec()),
                4
            )
            .is_err()
    );
}
