use super::*;
use crate::{application::proposals::ProposalStore, domain::projects::canonical_sha256};
use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
impl Fixture {
    fn create() -> Self {
        let root = std::env::temp_dir().join(format!(
            "proposal-origin-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        eprintln!("proposal_origin_fixture_root={}", root.display());
        Self(root)
    }
    fn backend(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::create_dir(&path).unwrap();
        path
    }
}
fn seed(store: &ProposalFiles) {
    let state = crate::domain::projects::tests::proposal();
    store.publish(None, &state).unwrap();
}
fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let metadata = std::fs::symlink_metadata(entry.path()).unwrap();
        assert!(
            !crate::infrastructure::os_policy::is_reparse_point(&metadata),
            "Fixture copy rejects reparse entries"
        );
        let target = destination.join(entry.file_name());
        if metadata.is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            assert!(metadata.is_file());
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn open_read_and_stale_capture_do_not_create_origin_metadata() {
    let fixture = Fixture::create();
    let root = fixture.backend("backend");
    let store = ProposalFiles::open(&root).unwrap();
    seed(&store);
    store.load("draft-a").unwrap();
    assert!(!root.join(NAME).exists());
    assert_eq!(
        store.capture_current("draft-a", 2).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert!(!root.join(NAME).exists());
}
#[test]
fn issued_origin_survives_reopen_and_keeps_metadata_guard() {
    let fixture = Fixture::create();
    let root = fixture.backend("backend");
    let store = ProposalFiles::open(&root).unwrap();
    seed(&store);
    let original_state = canonical_sha256(&store.load("draft-a").unwrap()).unwrap();
    let first = store.capture_current("draft-a", 1).unwrap();
    assert_eq!(canonical_sha256(&first.state).unwrap(), original_state);
    let bytes = std::fs::read(root.join(NAME)).unwrap();
    let writer = OpenOptions::new().write(true).open(root.join(NAME));
    assert_eq!(
        writer
            .as_ref()
            .is_err_and(crate::infrastructure::os_policy::is_sharing_violation),
        crate::infrastructure::os_policy::has_mandatory_share_modes()
    );
    drop(writer);
    drop(store);
    let reopened = ProposalFiles::open(&root).unwrap();
    let second = reopened.capture_current("draft-a", 1).unwrap();
    assert_eq!(first.origin, second.origin);
    assert_eq!(std::fs::read(root.join(NAME)).unwrap(), bytes);
}
#[test]
fn same_volume_rename_preserves_origin_but_copied_metadata_is_rejected() {
    let fixture = Fixture::create();
    let source = fixture.backend("original");
    let store = ProposalFiles::open(&source).unwrap();
    seed(&store);
    let origin = store.capture_current("draft-a", 1).unwrap().origin;
    drop(store);
    let renamed = fixture.0.join("renamed");
    std::fs::rename(&source, &renamed).unwrap();
    let store = ProposalFiles::open(&renamed).unwrap();
    assert_eq!(store.capture_current("draft-a", 1).unwrap().origin, origin);
    let copied = fixture.0.join("copied");
    copy_directory(&renamed, &copied);
    let before = std::fs::read(copied.join(NAME)).unwrap();
    let clone = ProposalFiles::open(&copied).unwrap();
    let error = clone.capture_current("draft-a", 1).unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert_eq!(error.remediation, "recover_proposal_store_origin");
    assert_eq!(std::fs::read(copied.join(NAME)).unwrap(), before);
}
#[test]
fn partial_origin_is_preserved_and_never_reissued() {
    let fixture = Fixture::create();
    let root = fixture.backend("backend");
    std::fs::write(root.join(NAME), b"{").unwrap();
    let store = ProposalFiles::open(&root).unwrap();
    seed(&store);
    store.load("draft-a").unwrap();
    let error = store.capture_current("draft-a", 1).unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert_eq!(std::fs::read(root.join(NAME)).unwrap(), b"{");
}
#[test]
fn competing_initializers_use_the_original_writer_and_one_persisted_origin() {
    let fixture = Fixture::create();
    let root = fixture.backend("backend");
    let first = ProposalFiles::open(&root).unwrap();
    seed(&first);
    let second = Arc::new(ProposalFiles::open(&root).unwrap());
    let writer = WriterReservation::acquire(&first).unwrap();
    let competing = second.clone();
    let result = thread::spawn(move || competing.capture_current("draft-a", 1))
        .join()
        .unwrap();
    assert_eq!(result.unwrap_err().code, ErrorCode::RevisionConflict);
    assert!(!root.join(NAME).exists());
    drop(writer);
    let a = first.capture_current("draft-a", 1).unwrap();
    let original = std::fs::read(root.join(NAME)).unwrap();
    let b = second.capture_current("draft-a", 1).unwrap();
    assert_eq!(a.origin, b.origin);
    assert_eq!(std::fs::read(root.join(NAME)).unwrap(), original);
}
#[test]
fn separate_roots_with_identical_proposal_bytes_issue_different_origins() {
    let fixture = Fixture::create();
    let first = ProposalFiles::open(&fixture.backend("first")).unwrap();
    let second = ProposalFiles::open(&fixture.backend("second")).unwrap();
    seed(&first);
    seed(&second);
    let a = first.capture_current("draft-a", 1).unwrap();
    let b = second.capture_current("draft-a", 1).unwrap();
    assert_eq!(
        canonical_sha256(&a.state).unwrap(),
        canonical_sha256(&b.state).unwrap()
    );
    assert_ne!(a.origin, b.origin);
}

#[test]
fn saved_origin_is_reused_without_contacting_the_old_proposal_backend() {
    use crate::application::{
        projects::{
            self, CreateProject, ProjectContentInput, ProposalRevisionRef, SaveProjectRevision,
        },
        proposals::UnavailableProposalStore,
    };
    use crate::domain::api_values::DecimalU64;
    use crate::infrastructure::projects::ProjectFiles;
    let fixture = Fixture::create();
    let root = fixture.backend("backend");
    let proposal = ProposalFiles::open(&root).unwrap();
    seed(&proposal);
    let projects = ProjectFiles::open(&fixture.backend("projects")).unwrap();
    let first = projects::create(
        &projects,
        &proposal,
        CreateProject {
            project_id: "work".into(),
            title: "Work".into(),
            content: ProjectContentInput {
                watching: None,
                proposals: vec![ProposalRevisionRef::Live {
                    proposal_id: "draft-a".into(),
                    draft_revision: DecimalU64(1),
                    working_draft: None,
                }],
            },
        },
    )
    .unwrap();
    let origin = first.project.proposals[0].proposal_origin.clone();
    assert!(origin.is_some());
    assert_eq!(first.project.schema_version, "2");
    let metadata = std::fs::read(root.join(NAME)).unwrap();
    drop(proposal);
    let second = projects::save(
        &projects,
        &UnavailableProposalStore,
        1,
        SaveProjectRevision {
            project_id: "work".into(),
            title: "Saved".into(),
            content: ProjectContentInput {
                watching: None,
                proposals: vec![ProposalRevisionRef::Saved {
                    proposal_id: "draft-a".into(),
                    capture_sha256: first.project.proposals[0].capture_sha256.clone(),
                    working_draft: None,
                }],
            },
        },
    )
    .unwrap();
    assert_eq!(second.project.schema_version, "2");
    assert_eq!(second.project.proposals[0].proposal_origin, origin);
    assert_eq!(
        second.project.proposals[0].capture_sha256,
        first.project.proposals[0].capture_sha256
    );
    assert_eq!(std::fs::read(root.join(NAME)).unwrap(), metadata);
}
