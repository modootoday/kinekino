use super::*;
use serde_json::json;
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
pub(crate) fn document(preset: ClipOutputPreset) -> ClipDocument {
    serde_json::from_value(json!({"schema_version":"1","document_id":"preset-short","revision":"1","sources":[{"media_id":"source","sha256":"a".repeat(64),"duration_us":"120000000","width":16,"height":16,"clock":{"origin_pts":"0","time_base_numerator":1,"time_base_denominator":1000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"h264"}]}],"segments":[{"segment_id":"one","media_id":"source","range":{"start_us":"0","end_us":"1000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null}],"preset":preset,"cut_mode":"exact_reencode"})).unwrap()
}
#[test]
fn application_proposals_are_versioned_distinct_and_not_engine_or_platform_approval() {
    let catalog = BuiltinClipPresets.inspect_presets().unwrap();
    assert!(catalog.definitions_available);
    assert!(!catalog.engine_availability_verified);
    assert_eq!(catalog.presets.len(), 2);
    let mut pins = std::collections::BTreeSet::new();
    for entry in catalog.presets {
        assert_eq!(
            entry.definition.origin,
            ClipPresetOrigin::ApplicationProposal
        );
        assert_eq!((entry.preset.width, entry.preset.height), (1080, 1920));
        assert_eq!(
            entry.preset.frame_rate,
            FrameRate {
                numerator: 30,
                denominator: 1
            }
        );
        assert_eq!(entry.definition.max_duration_us.0, 60_000_000);
        assert!(!entry.platform_requirements_verified);
        assert!(pins.insert(entry.preset.catalog.sha256.clone()));
        let prepared = prepare_clip_preset(
            &document(entry.preset.clone()),
            &BuiltinClipPresets,
            &Control,
        )
        .unwrap();
        assert_eq!(prepared.evidence().pin, entry.preset.catalog);
        assert!(!prepared.evidence().quality_approved);
        assert_eq!(prepared.evidence().definition_bytes, entry.definition_bytes);
    }
}
#[test]
fn altered_settings_hash_revision_unknown_profile_and_over_duration_are_explicit_errors() {
    let entry = BuiltinClipPresets::entries().unwrap().remove(0);
    let original = document(entry.preset);
    for change in 0..4 {
        let mut doc = original.clone();
        match change {
            0 => doc.preset.width -= 2,
            1 => doc.preset.catalog.sha256 = "0".repeat(64),
            2 => doc.preset.catalog.revision.0 += 1,
            _ => doc.preset.audio_codec = None,
        }
        assert_eq!(
            prepare_clip_preset(&doc, &BuiltinClipPresets, &Control)
                .err()
                .unwrap()
                .code,
            ErrorCode::SourceExpired
        );
    }
    let mut unknown = original.clone();
    unknown.preset.catalog.artifact_id = "unconfigured".into();
    assert_eq!(
        prepare_clip_preset(&unknown, &BuiltinClipPresets, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    let mut long = original;
    long.segments[0].range.end_us.0 = 60_000_001;
    assert_eq!(
        prepare_clip_preset(&long, &BuiltinClipPresets, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
}
#[test]
fn preset_definition_rejects_unbounded_or_malformed_data_without_assuming_render_support() {
    let valid = BuiltinClipPresets::entries().unwrap().remove(0).definition;
    for change in 0..5 {
        let mut definition = valid.clone();
        match change {
            0 => definition.revision.0 = 0,
            1 => definition.max_duration_us.0 = 180_000_001,
            2 => {
                definition.profile.frame_rate = FrameRate {
                    numerator: 60,
                    denominator: 2,
                }
            }
            3 => definition.profile.width = 8193,
            _ => definition.label = "\n".into(),
        }
        assert_eq!(
            definition.validate().unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    let mut wire = serde_json::to_value(valid).unwrap();
    wire["official_platform_approval"] = json!(true);
    assert!(serde_json::from_value::<ClipPresetDefinition>(wire).is_err());
    struct Cancel;
    impl MutationControl for Cancel {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Err(error(ErrorCode::Cancelled, "fixture"))
        }
    }
    let doc = document(BuiltinClipPresets::entries().unwrap().remove(0).preset);
    assert_eq!(
        prepare_clip_preset(&doc, &BuiltinClipPresets, &Cancel)
            .err()
            .unwrap()
            .code,
        ErrorCode::Cancelled
    );
}
