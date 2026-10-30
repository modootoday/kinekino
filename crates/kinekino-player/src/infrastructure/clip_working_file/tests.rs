use super::*;
use serde_json::json;
use std::{fs, io::Read};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
#[test]
#[ignore = "Actual Windows original-handle file lifetime; fresh evidence and fixed TMP, no GUI/models"]
fn original_delete_on_close_waits_for_readers_and_never_removes_foreign_names() {
    let evidence =
        PathBuf::from(std::env::var_os("CLIP_WORKING_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&evidence).unwrap();
    let sentinel = evidence.join("original-sentinel.txt");
    fs::write(&sentinel, b"original evidence is preserved").unwrap();
    let root = std::env::temp_dir();
    let mut working = ClipWorkingFile::create(&root, 262144, &Control).unwrap();
    let bytes = b"new ephemeral clip working input";
    let facts = working.seal(bytes, &Control).unwrap();
    let path = working.path().to_path_buf();
    assert!(OpenOptions::new().write(true).open(&path).is_err());
    assert!(fs::rename(&path, path.with_extension("moved")).is_err());
    let mut reader = working.with_reader(Ok).unwrap();
    fs::write(
        evidence.join("input-facts.json"),
        serde_json::to_vec_pretty(&facts).unwrap(),
    )
    .unwrap();
    drop(working);
    let mut observed = Vec::new();
    reader.read_to_end(&mut observed).unwrap();
    assert_eq!(observed, bytes);
    drop(reader);
    assert!(!path.exists());
    fs::write(&path, b"later independently created file").unwrap();
    assert_eq!(
        fs::read(&path).unwrap(),
        b"later independently created file"
    );
    let mut partial = ClipWorkingFile::create(&root, 262144, &Control).unwrap();
    let partial_path = partial.path().to_path_buf();
    struct CancelAfterWrite(PathBuf);
    impl MutationControl for CancelAfterWrite {
        fn checkpoint(&self) -> Result<(), ApiError> {
            if fs::metadata(&self.0).is_ok_and(|m| m.len() > 0) {
                Err(ApiError::new(
                    ErrorCode::Cancelled,
                    "cancel after original partial write",
                ))
            } else {
                Ok(())
            }
        }
    }
    let primary = partial
        .seal(&vec![7; 131072], &CancelAfterWrite(partial_path.clone()))
        .unwrap_err();
    assert_eq!(primary.code, ErrorCode::Cancelled);
    let length = partial.file.metadata().unwrap().len();
    assert!(length > 0 && length < 131072);
    assert_eq!(
        partial.seal(b"retry", &Control).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    drop(partial);
    assert!(!partial_path.exists());
    let mut bounded = ClipWorkingFile::create(&root, 8, &Control).unwrap();
    let bounded_path = bounded.path().to_path_buf();
    assert_eq!(
        bounded.seal(&[0; 9], &Control).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    drop(bounded);
    assert!(!bounded_path.exists());
    assert_eq!(
        fs::read(&sentinel).unwrap(),
        b"original evidence is preserved"
    );
    fs::write(evidence.join("verification.json"),serde_json::to_vec_pretty(&json!({"original_file_write_and_rename_denied":true,"independent_read_cursor_verified":true,"readers_outlive_parent_close_without_losing_bytes":true,"deleted_after_last_reader_closed":true,"cancelled_partial_and_oversized_new_files_deleted":true,"foreign_replacement_path":path,"foreign_replacement_and_original_evidence_preserved":true,"cancelled_primary_error_preserved":true,"jobs_or_full_cleanup_verified":false})).unwrap()).unwrap();
}
