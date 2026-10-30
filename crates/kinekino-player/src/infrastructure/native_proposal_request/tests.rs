use super::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let path = std::env::temp_dir().join(format!(
            "player-native-request-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn ordinary_bytes_are_read_without_json_or_utf8_policy_and_exact_limit_is_supported() {
    let fixture = Fixture::new();
    let path = fixture.0.join("request.json");
    fs::write(&path, [0xff, 0, b'{']).unwrap();
    assert_eq!(
        read_native_proposal_request(&path).unwrap(),
        [0xff, 0, b'{']
    );
    fs::write(&path, vec![b' '; MAX_NATIVE_PROPOSAL_REQUEST_BYTES]).unwrap();
    assert_eq!(
        read_native_proposal_request(&path).unwrap().len(),
        MAX_NATIVE_PROPOSAL_REQUEST_BYTES
    );
    fs::write(&path, vec![b' '; MAX_NATIVE_PROPOSAL_REQUEST_BYTES + 1]).unwrap();
    assert_eq!(
        read_native_proposal_request(&path).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
}

#[test]
fn relative_missing_directory_and_parent_traversal_are_rejected() {
    let fixture = Fixture::new();
    assert_eq!(
        read_native_proposal_request(Path::new("request.json"))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        read_native_proposal_request(&fixture.0.join("missing.json"))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    assert!(read_native_proposal_request(&fixture.0).is_err());
    assert_eq!(
        read_native_proposal_request(&fixture.0.join("..").join("request.json"))
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
}
