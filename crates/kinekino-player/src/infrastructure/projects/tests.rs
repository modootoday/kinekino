use super::*;
use std::sync::{Arc, Barrier};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "player-projects-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn project(revision: u64) -> ProjectRevision {
    ProjectRevision {
        schema_version: "1".into(),
        project_id: "work".into(),
        revision: DecimalU64(revision),
        parent_revision: (revision > 1).then(|| DecimalU64(revision - 1)),
        title: "Watch".into(),
        watching: Some(WatchingSelection {
            label: "Episode".into(),
            source: WatchingSource::EpisodePage {
                source_id: "episode-a".into(),
                episode_url: "https://catalog.example.test/e/a".into(),
            },
        }),
        proposals: vec![],
        work_state: ProjectWorkState::default(),
    }
}
#[test]
fn restart_competing_writers_and_corrupt_latest_never_fall_back() {
    let root = Temp::new();
    ProjectFiles::open(root.path())
        .unwrap()
        .publish(None, &project(1))
        .unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let mut writers = Vec::new();
    for _ in 0..2 {
        let root = root.path().to_owned();
        let barrier = barrier.clone();
        writers.push(std::thread::spawn(move || {
            let store = ProjectFiles::open(&root).unwrap();
            barrier.wait();
            store.publish(Some(1), &project(2))
        }));
    }
    let results: Vec<_> = writers
        .into_iter()
        .map(|writer| writer.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| result
                .as_ref()
                .is_err_and(|error| error.code == ErrorCode::RevisionConflict))
            .count(),
        1
    );
    let store = ProjectFiles::open(root.path()).unwrap();
    assert_eq!(store.load("work", Some(1)).unwrap().latest_revision.0, 2);
    fs::create_dir(root.path().join("p-work/r-3")).unwrap();
    assert_eq!(
        store.load("work", None).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        store.load("work", Some(1)).unwrap_err().code,
        ErrorCode::NeedsInput
    );
}
#[test]
fn invalid_and_oversized_content_does_not_reserve_identity() {
    let root = Temp::new();
    let store = ProjectFiles::open(root.path()).unwrap();
    let mut invalid = project(1);
    invalid.title = "x".repeat(MAX_RECORD_BYTES + 1);
    assert_eq!(
        store.publish(None, &invalid).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert!(!root.path().join("p-work").exists());
    invalid = project(1);
    invalid.watching = None;
    assert!(store.publish(None, &invalid).is_err());
    assert!(!root.path().join("p-work").exists());
}
#[test]
fn altered_hash_extra_file_and_noncontiguous_history_fail_closed() {
    for mode in ["changed", "extra", "gap"] {
        let root = Temp::new();
        let store = ProjectFiles::open(root.path()).unwrap();
        store.publish(None, &project(1)).unwrap();
        let directory = root.path().join("p-work/r-1");
        match mode {
            "changed" => {
                let file = fs::read_dir(&directory)
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path();
                let bytes = fs::read_to_string(&file).unwrap().replace("Watch", "Other");
                fs::write(file, bytes).unwrap();
            }
            "extra" => {
                fs::write(directory.join("extra.json"), b"{}").unwrap();
            }
            "gap" => {
                fs::create_dir(root.path().join("p-work/r-3")).unwrap();
            }
            _ => unreachable!(),
        }
        assert_eq!(
            store.load("work", None).unwrap_err().code,
            ErrorCode::NeedsInput,
            "{mode}"
        );
    }
}
