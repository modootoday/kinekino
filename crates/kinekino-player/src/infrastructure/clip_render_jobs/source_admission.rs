//! Parent source owners survive unwind; existing application media admission owns policy.
use super::*;
use crate::{
    domain::clip_document::ClipSourcePin,
    infrastructure::clip_sources::{NativeClipSourceLease, PreparedClipSourceProbe},
};
use std::cell::RefCell;
struct SourceOwner {
    expected: ClipSourcePin,
    probe: RefCell<PreparedClipSourceProbe>,
}
struct JobSourceOwners {
    sources: Vec<SourceOwner>,
}
impl JobSourceOwners {
    fn prepare(
        expected: &[ClipSourcePin],
        sources: &FfmpegClipSources,
        control: &dyn MutationControl,
    ) -> Result<Self, ApiError> {
        let mut retained = Vec::with_capacity(expected.len());
        for source in expected {
            control.checkpoint()?;
            let probe = sources.prepare_source_probe(&source.media_id, control)?;
            retained.push(SourceOwner {
                expected: source.clone(),
                probe: RefCell::new(probe),
            });
        }
        Ok(Self { sources: retained })
    }
    fn close(
        &self,
    ) -> Vec<crate::infrastructure::frame_capture::process::owned::operation::NativeProcessProgress>
    {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let progress = self
                .sources
                .iter()
                .map(|source| source.probe.borrow_mut().poll_close(deadline, None))
                .collect::<Vec<_>>();
            if progress.iter().all(|progress| progress.may_release()) {
                return progress;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn with_retained_owners<T>(
        &self,
        action: impl FnOnce() -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(action));
        match outcome {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(mut error)) => {
                let progress = self.close();
                if !error.details.is_object() {
                    error.details = json!({"original_details":error.details});
                }
                error.details["source_owner_cleanup"] =
                    serde_json::to_value(progress).expect("Parent source observations");
                Err(error)
            }
            Err(payload) => {
                let progress = self.close();
                let mut error = error(
                    ErrorCode::Internal,
                    "Original clip source admission panicked; retained source owners were closed",
                );
                error.details = json!({"stage":"clip_source_admission_unwind","source_owner_cleanup":progress,"panic_payload_retained_until_cleanup":true});
                drop(payload);
                Err(error)
            }
        }
    }
}
impl ClipSourceAdmission for JobSourceOwners {
    type Lease = NativeClipSourceLease;
    fn open_source(
        &self,
        expected: &ClipSourcePin,
        control: &dyn MutationControl,
    ) -> Result<Self::Lease, ApiError> {
        let original = self
            .sources
            .iter()
            .find(|source| &source.expected == expected)
            .ok_or_else(|| {
                error(
                    ErrorCode::SourceExpired,
                    "Source differs from original captured media admission",
                )
            })?;
        let mut probe = original.probe.borrow_mut();
        probe.run(control)?;
        probe.take_source(control)
    }
}
pub(super) fn prepare_job_media(
    projects: &dyn ClipProjectStore,
    request: SavedClipProjectPin,
    expected: &[ClipSourcePin],
    sources: &FfmpegClipSources,
    control: &dyn MutationControl,
) -> Result<PreparedClipMedia<NativeClipSourceLease>, ApiError> {
    let owners = JobSourceOwners::prepare(expected, sources, control)?;
    owners.with_retained_owners(|| prepare_media(projects, request, &owners, control))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::clip_sources::{ClipContainer, FixedClipSource};
    use std::{
        collections::BTreeMap,
        fs::{self, OpenOptions},
        sync::atomic::{AtomicUsize, Ordering},
    };
    struct Control;
    impl MutationControl for Control {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    #[test]
    #[ignore = "Actual original Jobs source owner holder, pinned probe/media, injected post-launch callback panic; fresh evidence"]
    fn job_source_parent_closes_original_probe_after_unwind_and_never_admits_failed_media() {
        let root = PathBuf::from(
            std::env::var_os("CLIP_JOB_SOURCE_OWNER_EVIDENCE_DIR").expect("Fresh evidence"),
        );
        fs::create_dir(&root).unwrap();
        let path = PathBuf::from(std::env::var_os("CLIP_RENDER_SOURCE").unwrap());
        let probe = PathBuf::from(std::env::var_os("CLIP_RENDER_PROBE").unwrap());
        let pin = std::env::var("CLIP_RENDER_PROBE_SHA256").unwrap();
        let sources = FfmpegClipSources::new(
            BTreeMap::from([(
                "source".into(),
                FixedClipSource {
                    path: path.clone(),
                    container: ClipContainer::Matroska,
                },
            )]),
            &probe,
            &pin,
            crate::domain::frame_capture::MAX_MEDIA_BYTES,
            Duration::from_secs(120),
        )
        .unwrap();
        let initial = sources.inspect_source("source", &Control).unwrap();
        let expected = initial.observation().clone();
        drop(initial);
        let owners =
            JobSourceOwners::prepare(std::slice::from_ref(&expected), &sources, &Control).unwrap();
        struct PanicOnSecond(AtomicUsize);
        impl MutationControl for PanicOnSecond {
            fn checkpoint(&self) -> Result<(), ApiError> {
                if self.0.fetch_add(1, Ordering::SeqCst) == 1 {
                    panic!("Expected Jobs source callback panic after original launch");
                }
                Ok(())
            }
        }
        let failure = owners
            .with_retained_owners(|| {
                owners.open_source(&expected, &PanicOnSecond(AtomicUsize::new(0)))
            })
            .err()
            .unwrap();
        assert_eq!(failure.code, ErrorCode::Internal);
        let cleanup = &failure.details["source_owner_cleanup"][0];
        assert_eq!(cleanup["not_started"], false);
        assert_eq!(cleanup["family_empty"], true);
        assert_eq!(cleanup["receipt_recorded"], true);
        assert!(cleanup["child_exit"].is_number());
        assert!(OpenOptions::new().write(true).open(&path).is_err());
        assert_eq!(
            owners.open_source(&expected, &Control).err().unwrap().code,
            ErrorCode::RevisionConflict
        );
        fs::write(
            root.join("failure.json"),
            serde_json::to_vec_pretty(&failure).unwrap(),
        )
        .unwrap();
        drop(owners);
        let owners =
            JobSourceOwners::prepare(std::slice::from_ref(&expected), &sources, &Control).unwrap();
        let retained = owners
            .with_retained_owners(|| owners.open_source(&expected, &Control))
            .unwrap();
        assert_eq!(retained.observation(), &expected);
        drop(owners);
        assert!(OpenOptions::new().write(true).open(&path).is_err());
        fs::write(
            root.join("facts.json"),
            serde_json::to_vec_pretty(retained.facts()).unwrap(),
        )
        .unwrap();
        fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&json!({"parent_retained_original_source_probe_across_unwind":true,"error_returned_only_after_family_and_receipt_closed":true,"original_media_guard_survived_cleanup":true,"failed_source_not_readmitted":true,"normal_source_lease_outlived_parent_holder":true,"all_jobs_native_stages_owned":false,"quality_approved":false})).unwrap()).unwrap();
    }
}
