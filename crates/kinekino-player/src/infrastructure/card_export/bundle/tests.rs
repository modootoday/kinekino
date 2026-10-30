use super::*;
use crate::{
    application::{
        card_bundle::{self, CardBundleRequest, SavedCardProjectPin},
        card_projects::{self, CardProjectStore},
    },
    domain::card_document::CardDocument,
    infrastructure::card_projects::CardProjectFiles,
};
use sha2::{Digest, Sha256};
use std::{io::Cursor, path::PathBuf};
struct Evidence {
    root: PathBuf,
    retain: bool,
}
impl Evidence {
    fn new(case: &str) -> Self {
        let (parent, retain) = match std::env::var_os("CARD_BUNDLE_FILES_EVIDENCE_DIR") {
            Some(path) => {
                let p = PathBuf::from(path);
                assert!(p.is_absolute());
                (p, true)
            }
            None => (std::env::temp_dir(), false),
        };
        let root = parent.join(format!("bundle-{case}-{}", std::process::id()));
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
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn png() -> Vec<u8> {
    let colors = [
        [220, 20, 30, 255],
        [30, 200, 40, 255],
        [30, 40, 210, 255],
        [210, 180, 20, 255],
    ];
    let mut pixels = Vec::new();
    for y in 0..8 {
        for x in 0..8 {
            pixels.extend_from_slice(&colors[(y / 4) * 2 + x / 4]);
        }
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 8, 8);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixels).unwrap();
        writer.finish().unwrap();
    }
    bytes
}
fn fixture(evidence: &Evidence) -> (CardProjectFiles, CardBundleRequest) {
    let frame = png();
    let frame_sha = sha(&frame);
    let path = evidence.root.join("frame.png");
    fs::write(&path, &frame).unwrap();
    let store_root = evidence.root.join("store");
    fs::create_dir(&store_root).unwrap();
    let store = CardProjectFiles::open(&store_root).unwrap();
    let card = serde_json::json!({"card_id":"cover","role":"cover","text_kind":"summary","content":{"front":"첫 카드","back":"검토할 요약 초안","source_refs":["cue-1"]},"frame":{"artifact_id":"frame-1","png_sha256":frame_sha,"source_sha256":"a".repeat(64),"declared_source_pts_us":null},"alt_text_draft":"설명 초안"});
    let mut closing = card.clone();
    closing["card_id"] = serde_json::json!("closing");
    closing["role"] = serde_json::json!("closing");
    closing["content"]["front"] = serde_json::json!("마지막 카드");
    let document: CardDocument = serde_json::from_value(serde_json::json!({"schema_version":"1","document_id":"bundle-1","revision":"1","source":{"source_id":"cue-source","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"원문 대사","start_us":null,"end_us":null}]},"layout":{"canvas":CardBundleFiles::current_proposed_canvas(),"cards":[card,closing],"caption_draft":"게시문 초안"}})).unwrap();
    let saved = card_projects::create(&store, &document).unwrap();
    let request = CardBundleRequest {
        project: SavedCardProjectPin {
            document_id: document.document_id,
            publication_revision: DecimalU64(1),
            publication_sha256: saved.publication_sha256,
        },
        frames: vec![SelectedCardFrame {
            artifact_id: "frame-1".into(),
            path: path.to_str().unwrap().into(),
            png_sha256: frame_sha,
        }],
        output_directory: evidence.root.join("new-bundle").to_str().unwrap().into(),
        review_artifact_only: true,
    };
    (store, request)
}

#[test]
fn real_project_frame_fonts_and_template_prepare_and_render_without_fake_acceptance() {
    let evidence = Evidence::new("render");
    let (store, request) = fixture(&evidence);
    let before = serde_json::to_vec(&store.load("bundle-1", None).unwrap()).unwrap();
    let output_directory = request.output_directory.clone();
    let prepared = card_bundle::prepare(&store, &CardBundleFiles, request).unwrap();
    fs::write(
        evidence.root.join("prepared.json"),
        serde_json::to_vec_pretty(&prepared).unwrap(),
    )
    .unwrap();
    assert_eq!(prepared.template.common_font_sha256.len(), 3);
    for position in 0..2 {
        let result = CardBundleFiles
            .render_card(&prepared, DecimalU64(position))
            .unwrap();
        let mut reader = png::Decoder::new(Cursor::new(&result.png))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            (info.width, info.height, info.color_type),
            (1080, 1080, png::ColorType::Rgba)
        );
        let pixel = |x: usize, y: usize| &pixels[(y * 1080 + x) * 4..(y * 1080 + x) * 4 + 4];
        assert_eq!(pixel(400, 200), [220, 20, 30, 255]);
        assert_eq!(pixel(650, 450), [210, 180, 20, 255]);
        assert_eq!(
            result.provenance["card_project"]["publication_sha256"],
            prepared.project.publication_sha256
        );
        assert_eq!(result.provenance["quality_approved"], false);
        assert!(result.provenance.get("accepted_output_revision").is_none());
        assert!(result.provenance.get("target").is_none());
        fs::write(
            evidence.root.join(format!("observation-{position}.png")),
            &result.png,
        )
        .unwrap();
        fs::write(
            evidence.root.join(format!("provenance-{position}.json")),
            serde_json::to_vec_pretty(&result.provenance).unwrap(),
        )
        .unwrap();
    }
    assert!(
        !Path::new(&output_directory).exists(),
        "Buffer rendering is not bundle publication"
    );
    assert_eq!(
        serde_json::to_vec(&store.load("bundle-1", None).unwrap()).unwrap(),
        before
    );
}

#[test]
fn real_png_hash_crc_and_trailing_errors_are_not_admitted() {
    let evidence = Evidence::new("png-errors");
    let (_, request) = fixture(&evidence);
    let selected = &request.frames[0];
    let mut wrong = selected.clone();
    wrong.png_sha256 = "f".repeat(64);
    assert_eq!(
        CardBundleFiles.inspect_frame(&wrong).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let mut trailing = png();
    trailing.extend_from_slice(b"trailing");
    let path = evidence.root.join("trailing.png");
    fs::write(&path, &trailing).unwrap();
    wrong.path = path.to_str().unwrap().into();
    wrong.png_sha256 = sha(&trailing);
    assert_eq!(
        CardBundleFiles.inspect_frame(&wrong).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut crc = png();
    crc[29] ^= 1;
    let path = evidence.root.join("crc.png");
    fs::write(&path, &crc).unwrap();
    wrong.path = path.to_str().unwrap().into();
    wrong.png_sha256 = sha(&crc);
    assert_eq!(
        CardBundleFiles.inspect_frame(&wrong).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}

#[test]
#[ignore = "Central allocation: actual CPU raster for three explicit canvas layouts"]
fn explicit_canvas_layouts_render_at_native_size_and_preserve_legacy_pin() {
    let evidence = Evidence::new("canvases");
    let (store, mut request) = fixture(&evidence);
    let original = store.load("bundle-1", None).unwrap();
    let canvases = CardBundleFiles::proposed_canvases();
    assert_eq!(canvases.len(), 13);
    assert_eq!(
        serde_json::to_value(&canvases[0]).unwrap(),
        serde_json::to_value(CardBundleFiles::current_proposed_canvas()).unwrap()
    );
    for (index, canvas) in canvases.into_iter().skip(1).take(3).enumerate() {
        let mut document = original.publication.document.clone();
        document.revision = DecimalU64(index as u64 + 2);
        document.layout.canvas = canvas.clone();
        let saved = store
            .publish(Some(DecimalU64(index as u64 + 1)), &document)
            .unwrap();
        request.project.publication_revision = saved.publication.publication_revision;
        request.project.publication_sha256 = saved.publication_sha256;
        let prepared = card_bundle::prepare(&store, &CardBundleFiles, request.clone()).unwrap();
        let output = CardBundleFiles
            .render_card(&prepared, DecimalU64(0))
            .unwrap();
        let (width, height) =
            crate::infrastructure::snapshot_export::inspect_static_png(&output.png).unwrap();
        assert_eq!((width, height), (canvas.width, canvas.height));
        assert_eq!(output.provenance["template"]["id"], canvas.template_id);
        assert_eq!(
            output.provenance["template_status"],
            "proposed_not_user_approved"
        );
        fs::write(
            evidence.root.join(format!("canvas-{index}.png")),
            output.png,
        )
        .unwrap();
        fs::write(
            evidence.root.join(format!("canvas-{index}.json")),
            serde_json::to_vec_pretty(&output.provenance).unwrap(),
        )
        .unwrap();
        let mut stale = canvas;
        stale.template_sha256 = "0".repeat(64);
        assert_eq!(
            CardBundleFiles.verify_template(&stale).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
    }
    assert_eq!(
        store
            .load("bundle-1", Some(DecimalU64(1)))
            .unwrap()
            .publication_sha256,
        original.publication_sha256
    );
    assert!(!Path::new(&request.output_directory).exists());
}

#[test]
fn template_canvas_and_output_collisions_are_rejected_without_creation() {
    let evidence = Evidence::new("template-output");
    let (_, request) = fixture(&evidence);
    let mut canvas = CardBundleFiles::current_proposed_canvas();
    canvas.template_sha256 = "e".repeat(64);
    assert_eq!(
        CardBundleFiles.verify_template(&canvas).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    canvas = CardBundleFiles::current_proposed_canvas();
    canvas.width = 720;
    assert_eq!(
        CardBundleFiles.verify_template(&canvas).unwrap_err().code,
        ErrorCode::Unsupported
    );
    CardBundleFiles
        .verify_new_output_directory(&request.output_directory)
        .unwrap();
    fs::create_dir(&request.output_directory).unwrap();
    assert_eq!(
        CardBundleFiles
            .verify_new_output_directory(&request.output_directory)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        CardBundleFiles
            .verify_new_output_directory("relative")
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let traversal = evidence.root.join("../foreign");
    assert_eq!(
        CardBundleFiles
            .verify_new_output_directory(traversal.to_str().unwrap())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn prepared_metadata_and_changed_frame_fail_before_product_publication() {
    let evidence = Evidence::new("changed");
    let (store, request) = fixture(&evidence);
    let prepared = card_bundle::prepare(&store, &CardBundleFiles, request).unwrap();
    let mut changed = prepared.clone();
    changed.cards[0].output_name = "../escape.png".into();
    assert_eq!(
        CardBundleFiles
            .render_card(&changed, DecimalU64(0))
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    let mut changed = prepared.clone();
    changed.template.common_font_sha256[0] = "e".repeat(64);
    assert_eq!(
        CardBundleFiles
            .render_card(&changed, DecimalU64(0))
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    fs::copy(
        &prepared.cards[0].frame.as_ref().unwrap().path,
        evidence.root.join("frame-before-change.png"),
    )
    .unwrap();
    fs::write(
        &prepared.cards[0].frame.as_ref().unwrap().path,
        b"changed after admission",
    )
    .unwrap();
    assert_eq!(
        CardBundleFiles
            .render_card(&prepared, DecimalU64(0))
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    assert!(!Path::new(&prepared.output_directory).exists());
}
