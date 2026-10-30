use super::*;
use crate::{
    application::{
        clip_projects::{
            ClipProjectStore, ClipPublication, StoredClipProject, prepare_publication,
            publication_sha256,
        },
        clip_render::{SavedClipProjectPin, prepare_media},
    },
    domain::{api_values::DecimalU64, clip_document::ClipDocument},
};
use serde_json::json;
use std::fs::{self, OpenOptions};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Store(StoredClipProject);
impl ClipProjectStore for Store {
    fn load(&self, id: &str, revision: Option<DecimalU64>) -> Result<StoredClipProject, ApiError> {
        assert_eq!(id, "actual-short");
        assert_eq!(revision, Some(DecimalU64(1)));
        Ok(self.0.clone())
    }
    fn publish(
        &self,
        _: Option<DecimalU64>,
        _: &ClipDocument,
    ) -> Result<StoredClipProject, ApiError> {
        panic!("no project writes during admission")
    }
}
fn publication(source: &ClipSourcePin) -> ClipPublication {
    let document:ClipDocument=serde_json::from_value(json!({"schema_version":"1","document_id":"actual-short","revision":"1","sources":[source],"segments":[{"segment_id":"one","media_id":source.media_id,"range":{"start_us":"0","end_us":source.duration_us.0.min(1_000_000).to_string()},"video_stream_index":source.tracks.iter().filter(|track|track.kind==crate::domain::clip_document::SourceTrackKind::Video).map(|track|track.stream_index).min().unwrap(),"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null}],"preset":{"catalog":{"artifact_id":"candidate-preset","revision":"1","sha256":"c".repeat(64)},"width":1080,"height":1920,"frame_rate":{"numerator":30,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":null,"color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap();
    prepare_publication(None, &document).unwrap()
}
#[test]
#[ignore = "Explicit original media, pinned local FFprobe and fresh evidence; no encode/inference"]
fn actual_native_clip_probe_retains_source_tool_guards_and_matches_saved_project() {
    let root = PathBuf::from(std::env::var_os("CLIP_PROBE_EVIDENCE_DIR").expect("Fresh evidence"));
    assert!(root.is_absolute());
    fs::create_dir(&root).unwrap();
    let source = PathBuf::from(std::env::var_os("CLIP_PROBE_SOURCE").expect("Original media"));
    let probe = PathBuf::from(std::env::var_os("CLIP_PROBE_EXE").expect("Pinned probe"));
    let probe_sha = std::env::var("CLIP_PROBE_SHA256").unwrap().to_lowercase();
    let catalog = BTreeMap::from([(
        "actual-source".into(),
        FixedClipSource {
            path: source.clone(),
            container: ClipContainer::Matroska,
        },
    )]);
    let port = FfmpegClipSources::new(
        catalog,
        &probe,
        &probe_sha,
        MAX_MEDIA_BYTES,
        Duration::from_secs(120),
    )
    .unwrap();
    let lease = port.inspect_source("actual-source", &Control).unwrap();
    fs::write(root.join("raw-probe.json"), lease.raw_probe()).unwrap();
    fs::write(
        root.join("facts.json"),
        serde_json::to_vec_pretty(lease.facts()).unwrap(),
    )
    .unwrap();
    assert!(OpenOptions::new().write(true).open(&source).is_err());
    assert!(OpenOptions::new().write(true).open(&probe).is_err());
    let original = lease.observation().clone();
    let publication = publication(&original);
    let sha = publication_sha256(&publication).unwrap();
    let project = SavedClipProjectPin {
        document_id: "actual-short".into(),
        publication_revision: DecimalU64(1),
        publication_sha256: sha.clone(),
    };
    let store = Store(StoredClipProject {
        publication_sha256: sha,
        latest_publication_revision: DecimalU64(1),
        publication,
    });
    let prepared = prepare_media(&store, project.clone(), &port, &Control).unwrap();
    assert_eq!(prepared.sources()[0].observation(), &original);
    fs::write(
        root.join("saved-publication.json"),
        serde_json::to_vec_pretty(&store.0).unwrap(),
    )
    .unwrap();
    let mut changed = store.0.clone();
    changed.publication.document.sources[0].sha256 = "d".repeat(64);
    changed.publication_sha256 = publication_sha256(&changed.publication).unwrap();
    let changed_pin = SavedClipProjectPin {
        publication_sha256: changed.publication_sha256.clone(),
        ..project
    };
    match prepare_media(&Store(changed), changed_pin, &port, &Control) {
        Err(error) => assert_eq!(error.code, ErrorCode::SourceExpired),
        Ok(_) => panic!("foreign source accepted"),
    }
    drop(prepared);
    drop(lease);
    let after = port.inspect_source("actual-source", &Control).unwrap();
    assert_eq!(after.observation(), &original);
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&json!({"original_source_sha256":original.sha256,"probe_sha256":probe_sha,"facts_stable_across_reopen":true,"saved_project_matches":true,"changed_source_pin_rejected":true,"source_and_tool_write_denied_while_retained":true,"boundary_frames_verified":false,"encoded":false,"quality_approved":false})).unwrap()).unwrap();
}
