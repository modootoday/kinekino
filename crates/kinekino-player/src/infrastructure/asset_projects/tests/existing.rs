//! Stored project → original guarded PNG inputs → shared renderer, without model/GUI.
use super::*;
use crate::{
    application::{asset_generation, review_source::MutationControl},
    infrastructure::{
        asset_inputs::{FixedAssetImage, FixedAssetInputs, FixedAssetMask},
        asset_raster::RustAssetRaster,
    },
};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
#[test]
#[ignore = "Pinned existing request/PNG pair and fresh evidence; actual immutable project storage/application, no inference or GUI"]
fn saved_project_reopens_original_input_and_renders_without_treating_save_as_review() {
    use sha2::{Digest, Sha256};
    let evidence =
        PathBuf::from(std::env::var_os("ASSET_PROJECT_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&evidence).unwrap();
    let input_path = PathBuf::from(
        std::env::var_os("ASSET_PROJECT_REQUEST").expect("Pinned existing frame request"),
    );
    let request_bytes = fs::read(&input_path).unwrap();
    let hash = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));
    assert_eq!(
        hash(&request_bytes),
        std::env::var("ASSET_PROJECT_REQUEST_SHA256").unwrap()
    );
    let request: AssetFrameRequest = serde_json::from_slice(&request_bytes).unwrap();
    let image_path = PathBuf::from(std::env::var_os("ASSET_INPUT_IMAGE").unwrap());
    let mask_path = PathBuf::from(std::env::var_os("ASSET_INPUT_MASK").unwrap());
    let store_root = evidence.join("projects");
    fs::create_dir(&store_root).unwrap();
    let store = AssetProjectFiles::open(&store_root).unwrap();
    let input = AssetFrameInput {
        image: request.image.clone(),
        mask: request.mask.clone(),
        review: request.review.clone(),
    };
    let mut document = selected(draft("saved-existing"), input);
    document.layers[0].kind = request.layer;
    document.layers[0].transform = request.transform;
    let first = asset_projects::create(&store, &document).unwrap();
    document.revision.0 = 2;
    document.layers[0].transform.pivot.x = 0;
    let second = asset_projects::save(&store, DecimalU64(1), &document).unwrap();
    assert_eq!(
        asset_projects::save(&store, DecimalU64(1), &document)
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    drop(store);
    let reopened = AssetProjectFiles::open(&store_root).unwrap();
    let old = reopened
        .load("saved-existing", Some(DecimalU64(1)))
        .unwrap();
    let current = reopened.load("saved-existing", None).unwrap();
    assert_eq!(old.publication_sha256, first.publication_sha256);
    assert_eq!(
        old.publication
            .document
            .frame_request("frame-001", "foreground")
            .unwrap(),
        request
    );
    assert_eq!(current, second);
    let selected = current
        .publication
        .document
        .frame_request("frame-001", "foreground")
        .unwrap();
    let inputs = FixedAssetInputs::new(
        BTreeMap::from([(
            selected.image.blob.artifact_id.clone(),
            FixedAssetImage {
                pin: selected.image.clone(),
                path: image_path.clone(),
            },
        )]),
        BTreeMap::from([(
            selected.mask.blob.artifact_id.clone(),
            FixedAssetMask {
                pin: selected.mask.clone(),
                path: mask_path.clone(),
            },
        )]),
    )
    .unwrap();
    let admitted = asset_generation::prepare(&selected, &inputs, &Control).unwrap();
    drop(inputs);
    let rendered = admitted.render(&RustAssetRaster, &Control).unwrap();
    assert!(rendered.unreviewed_mask_warning());
    assert!(OpenOptions::new().write(true).open(&image_path).is_err());
    assert!(OpenOptions::new().write(true).open(&mask_path).is_err());
    let pixels = rendered.pixels();
    let output = evidence.join("rendered.png");
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .unwrap();
    let mut encoder = png::Encoder::new(
        file,
        pixels.geometry.output_width,
        pixels.geometry.output_height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&pixels.rgba).unwrap();
    writer.finish().unwrap();
    let bytes = fs::read(&output).unwrap();
    assert_eq!(
        image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .unwrap()
            .into_rgba8()
            .into_raw(),
        pixels.rgba
    );
    fs::write(evidence.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"request_sha256":hash(&request_bytes),"first_publication_sha256":first.publication_sha256,"second_publication_sha256":second.publication_sha256,"old_revision_reopened_unchanged":true,"stale_cas_rejected":true,"stored_input_readmitted_before_render":true,"source_image_sha256":request.image.blob.sha256,"source_mask_sha256":request.mask.blob.sha256,"source_guards_retained_through_render":true,"output_png_sha256":hash(&bytes),"geometry":pixels.geometry,"unreviewed_mask_warning":rendered.unreviewed_mask_warning(),"source_pts":null,"quality_approved":false,"jobs_verified":false,"sdk_cli_verified":false,"gui_visual_verified":false})).unwrap()).unwrap();
    drop(rendered);
    assert_eq!(
        hash(&fs::read(image_path).unwrap()),
        request.image.blob.sha256
    );
    assert_eq!(
        hash(&fs::read(mask_path).unwrap()),
        request.mask.blob.sha256
    );
}
