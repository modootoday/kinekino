use super::*;
use crate::{
    application::clip_projects::{StoredClipProject, prepare_publication},
    domain::clip_document::ClipDocument,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
fn stored() -> StoredClipProject {
    let document: ClipDocument = serde_json::from_value(serde_json::json!({"schema_version":"1","document_id":"short","revision":"1","sources":[{"media_id":"source","sha256":"a".repeat(64),"duration_us":"60000000","width":1,"height":1,"clock":{"origin_pts":"-9000","time_base_numerator":1,"time_base_denominator":90000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"h264"},{"stream_index":2,"kind":"subtitle","codec_id":"subrip"}]}],"segments":[{"segment_id":"segment","media_id":"source","range":{"start_us":"0","end_us":"30000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null}],"preset":{"catalog":{"artifact_id":"preset","revision":"1","sha256":"c".repeat(64)},"width":1,"height":1,"frame_rate":{"numerator":1,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":null,"color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap();
    let publication = prepare_publication(None, &document).unwrap();
    StoredClipProject {
        publication_sha256: publication_sha256(&publication).unwrap(),
        latest_publication_revision: DecimalU64(5),
        publication,
    }
}
struct Store(StoredClipProject);
impl ClipProjectStore for Store {
    fn load(&self, id: &str, revision: Option<DecimalU64>) -> Result<StoredClipProject, ApiError> {
        assert_eq!(id, "short");
        assert_eq!(revision, Some(DecimalU64(1)));
        Ok(self.0.clone())
    }
    fn publish(
        &self,
        _: Option<DecimalU64>,
        _: &ClipDocument,
    ) -> Result<StoredClipProject, ApiError> {
        panic!("preflight cannot publish")
    }
}
fn pin(stored: &StoredClipProject) -> SavedClipProjectPin {
    SavedClipProjectPin {
        document_id: "short".into(),
        publication_revision: DecimalU64(1),
        publication_sha256: stored.publication_sha256.clone(),
    }
}
struct Control {
    calls: AtomicUsize,
    cancel_at: usize,
}
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.calls.fetch_add(1, Ordering::SeqCst) == self.cancel_at {
            Err(ApiError::new(ErrorCode::Cancelled, "explicit cancellation"))
        } else {
            Ok(())
        }
    }
}
fn control() -> Control {
    Control {
        calls: AtomicUsize::new(0),
        cancel_at: usize::MAX,
    }
}
struct Lease {
    observed: ClipSourcePin,
    drops: Arc<AtomicUsize>,
}
impl ClipSourceLease for Lease {
    fn observation(&self) -> &ClipSourcePin {
        &self.observed
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
struct Admission {
    observed: ClipSourcePin,
    opens: AtomicUsize,
    drops: Arc<AtomicUsize>,
    error: bool,
}
impl ClipSourceAdmission for Admission {
    type Lease = Lease;
    fn open_source(
        &self,
        _: &ClipSourcePin,
        control: &dyn MutationControl,
    ) -> Result<Lease, ApiError> {
        control.checkpoint()?;
        self.opens.fetch_add(1, Ordering::SeqCst);
        if self.error {
            return Err(ApiError::new(ErrorCode::NeedsInput, "source unavailable"));
        }
        Ok(Lease {
            observed: self.observed.clone(),
            drops: self.drops.clone(),
        })
    }
}
fn admission(stored: &StoredClipProject) -> Admission {
    Admission {
        observed: stored.publication.document.sources[0].clone(),
        opens: AtomicUsize::new(0),
        drops: Arc::new(AtomicUsize::new(0)),
        error: false,
    }
}
fn code<T>(result: Result<T, ApiError>) -> ErrorCode {
    match result {
        Err(error) => error.code,
        Ok(_) => panic!("unexpected admission"),
    }
}
#[test]
fn historical_pin_keeps_negative_clock_tracks_and_original_lease_until_drop() {
    let saved = stored();
    let original = serde_json::to_vec(&saved).unwrap();
    let mut admission = admission(&saved);
    admission.observed.tracks.reverse();
    let expected = pin(&saved);
    let store = Store(saved);
    let prepared = prepare_media(&store, expected, &admission, &control()).unwrap();
    assert_eq!(prepared.project().publication_revision, DecimalU64(1));
    assert_eq!(
        prepared.publication().document.sources[0].clock.origin_pts,
        "-9000"
    );
    assert_eq!(prepared.sources().len(), 1);
    assert_eq!(admission.drops.load(Ordering::SeqCst), 0);
    assert_eq!(serde_json::to_vec(&store.0).unwrap(), original);
    drop(prepared);
    assert_eq!(admission.drops.load(Ordering::SeqCst), 1);
}
#[test]
fn stale_or_corrupt_publication_stops_before_opening_media() {
    for corruption in 0..4 {
        let mut saved = stored();
        let mut expected = pin(&saved);
        let admission = admission(&saved);
        match corruption {
            0 => expected.publication_sha256 = "d".repeat(64),
            1 => saved.publication.document.preset.width = 2,
            2 => saved.publication.document.document_id = "foreign".into(),
            _ => saved.publication.parent_publication_sha256 = Some("e".repeat(64)),
        }
        assert_eq!(
            code(prepare_media(
                &Store(saved),
                expected,
                &admission,
                &control()
            )),
            ErrorCode::SourceExpired
        );
        assert_eq!(admission.opens.load(Ordering::SeqCst), 0);
    }
}
#[test]
fn hash_dimensions_duration_clock_and_track_mismatch_reject_and_release_lease() {
    for mismatch in 0..7 {
        let saved = stored();
        let mut admission = admission(&saved);
        match mismatch {
            0 => admission.observed.sha256 = "d".repeat(64),
            1 => admission.observed.width = 2,
            2 => admission.observed.duration_us = DecimalU64(30_000_000),
            3 => admission.observed.clock.origin_pts = "0".into(),
            4 => admission.observed.clock.time_base_denominator = 1000,
            5 => admission.observed.clock.observation_sha256 = "e".repeat(64),
            _ => admission.observed.tracks[0].stream_index = 1,
        }
        let expected = pin(&saved);
        assert_eq!(
            code(prepare_media(
                &Store(saved),
                expected,
                &admission,
                &control()
            )),
            ErrorCode::SourceExpired
        );
        assert_eq!(admission.drops.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn cancellation_and_missing_source_preserve_original_error_and_release_partial_custody() {
    let saved = stored();
    let expected = pin(&saved);
    let admission = admission(&saved);
    let cancelled = Control {
        calls: AtomicUsize::new(0),
        cancel_at: 4,
    };
    assert_eq!(
        code(prepare_media(
            &Store(saved.clone()),
            expected.clone(),
            &admission,
            &cancelled
        )),
        ErrorCode::Cancelled
    );
    assert_eq!(admission.opens.load(Ordering::SeqCst), 1);
    assert_eq!(admission.drops.load(Ordering::SeqCst), 1);
    let missing = Admission {
        error: true,
        ..admission
    };
    assert_eq!(
        code(prepare_media(&Store(saved), expected, &missing, &control())),
        ErrorCode::NeedsInput
    );
    assert_eq!(missing.drops.load(Ordering::SeqCst), 1);
}
