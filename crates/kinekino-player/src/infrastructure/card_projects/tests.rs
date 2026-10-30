use super::*;
use crate::{
    application::card_projects::{create, inspect, save},
    domain::card_document::{CardEdit, CardEditor},
};
use std::sync::{Arc, Barrier};

struct Evidence {
    root: PathBuf,
    retain: bool,
}
impl Evidence {
    fn new(case: &str) -> Self {
        let (parent, retain) = match std::env::var_os("CARD_PROJECT_STORAGE_EVIDENCE_DIR") {
            Some(path) => {
                let path = PathBuf::from(path);
                assert!(path.is_absolute());
                (path, true)
            }
            None => (std::env::temp_dir(), false),
        };
        let root = parent.join(format!("card-project-{case}-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self { root, retain }
    }
}
impl Drop for Evidence {
    fn drop(&mut self) {
        if !self.retain {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn document() -> CardDocument {
    serde_json::from_value(serde_json::json!({
        "schema_version":"1","document_id":"cards-1","revision":"1",
        "source":{"source_id":"original","source_revision":"2","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"원문 대사","start_us":null,"end_us":null}]},
        "layout":{"canvas":{"width":1080,"height":1080,"template_id":"card.frame-two-slot","template_version":"0.1.0-proposal.1","template_sha256":"b".repeat(64)},
            "cards":[{"card_id":"first","role":"body","text_kind":"summary","content":{"front":"제목","back":"요약 초안","source_refs":["cue-1"]},"frame":null,"alt_text_draft":"설명 초안"}],"caption_draft":"게시문"}
    })).unwrap()
}
fn edited(caption: &str) -> CardDocument {
    let mut editor = CardEditor::open(document()).unwrap();
    editor
        .apply(
            DecimalU64(1),
            CardEdit::SetCaption {
                caption_draft: caption.into(),
            },
        )
        .unwrap();
    editor.document().clone()
}
fn record_path(root: &Path, publication: &StoredCardProject) -> PathBuf {
    root.join("card-cards-1")
        .join(format!(
            "r-{}",
            publication.publication.publication_revision.0
        ))
        .join(format!("{}.json", publication.publication_sha256))
}

#[test]
fn actual_files_reopen_history_and_separate_edit_from_publication_revision() {
    let evidence = Evidence::new("history");
    let store = CardProjectFiles::open(&evidence.root).unwrap();
    let original = document();
    let first = create(&store, &original).unwrap();
    let first_bytes = fs::read(record_path(&evidence.root, &first)).unwrap();
    let mut editor = CardEditor::open(original).unwrap();
    for revision in 1..=3 {
        editor
            .apply(
                DecimalU64(revision),
                CardEdit::SetCaption {
                    caption_draft: revision.to_string(),
                },
            )
            .unwrap();
    }
    let second = save(&store, DecimalU64(1), editor.document()).unwrap();
    assert_eq!(second.publication.publication_revision, DecimalU64(2));
    assert_eq!(second.publication.document.revision, DecimalU64(4));
    assert_eq!(
        second.publication.parent_publication_sha256.as_deref(),
        Some(first.publication_sha256.as_str())
    );
    drop(store);
    let reopened = CardProjectFiles::open(&evidence.root).unwrap();
    let head = inspect(&reopened, "cards-1", None).unwrap();
    assert_eq!(head.publication_sha256, second.publication_sha256);
    let old = inspect(&reopened, "cards-1", Some(DecimalU64(1))).unwrap();
    assert_eq!(old.latest_publication_revision, DecimalU64(2));
    assert_eq!(old.publication_sha256, first.publication_sha256);
    assert_eq!(
        fs::read(record_path(&evidence.root, &first)).unwrap(),
        first_bytes
    );
}

#[test]
fn actual_competing_writers_have_one_cas_winner() {
    let evidence = Evidence::new("writers");
    create(
        &CardProjectFiles::open(&evidence.root).unwrap(),
        &document(),
    )
    .unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let mut writers = Vec::new();
    for caption in ["writer-one", "writer-two"] {
        let root = evidence.root.clone();
        let barrier = barrier.clone();
        writers.push(std::thread::spawn(move || {
            let store = CardProjectFiles::open(&root).unwrap();
            barrier.wait();
            save(&store, DecimalU64(1), &edited(caption))
        }));
    }
    let results: Vec<_> = writers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| r
                .as_ref()
                .is_err_and(|e| e.code == ErrorCode::RevisionConflict))
            .count(),
        1
    );
    assert_eq!(
        inspect(
            &CardProjectFiles::open(&evidence.root).unwrap(),
            "cards-1",
            None
        )
        .unwrap()
        .latest_publication_revision,
        DecimalU64(2)
    );
}

#[test]
fn source_replacement_stale_and_unchanged_edit_are_rejected_without_publication() {
    let evidence = Evidence::new("rejections");
    let store = CardProjectFiles::open(&evidence.root).unwrap();
    let original = create(&store, &document()).unwrap();
    let before = fs::read(record_path(&evidence.root, &original)).unwrap();
    assert_eq!(
        create(&store, &document()).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        save(&store, DecimalU64(1), &document()).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        save(&store, DecimalU64(2), &edited("stale"))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let mut changed = edited("source replacement");
    changed.source.cues[0].text = "다른 원문".into();
    assert_eq!(
        save(&store, DecimalU64(1), &changed).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        fs::read_dir(evidence.root.join("card-cards-1"))
            .unwrap()
            .count(),
        1
    );
    assert_eq!(
        fs::read(record_path(&evidence.root, &original)).unwrap(),
        before
    );
}

#[test]
fn corrupt_head_blocks_historical_read_and_never_falls_back_or_repairs() {
    let evidence = Evidence::new("corrupt");
    let store = CardProjectFiles::open(&evidence.root).unwrap();
    create(&store, &document()).unwrap();
    let head = save(&store, DecimalU64(1), &edited("head")).unwrap();
    let path = record_path(&evidence.root, &head);
    fs::write(
        evidence.root.join("head-before-corruption.json"),
        fs::read(&path).unwrap(),
    )
    .unwrap();
    fs::write(&path, b"{truncated").unwrap();
    drop(store);
    let store = CardProjectFiles::open(&evidence.root).unwrap();
    for requested in [None, Some(DecimalU64(1))] {
        let error = inspect(&store, "cards-1", requested).unwrap_err();
        assert_eq!(error.code, ErrorCode::NeedsInput);
        assert_eq!(error.remediation, "recover_card_project_publication");
    }
    assert_eq!(fs::read(path).unwrap(), b"{truncated");
}

#[test]
fn partial_publication_is_preserved_and_blocks_writes() {
    let evidence = Evidence::new("partial");
    let store = CardProjectFiles::open(&evidence.root).unwrap();
    create(&store, &document()).unwrap();
    let incomplete = evidence.root.join("card-cards-1/r-2");
    fs::create_dir(&incomplete).unwrap();
    assert_eq!(
        inspect(&store, "cards-1", None).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        save(&store, DecimalU64(1), &edited("retry"))
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert!(incomplete.is_dir());
    assert_eq!(fs::read_dir(incomplete).unwrap().count(), 0);
}

#[test]
fn writer_lock_and_path_boundaries_do_not_create_a_project() {
    let evidence = Evidence::new("boundaries");
    let store = CardProjectFiles::open(&evidence.root).unwrap();
    assert_eq!(
        inspect(&store, "../foreign", None).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert!(CardProjectFiles::open(Path::new("relative")).is_err());
    let lock = paths::lock(&evidence.root.join("card-writer.lock")).unwrap();
    assert_eq!(
        create(&store, &document()).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert!(!evidence.root.join("card-cards-1").exists());
    drop(lock);
    create(&store, &document()).unwrap();
}
