use super::*;
use crate::application::clip_render::presets::BuiltinClipPresets;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
pub(crate) fn document() -> ClipDocument {
    let mut document: ClipDocument = serde_json::from_value(serde_json::json!({"schema_version":"1","document_id":"encoder-short","revision":"1","sources":[{"media_id":"source","sha256":"a".repeat(64),"duration_us":"60000000","width":1920,"height":1080,"clock":{"origin_pts":"0","time_base_numerator":1,"time_base_denominator":1000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"h264"},{"stream_index":1,"kind":"audio","codec_id":"aac"}]}],"segments":[{"segment_id":"one","media_id":"source","range":{"start_us":"0","end_us":"1000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"selected","stream_index":1,"volume_percent":75,"muted":false},"subtitles":null,"subject_ref":null}],"preset":{"catalog":{"artifact_id":"candidate","revision":"1","sha256":"c".repeat(64)},"width":1080,"height":1920,"frame_rate":{"numerator":30,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":"aac","color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap();
    document.preset = crate::application::clip_render::presets::BuiltinClipPresets::entries()
        .unwrap()
        .remove(0)
        .preset;
    document
}
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Lease {
    inventory: ClipEncoderInventory,
    drops: Arc<AtomicUsize>,
}
impl ClipEncoderLease for Lease {
    fn inventory(&self) -> &ClipEncoderInventory {
        &self.inventory
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
struct Admission {
    inventory: ClipEncoderInventory,
    opens: AtomicUsize,
    drops: Arc<AtomicUsize>,
}
impl ClipEncoderAdmission for Admission {
    type Lease = Lease;
    fn open_encoder(&self, control: &dyn MutationControl) -> Result<Lease, ApiError> {
        control.checkpoint()?;
        self.opens.fetch_add(1, Ordering::SeqCst);
        Ok(Lease {
            inventory: self.inventory.clone(),
            drops: self.drops.clone(),
        })
    }
}
fn admission() -> Admission {
    let required = encoder_requirements(&document()).unwrap();
    Admission {
        inventory: ClipEncoderInventory {
            tool_sha256: "a".repeat(64),
            inventory_sha256: "b".repeat(64),
            version_line: "ffmpeg version synthetic".into(),
            build_configuration_sha256: "c".repeat(64),
            license_sha256: "d".repeat(64),
            encoders: BTreeSet::from(["libx264".into(), "aac".into()]),
            filters: required.filters,
            muxers: BTreeSet::from(["mp4".into()]),
        },
        opens: AtomicUsize::new(0),
        drops: Arc::new(AtomicUsize::new(0)),
    }
}
fn code<T>(result: Result<T, ApiError>) -> ErrorCode {
    match result {
        Err(error) => error.code,
        Ok(_) => panic!("unexpected admission"),
    }
}
#[test]
fn optional_audio_and_selected_subtitles_derive_required_features_without_approving_assets() {
    let mut doc = document();
    let required = encoder_requirements(&doc).unwrap();
    assert_eq!(required.audio_encoder.as_deref(), Some("aac"));
    assert!(required.filters.contains("volume"));
    assert!(!required.filters.contains("subtitles"));
    doc.segments[0].audio = AudioSelection::Disabled;
    let required = encoder_requirements(&doc).unwrap();
    assert!(required.audio_encoder.is_none());
    assert!(!required.filters.contains("atrim"));
    doc.segments[0].subtitles=Some(serde_json::from_value(serde_json::json!({"document":{"artifact":{"artifact_id":"subs","revision":"1","sha256":"e".repeat(64)},"source_sha256":"a".repeat(64)},"cue_ids":["cue-1"],"style":{"artifact_id":"style","revision":"1","sha256":"f".repeat(64)}})).unwrap());
    assert!(
        encoder_requirements(&doc)
            .unwrap()
            .filters
            .contains("overlay")
    );
    assert!(
        !encoder_requirements(&doc)
            .unwrap()
            .filters
            .contains("subtitles")
    );
}
#[test]
fn exact_document_digest_and_original_encoder_lease_survive_until_prepared_owner_drops() {
    let admission = admission();
    let doc = document();
    let original = serde_json::to_vec(&doc).unwrap();
    let prepared = prepare_encoder(&doc, &BuiltinClipPresets, &admission, &Control).unwrap();
    assert_eq!(admission.drops.load(Ordering::SeqCst), 0);
    assert_eq!(prepared.preset(), &doc.preset);
    assert_eq!(prepared.requirements().video_encoder, "libx264");
    assert!(prepared.lease().inventory().encoders.contains("aac"));
    let mut changed = doc.clone();
    changed.segments[0].range.end_us.0 += 1;
    let another = prepare_encoder(&changed, &BuiltinClipPresets, &admission, &Control).unwrap();
    assert_ne!(prepared.document_sha256(), another.document_sha256());
    assert_eq!(serde_json::to_vec(&doc).unwrap(), original);
    drop(another);
    drop(prepared);
    assert_eq!(admission.drops.load(Ordering::SeqCst), 2);
}
#[test]
fn missing_features_and_incomplete_inventory_release_tool_and_keep_needs_input() {
    for missing in 0..4 {
        let mut admission = admission();
        match missing {
            0 => {
                admission.inventory.encoders.remove("libx264");
            }
            1 => {
                admission.inventory.filters.remove("crop");
            }
            2 => {
                admission.inventory.muxers.clear();
            }
            _ => admission.inventory.license_sha256 = "".into(),
        }
        assert_eq!(
            code(prepare_encoder(
                &document(),
                &BuiltinClipPresets,
                &admission,
                &Control
            )),
            ErrorCode::NeedsInput
        );
        assert_eq!(admission.drops.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn unverified_profiles_reject_before_open_and_cancellation_releases_original_lease() {
    let admission = admission();
    for unsupported in 0..3 {
        let mut doc = document();
        match unsupported {
            0 => doc.cut_mode = CutMode::FastKeyframe,
            1 => doc.preset.width = 1079,
            _ => doc.preset.container = Container::Webm,
        };
        assert_eq!(
            code(prepare_encoder(
                &doc,
                &BuiltinClipPresets,
                &admission,
                &Control
            )),
            ErrorCode::Unsupported
        );
    }
    assert_eq!(admission.opens.load(Ordering::SeqCst), 0);
    struct Cancel<'a>(&'a AtomicUsize);
    impl MutationControl for Cancel<'_> {
        fn checkpoint(&self) -> Result<(), ApiError> {
            if self.0.load(Ordering::SeqCst) != 0 {
                Err(ApiError::new(ErrorCode::Cancelled, "cancelled"))
            } else {
                Ok(())
            }
        }
    }
    assert_eq!(
        code(prepare_encoder(
            &document(),
            &BuiltinClipPresets,
            &admission,
            &Cancel(&admission.opens)
        )),
        ErrorCode::Cancelled
    );
    assert_eq!(admission.drops.load(Ordering::SeqCst), 1);
}
#[test]
fn altered_preset_settings_reject_before_encoder_open() {
    let admission = admission();
    let mut doc = document();
    doc.preset.width -= 2;
    assert_eq!(
        code(prepare_encoder(
            &doc,
            &BuiltinClipPresets,
            &admission,
            &Control
        )),
        ErrorCode::SourceExpired
    );
    assert_eq!(admission.opens.load(Ordering::SeqCst), 0);
}
