//! Test custody and synthetic documents; listing policy stays in the SDK.
use kinekino_player::{
    application::{
        api::{ApiError, PlayerSdk, Request, RequestContext},
        proposals::{self, CreateProposal, ProposalStore},
    },
    domain::{api_values::DecimalU64, proposals::*},
    infrastructure::{content_digest, proposals::ProposalFiles, subtitle_files::SubtitleFiles},
};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct StoreFixture {
    pub root: PathBuf,
}
impl StoreFixture {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "proposal-list-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Self::at(root)
    }
    pub fn at(root: PathBuf) -> Self {
        fs::create_dir(&root).unwrap();
        eprintln!("proposal_list_fixture_root={}", root.display());
        Self { root }
    }
    pub fn store(&self) -> ProposalFiles {
        ProposalFiles::open(&self.root).unwrap()
    }
    pub fn client(&self) -> PlayerSdk<SubtitleFiles> {
        PlayerSdk::new(SubtitleFiles).with_proposal_store(self.store())
    }
    pub fn seed(&self, id: &str, task: ProposalTask, repair: bool) {
        let payload = match task {
            ProposalTask::Correction => ProposalPayload::Correction {
                edits: vec![Correction {
                    cue_id: "cue-1".into(),
                    text: "Corrected".into(),
                }],
            },
            ProposalTask::Cards => ProposalPayload::Cards {
                cards: vec![Card {
                    front: "Question".into(),
                    back: "Answer".into(),
                    source_refs: vec!["cue-1".into()],
                }],
            },
            ProposalTask::Highlights => ProposalPayload::Highlights {
                highlights: vec![Highlight {
                    title: "Selected".into(),
                    source_refs: vec!["cue-1".into()],
                }],
            },
        };
        proposals::create(
            &self.store(),
            CreateProposal {
                proposal_id: id.into(),
                source: SourceSnapshot::Cue(CueSourceSnapshot {
                    source_id: format!("source-{id}"),
                    source_revision: DecimalU64(1),
                    source_sha256: "a".repeat(64),
                    cues: vec![SourceCue {
                        cue_id: "cue-1".into(),
                        text: "Original immutable source".into(),
                        start_us: None,
                        end_us: None,
                    }],
                }),
                generated: GeneratedResult {
                    task,
                    raw_text: if repair {
                        "invalid generated output".into()
                    } else {
                        serde_json::to_string(&payload).unwrap()
                    },
                },
            },
        )
        .unwrap();
    }
    pub fn edit(&self, id: &str) {
        let state = self.store().load(id).unwrap();
        let mut draft = state.document.draft.unwrap();
        match &mut draft {
            ProposalPayload::Cards { cards } => cards[0].front = "Changed".into(),
            ProposalPayload::Highlights { highlights } => highlights[0].title = "Changed".into(),
            ProposalPayload::Correction { edits } => edits[0].text = "Changed".into(),
            ProposalPayload::DocumentCorrection { edits } => edits[0].text = "Changed".into(),
        }
        proposals::edit(
            &self.store(),
            proposals::EditProposal {
                proposal_id: id.into(),
                draft,
            },
            state.document.draft_revision.0,
        )
        .unwrap();
    }
    pub fn request(&self, payload: Value) -> kinekino_player::application::api::Response {
        let mut request = Request::new("proposals.list");
        request.payload = payload;
        self.client().execute(request)
    }
    pub fn page(
        &self,
        payload: Value,
    ) -> kinekino_player::application::proposals::list::ProposalListPage {
        let response = self.request(payload);
        assert!(response.ok, "{:?}", response.error);
        serde_json::from_value(response.data.unwrap()).unwrap()
    }
    pub fn origin_path(&self) -> PathBuf {
        self.root.join(".proposal-store-origin.v1.json")
    }
}

pub struct CliFixture {
    pub evidence: PathBuf,
    pub store: StoreFixture,
    executable: PathBuf,
    _guards: Vec<File>,
    sequence: AtomicU64,
}
impl CliFixture {
    pub fn new(label: &str) -> Self {
        let evidence = PathBuf::from(
            std::env::var_os("PROPOSAL_LIST_EVIDENCE_ROOT")
                .expect("Allocated evidence root required"),
        )
        .join(label);
        assert!(evidence.is_absolute());
        fs::create_dir(&evidence).unwrap();
        let store = StoreFixture::at(evidence.join("store"));
        let executable = PathBuf::from(
            std::env::var_os("PROPOSAL_LIST_CLI_BIN").expect("Pinned actual CLI required"),
        );
        assert!(executable.is_absolute());
        let expected = std::env::var("PROPOSAL_LIST_CLI_SHA256")
            .unwrap()
            .to_ascii_lowercase();
        assert!(kinekino_player::domain::identifiers::valid_sha256(
            &expected
        ));
        let bytes: u64 = std::env::var("PROPOSAL_LIST_CLI_BYTES")
            .unwrap()
            .parse()
            .unwrap();
        let (_, mut guards) =
            crate::immutable_storage_paths::root(executable.parent().unwrap()).unwrap();
        let mut file = crate::immutable_storage_paths::file(&executable).unwrap();
        assert_eq!(
            content_digest::sha256(&mut file, bytes).unwrap(),
            (bytes, expected.clone())
        );
        guards.push(file);
        fs::write(
            evidence.join("binary.json"),
            serde_json::to_vec_pretty(
                &json!({"executable":executable,"sha256":expected,"bytes":bytes}),
            )
            .unwrap(),
        )
        .unwrap();
        Self {
            evidence,
            store,
            executable,
            _guards: guards,
            sequence: AtomicU64::new(0),
        }
    }
    pub fn call(&self, command: &str, payload: Value) -> Value {
        self.call_at(command, payload, &self.store.root)
    }
    pub fn call_at(&self, command: &str, payload: Value, root: &Path) -> Value {
        let n = self.sequence.fetch_add(1, Ordering::AcqRel);
        let evidence = self.evidence.join(format!("rpc-{n}"));
        fs::create_dir(&evidence).unwrap();
        let mut request = Request::new(command);
        request.context = RequestContext {
            request_id: format!("proposal-list-{n}"),
            ..RequestContext::default()
        };
        request.payload = payload;
        let raw = serde_json::to_vec(&request).unwrap();
        let outcome = crate::owned_process_evidence::run_process_with_arguments(
            self.executable.clone(),
            &evidence,
            &raw,
            vec![
                "--input".into(),
                "-".into(),
                "--proposal-root".into(),
                root.as_os_str().to_owned(),
            ],
            Duration::from_secs(15),
            false,
        );
        let output = match outcome {
            Ok(output) => output,
            Err(failure) => {
                let retained = failure.closeout();
                let observation = retained.observation();
                Box::leak(retained);
                panic!("Original CLI owner retained: {observation}");
            }
        };
        assert_eq!(
            output.receipt.family,
            kinekino_process::domain::observation::FamilyObservation::Empty
        );
        assert!(!output.receipt.forced);
        assert!(output.receipt.stdin_eof_requested);
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["request_id"], request.context.request_id);
        let expected = if response["ok"] == true {
            0
        } else {
            serde_json::from_value::<ApiError>(response["error"].clone())
                .unwrap()
                .exit_code()
        };
        assert_eq!(output.receipt.child_exit, Some(u32::from(expected)));
        response
    }
}
