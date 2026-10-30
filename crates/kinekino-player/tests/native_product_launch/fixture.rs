use kinekino_player::application::api::ProposalClient;
use kinekino_player::{
    application::{
        api::{ApplicationClientHandle, PlayerSdk, RequestContext},
        launch::native_proposal::*,
        proposals::{CreateProposal, InspectProposal},
    },
    domain::{api_values::DecimalU64, proposals::*},
    infrastructure::{content_digest, proposals::ProposalFiles},
};
use meari::domain::build::ContentHash;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy)]
pub enum Kind {
    Cards,
    Highlights,
    CueCorrection,
    DocumentCorrection,
    RepairUnselected,
    RepairSelected,
}
pub fn evidence_root(label: &str) -> PathBuf {
    let parent = std::env::var_os("NATIVE_PRODUCT_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
                .join("runtime/quality-review/product-native-main-r1/actual")
        });
    assert!(parent.is_absolute());
    fs::create_dir_all(&parent).unwrap();
    let root = parent.join(format!(
        "{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    root
}
pub fn binary_sha(path: &Path) -> String {
    content_digest::sha256(&mut fs::File::open(path).unwrap(), 2 * 1024 * 1024 * 1024)
        .unwrap()
        .1
}
pub fn pin_binary(root: &Path, source: &Path) -> PathBuf {
    let binary = root.join(kinekino_player::domain::product::PRODUCT.gui_executable);
    let before = binary_sha(source);
    fs::copy(source, &binary).unwrap();
    assert_eq!(binary_sha(source), before);
    assert_eq!(binary_sha(&binary), before);
    fs::write(root.join("binary-sha256.txt"), before).unwrap();
    binary
}

pub struct Fixture {
    pub name: &'static str,
    pub root: PathBuf,
    pub store: PathBuf,
    pub request_path: PathBuf,
    client: ApplicationClientHandle,
    original: serde_json::Value,
}
impl Fixture {
    pub fn client(&self) -> ApplicationClientHandle {
        self.client.clone()
    }
    pub fn new(parent: &Path, kind: Kind) -> Self {
        let name = match kind {
            Kind::Cards => "cards",
            Kind::Highlights => "highlights",
            Kind::CueCorrection => "cue-correction",
            Kind::DocumentCorrection => "document-correction",
            Kind::RepairSelected => "repair-selected",
            Kind::RepairUnselected => "repair-unselected",
        };
        let root = parent.join(name);
        fs::create_dir(&root).unwrap();
        let store = root.join("store");
        fs::create_dir(&store).unwrap();
        let client: ApplicationClientHandle = Arc::new(
            PlayerSdk::default().with_proposal_store(ProposalFiles::open(&store).unwrap()),
        );
        let (source, payload, target) = match kind {
            Kind::Cards | Kind::Highlights | Kind::CueCorrection => {
                let source = SourceSnapshot::Cue(CueSourceSnapshot {
                    source_id: "source-a".into(),
                    source_revision: DecimalU64(7),
                    source_sha256: "a".repeat(64),
                    cues: ["a", "b"]
                        .into_iter()
                        .map(|id| SourceCue {
                            cue_id: id.into(),
                            text: format!("보존 원문 {id}"),
                            start_us: None,
                            end_us: None,
                        })
                        .collect(),
                });
                let (payload, target) = match kind {
                    Kind::Cards => (
                        ProposalPayload::Cards {
                            cards: ["first", "second"]
                                .into_iter()
                                .map(|text| Card {
                                    front: text.into(),
                                    back: format!("{text} back"),
                                    source_refs: vec!["b".into()],
                                })
                                .collect(),
                        },
                        NativeProposalTarget::Card { index: 1 },
                    ),
                    Kind::Highlights => (
                        ProposalPayload::Highlights {
                            highlights: ["first", "second"]
                                .into_iter()
                                .map(|text| Highlight {
                                    title: text.into(),
                                    source_refs: vec!["b".into()],
                                })
                                .collect(),
                        },
                        NativeProposalTarget::Highlight { index: 1 },
                    ),
                    _ => (
                        ProposalPayload::Correction {
                            edits: ["a", "b"]
                                .into_iter()
                                .map(|id| Correction {
                                    cue_id: id.into(),
                                    text: format!("수정 {id}"),
                                })
                                .collect(),
                        },
                        NativeProposalTarget::CueCorrection {
                            index: 1,
                            cue_id: "b".into(),
                        },
                    ),
                };
                (source, Some(payload), target)
            }
            _ => {
                let prefix = "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n";
                let asset=serde_json::json!({"messages":[{"role":"user","content":format!("{prefix}[\"첫 원문\",\"明示した原文\"]")}]}).to_string();
                let source = DocumentSourceSnapshot::from_asset(
                    "document-source".into(),
                    DecimalU64(7),
                    ContentHash::of(asset.as_bytes()).as_str().into(),
                    asset,
                )
                .unwrap();
                let reference = source.text_reference("/1").unwrap();
                let payload = matches!(kind, Kind::DocumentCorrection).then(|| {
                    ProposalPayload::DocumentCorrection {
                        edits: ["/0", "/1"]
                            .into_iter()
                            .map(|pointer| DocumentCorrection {
                                source_ref: source.text_reference(pointer).unwrap(),
                                text: format!("수정 {pointer}"),
                            })
                            .collect(),
                    }
                });
                let target = match kind {
                    Kind::DocumentCorrection => NativeProposalTarget::DocumentCorrection {
                        index: 1,
                        source_ref: reference,
                    },
                    Kind::RepairSelected => NativeProposalTarget::DocumentRepair {
                        selection: DocumentRepairSelection::Selected {
                            source_ref: reference,
                        },
                    },
                    _ => NativeProposalTarget::DocumentRepair {
                        selection: DocumentRepairSelection::Unselected {},
                    },
                };
                (SourceSnapshot::Document(source), payload, target)
            }
        };
        let generated = GeneratedResult {
            task: payload
                .as_ref()
                .map_or(ProposalTask::Correction, ProposalPayload::task),
            raw_text: payload
                .as_ref()
                .map(|value| serde_json::to_string(value).unwrap())
                .unwrap_or_else(|| "{preserved invalid generated result".into()),
        };
        let document = client
            .create_proposal(
                RequestContext::default(),
                CreateProposal {
                    proposal_id: "explicit-proposal".into(),
                    source,
                    generated,
                },
            )
            .unwrap();
        let request = NativeProposalLaunchRequest {
            schema_version: NATIVE_PROPOSAL_SCHEMA_VERSION.into(),
            context: RequestContext {
                request_id: format!("actual-{name}"),
                ..Default::default()
            },
            selection: NativeProposalSelection {
                proposal_id: document.proposal_id,
                document_revision: document.draft_revision,
                source_id: document.source.source_id().into(),
                source_revision: document.source.source_revision(),
                source_sha256: document.source.source_sha256().into(),
                target,
            },
        };
        let request_path = root.join("request.json");
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        let original = serde_json::to_value(
            client
                .inspect_proposal(
                    RequestContext::default(),
                    InspectProposal {
                        proposal_id: "explicit-proposal".into(),
                    },
                )
                .unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("original.json"),
            serde_json::to_vec(&original).unwrap(),
        )
        .unwrap();
        Self {
            name,
            root,
            store,
            request_path,
            client,
            original,
        }
    }
    pub fn assert_unchanged(&self) {
        let after = serde_json::to_value(
            self.client
                .inspect_proposal(
                    RequestContext::default(),
                    InspectProposal {
                        proposal_id: "explicit-proposal".into(),
                    },
                )
                .unwrap(),
        )
        .unwrap();
        fs::write(
            self.root.join("after.json"),
            serde_json::to_vec(&after).unwrap(),
        )
        .unwrap();
        assert_eq!(after, self.original);
    }
}
