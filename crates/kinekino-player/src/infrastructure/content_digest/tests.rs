use super::{DigestFailure, sha256, sha256_with_checkpoint};
use crate::application::api::{ApiError, ErrorCode};
use std::{
    cell::Cell,
    io::{self, Cursor, Read},
};

#[test]
fn legacy_and_checkpoint_hashes_match_empty_and_known_vectors() {
    for (input, expected) in [
        (
            b"".as_slice(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".as_slice(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
    ] {
        let bytes = input.len() as u64;
        assert_eq!(
            sha256(&mut Cursor::new(input), bytes).unwrap(),
            (bytes, expected.into())
        );
        let checkpoints = Cell::new(0);
        let result = sha256_with_checkpoint(&mut Cursor::new(input), bytes, || {
            checkpoints.set(checkpoints.get() + 1);
            Ok(())
        })
        .unwrap();
        assert_eq!(result, (bytes, expected.into()));
        assert_eq!(checkpoints.get(), if input.is_empty() { 1 } else { 2 });
    }
}
#[test]
fn exact_byte_bound_and_one_more_preserve_checkpoint_boundary() {
    let input = vec![7; 65536];
    let calls = Cell::new(0);
    let exact = sha256_with_checkpoint(&mut Cursor::new(&input), 65536, || {
        calls.set(calls.get() + 1);
        Ok(())
    })
    .unwrap();
    assert_eq!(exact.0, 65536);
    assert_eq!(calls.get(), 2, "The EOF read retains its checkpoint");
    let oversized = sha256_with_checkpoint(&mut Cursor::new(&input), 65535, || Ok(())).unwrap_err();
    assert!(matches!(
        oversized,
        DigestFailure::ByteBoundExceeded {
            maximum: 65535,
            observed: 65536
        }
    ));
    assert_eq!(
        sha256(&mut Cursor::new(&input), 65535).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
}
struct CountingReader<'a> {
    reads: &'a Cell<usize>,
}
impl Read for CountingReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        output.fill(7);
        Ok(output.len())
    }
}
fn stopped_before_second_read(code: ErrorCode) {
    let reads = Cell::new(0);
    let calls = Cell::new(0);
    let mut original = ApiError::new(code, "Original checkpoint error");
    original.details = serde_json::json!({"original_attempt":"retained"});
    original.remediation = "original remediation".into();
    original.request_id = "checkpoint-origin".into();
    let expected = serde_json::to_value(&original).unwrap();
    let mut original = Some(original);
    let error = sha256_with_checkpoint(&mut CountingReader { reads: &reads }, 131072, || {
        calls.set(calls.get() + 1);
        if calls.get() == 2 {
            Err(original.take().unwrap())
        } else {
            Ok(())
        }
    })
    .unwrap_err();
    let DigestFailure::Checkpoint(error) = error else {
        panic!("Original checkpoint failure required");
    };
    assert_eq!(serde_json::to_value(error).unwrap(), expected);
    assert_eq!(reads.get(), 1);
    assert_eq!(calls.get(), 2);
}
#[test]
fn cancellation_preserves_original_error_before_next_read() {
    stopped_before_second_read(ErrorCode::Cancelled);
}
#[test]
fn timeout_preserves_original_error_before_next_read() {
    stopped_before_second_read(ErrorCode::Timeout);
}
struct FailedReader;
impl Read for FailedReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        // 2 is "not found" on both Win32 (ERROR_FILE_NOT_FOUND) and POSIX (ENOENT).
        Err(io::Error::from_raw_os_error(2))
    }
}
#[test]
fn typed_read_failure_retains_original_kind_and_os_code() {
    let error = sha256_with_checkpoint(&mut FailedReader, 1, || Ok(())).unwrap_err();
    let DigestFailure::Read(error) = error else {
        panic!("Original read error required");
    };
    assert_eq!(error.raw_os_error(), Some(2));
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}
#[test]
fn legacy_read_failure_keeps_previous_public_error_shape() {
    let error = sha256(&mut FailedReader, 1).unwrap_err();
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.message, "Content identity verification failed");
    assert_eq!(error.details, serde_json::json!({}));
}
