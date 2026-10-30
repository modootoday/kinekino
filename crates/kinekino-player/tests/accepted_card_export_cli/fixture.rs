use kinekino_player::{
    application::proposals::{
        self, AcceptProposal, CreateProposal, InspectProposal,
        card_export::{ExportCardReviewPng, TemplateSelection},
    },
    assembly::card_export::product_template,
    domain::{
        api_values::DecimalU64,
        proposals::{
            CueSourceSnapshot, GeneratedResult, ProposalTask, SourceCue, SourceSnapshot,
            output_review::OutputReviewTarget,
        },
    },
    infrastructure::proposals::ProposalFiles,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

pub(super) struct Fixture {
    pub store_path: PathBuf,
    pub frame: PathBuf,
    pub target: OutputReviewTarget,
    pub overflow: OutputReviewTarget,
    store: ProposalFiles,
}
impl Fixture {
    pub fn create(root: &Path) -> Self {
        let store_path = root.join("store");
        fs::create_dir(&store_path).unwrap();
        let store = ProposalFiles::open(&store_path).unwrap();
        let target = accepted(&store, "card-plain", "앞면 <原文>");
        let overflow = accepted(&store, "card-overflow", &"한".repeat(10000));
        let frame = root.join("frame.png");
        // Both bilinear neighbours at each unchanged sample stay in one solid block.
        let colors = [
            [220, 20, 30, 255],
            [30, 200, 40, 255],
            [30, 40, 210, 255],
            [210, 180, 20, 255],
        ];
        let mut source_pixels = Vec::with_capacity(8 * 8 * 4);
        for y in 0..8 {
            for x in 0..8 {
                source_pixels.extend_from_slice(&colors[(y / 4) * 2 + x / 4]);
            }
        }
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 8, 8);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&source_pixels).unwrap();
            writer.finish().unwrap();
        }
        fs::write(&frame, bytes).unwrap();
        Self {
            store_path,
            frame,
            target,
            overflow,
            store,
        }
    }
    pub fn input(&self, output: &Path) -> ExportCardReviewPng {
        ExportCardReviewPng {
            target: self.target.clone(),
            position: DecimalU64(0),
            frame_path: self.frame.to_str().unwrap().into(),
            frame_sha256: super::hash(&fs::read(&self.frame).unwrap()),
            template: TemplateSelection {
                id: product_template::TEMPLATE_ID.into(),
                version: product_template::PROPOSAL_VERSION.into(),
                declaration_sha256: super::hash(include_bytes!(
                    "../../src/assembly/card_export/product_template.rs"
                )),
            },
            output_path: output.to_str().unwrap().into(),
            review_artifact_only: true,
        }
    }
    pub fn logical_snapshot(&self) -> Vec<u8> {
        let states: Vec<_> = [&self.target, &self.overflow]
            .into_iter()
            .map(|target| {
                proposals::inspect(
                    &self.store,
                    InspectProposal {
                        proposal_id: target.proposal_id.clone(),
                    },
                )
                .unwrap()
            })
            .collect();
        serde_json::to_vec(&states).unwrap()
    }
}
fn accepted(store: &ProposalFiles, id: &str, front: &str) -> OutputReviewTarget {
    proposals::create(store, CreateProposal {
        proposal_id:id.into(),
        source: SourceSnapshot::Cue(CueSourceSnapshot {source_id:format!("{id}-source"),source_revision:DecimalU64(1),source_sha256:"a".repeat(64),cues:vec![SourceCue {cue_id:"cue-1".into(),text:"Synthetic source".into(),start_us:None,end_us:None}]}),
        generated: GeneratedResult {task:ProposalTask::Cards,raw_text:json!({"task":"cards","cards":[{"front":front,"back":"かな 뒷면 & 보존","source_refs":["cue-1"]}]}).to_string()},
    }).unwrap();
    proposals::accept(
        store,
        AcceptProposal {
            proposal_id: id.into(),
            target_source_revision: DecimalU64(1),
        },
        1,
    )
    .unwrap();
    let state = proposals::inspect(
        store,
        InspectProposal {
            proposal_id: id.into(),
        },
    )
    .unwrap();
    OutputReviewTarget::from_stored(&state).unwrap()
}
pub(super) fn store_snapshot(root: &Path) -> BTreeMap<String, Value> {
    fn visit(base: &Path, dir: &Path, files: &mut BTreeMap<String, Value>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let meta = fs::symlink_metadata(&path).unwrap();
            assert!(!meta.file_type().is_symlink());
            if meta.is_dir() {
                visit(base, &path, files)
            } else {
                assert!(meta.is_file());
                files.insert(path.strip_prefix(base).unwrap().to_str().unwrap().into(),json!({"sha256":super::hash(&fs::read(&path).unwrap()),"bytes":meta.len(),"modified_ns":meta.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string()}));
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}
