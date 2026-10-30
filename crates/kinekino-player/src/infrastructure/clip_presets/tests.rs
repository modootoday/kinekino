use super::*;
use crate::application::clip_render::presets::tests::document;
use serde_json::{Value, json};
use std::{fs, path::Path};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn write(path: &Path, value: &Value) -> ArtifactPin {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    fs::write(path, &bytes).unwrap();
    ArtifactPin {
        artifact_id: "local-test".into(),
        revision: DecimalU64(1),
        sha256: hash(&bytes),
    }
}
fn files(path: &Path) -> FixedClipPresets {
    FixedClipPresets::open(BTreeMap::from([("local-test".into(), path.into())])).unwrap()
}
#[test]
#[ignore = "Fresh evidence; actual Windows file/ancestor guards and raw preset bytes, no encoder/media/GUI/models"]
fn raw_host_manual_definition_is_pinned_guarded_and_settings_and_limits_are_enforced() {
    let root = PathBuf::from(
        std::env::var_os("CLIP_PRESET_EVIDENCE_DIR").expect("Fresh absolute evidence"),
    );
    assert!(root.is_absolute());
    fs::create_dir(&root).unwrap();
    let value = json!({"schema_version":"clip-render-preset/1","preset_id":"local-test","revision":"1","label":"로컬 수동 640×360","origin":"host_manual","profile":{"width":640,"height":360,"frame_rate":{"numerator":24,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":null,"color":"sdr_bt709"},"max_duration_us":"2000000"});
    let path = root.join("preset.json");
    let pin = write(&path, &value);
    let port = files(&path);
    let catalog = port.inspect_presets().unwrap();
    assert_eq!(catalog.presets.len(), 3);
    assert!(!catalog.engine_availability_verified);
    let entry = catalog
        .presets
        .iter()
        .find(|entry| entry.preset.catalog.artifact_id == "local-test")
        .unwrap();
    assert_eq!(entry.preset.catalog, pin);
    assert_eq!(entry.definition.origin, ClipPresetOrigin::HostManual);
    let doc = document(entry.preset.clone());
    let prepared = prepare_clip_preset(&doc, &port, &Control).unwrap();
    assert_eq!(prepared.evidence().pin, pin);
    assert!(!prepared.evidence().platform_requirements_verified);
    assert!(fs::OpenOptions::new().write(true).open(&path).is_err());
    assert!(fs::rename(&path, path.with_extension("moved")).is_err());
    let mut changed = doc.clone();
    changed.preset.width += 2;
    assert_eq!(
        prepare_clip_preset(&changed, &port, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    let mut wrong = doc.clone();
    wrong.preset.catalog.sha256 = "0".repeat(64);
    assert_eq!(
        prepare_clip_preset(&wrong, &port, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    let mut long = doc.clone();
    long.segments[0].range.end_us.0 = 2_000_001;
    assert_eq!(
        prepare_clip_preset(&long, &port, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    fs::write(
        root.join("catalog.json"),
        serde_json::to_vec_pretty(&catalog).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("admission.json"),
        serde_json::to_vec_pretty(prepared.evidence()).unwrap(),
    )
    .unwrap();
    for index in 0..4 {
        let mut variant = value.clone();
        match index {
            0 => variant["origin"] = json!("application_proposal"),
            1 => variant["preset_id"] = json!("foreign"),
            2 => variant["revision"] = json!("2"),
            _ => variant["shell"] = json!("untrusted"),
        }
        let variant_path = root.join(format!("rejected-{index}.json"));
        let variant_pin = write(&variant_path, &variant);
        let code = files(&variant_path)
            .open_preset(&variant_pin, &Control)
            .err()
            .unwrap()
            .code;
        assert_eq!(
            code,
            if index == 1 || index == 2 {
                ErrorCode::SourceExpired
            } else {
                ErrorCode::InvalidArgument
            }
        );
    }
    let builtin_id = BuiltinClipPresets::entries()
        .unwrap()
        .remove(0)
        .preset
        .catalog
        .artifact_id;
    assert_eq!(
        FixedClipPresets::open(BTreeMap::from([(builtin_id, path.clone())]))
            .err()
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
    struct Cancel;
    impl MutationControl for Cancel {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Err(error(ErrorCode::Cancelled, "fixture"))
        }
    }
    let absent = files(&root.join("absent.json"));
    assert_eq!(
        absent.open_preset(&pin, &Cancel).err().unwrap().code,
        ErrorCode::Cancelled
    );
    assert_eq!(hash(&fs::read(&path).unwrap()), pin.sha256);
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&json!({"definition_sha256":pin.sha256,"raw_bytes_and_catalog_identity_verified":true,"original_write_and_rename_denied":true,"altered_settings_pin_revision_id_origin_and_unknown_fields_rejected":true,"max_duration_enforced":true,"cancel_before_io_verified":true,"original_definition_preserved":true,"encoder_or_jobs_or_cli_verified":false,"platform_requirements_verified":false,"quality_approved":false})).unwrap()).unwrap();
}
