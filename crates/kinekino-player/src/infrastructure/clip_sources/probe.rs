//! Prepared original media/tool owner; preparation never starts an engine.
use super::*;
use crate::application::sdk_execution::ReceiptRecoveryControl;
use crate::infrastructure::frame_capture::process::owned::operation::{
    NativeProcessOperation, NativeProcessProgress,
};
pub(crate) struct PreparedClipSourceProbe {
    media_id: String,
    container: ClipContainer,
    source_sha256: String,
    path: PathBuf,
    guards: Option<Vec<File>>,
    tool: Arc<ProbeTool>,
    operation: NativeProcessOperation,
    deadline: Instant,
    attempted: bool,
    closed: bool,
    raw: Option<Vec<u8>>,
}

impl PreparedClipSourceProbe {
    pub(super) fn prepare(
        media_id: &str,
        selection: &FixedClipSource,
        tool: Arc<ProbeTool>,
        max_bytes: u64,
        deadline: Instant,
        outer: &dyn MutationControl,
    ) -> Result<Self, ApiError> {
        let control = ProbeControl { outer, deadline };
        control.checkpoint()?;
        let parent = selection.path.parent().ok_or_else(|| {
            ApiError::new(ErrorCode::InvalidArgument, "Missing clip source parent")
        })?;
        let name = selection.path.file_name().ok_or_else(|| {
            ApiError::new(ErrorCode::InvalidArgument, "Missing clip source filename")
        })?;
        if name
            .to_str()
            .is_none_or(|name| name.contains(':') || name.ends_with(['.', ' ']))
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Ordinary clip source filename required",
            ));
        }
        let (parent, mut guards) = paths::root(parent)?;
        let path = parent.join(name);
        let mut file = paths::file(&path)?;
        let mut header = [0u8; 12];
        file.read_exact(&mut header).map_err(|_| {
            ApiError::new(ErrorCode::NeedsInput, "Incomplete clip container header")
        })?;
        if !selection.container.matches(&header) {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Clip source signature differs from configured container",
            ));
        }
        file.seek(SeekFrom::Start(0)).map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "Cannot rewind retained clip source",
            )
        })?;
        let (_, source_sha256) =
            content_digest::sha256_with_checkpoint(&mut file, max_bytes, || control.checkpoint())
                .map_err(|error| error.into_api_error())?;
        guards.push(file);
        control.checkpoint()?;
        let mut command = Command::new(&tool.path);
        command.args([
            "-v",
            "error",
            "-threads",
            "1",
            "-max_alloc",
            "268435456",
            "-probesize",
            "16777216",
            "-analyzeduration",
            "10000000",
            "-protocol_whitelist",
            "file",
            "-f",
            selection.container.demuxer(),
        ]);
        if matches!(selection.container, ClipContainer::Mp4) {
            command.args(["-enable_drefs", "0"]);
        }
        command.args(["-show_entries","stream=index,codec_type,codec_name,width,height,time_base,start_pts,duration_ts,color_transfer:stream_disposition=attached_pic:stream_side_data=rotation:format=duration","-of","json"]).arg(&path);

        let operation =
            NativeProcessOperation::prepare(&command, deadline, 262144, Some(&control), None)?;
        Ok(Self {
            media_id: media_id.into(),
            container: selection.container,
            source_sha256,
            path,
            guards: Some(guards),
            tool,
            operation,
            deadline,
            attempted: false,
            closed: false,
            raw: None,
        })
    }
    pub(crate) fn run(&mut self, outer: &dyn MutationControl) -> Result<(), ApiError> {
        if self.attempted || self.closed {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original clip source probe was already attempted or closed",
            ));
        }
        self.attempted = true;
        let control = ProbeControl {
            outer,
            deadline: self.deadline,
        };
        control.checkpoint()?;
        self.operation.launch()?;
        loop {
            if self.operation.poll(Some(&control), None) {
                let output = self
                    .operation
                    .take_result()
                    .expect("Settled original clip probe")?;
                control.checkpoint()?;
                self.raw = Some(output.stdout);
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    pub(crate) fn progress(&self) -> NativeProcessProgress {
        self.operation
            .progress()
            .expect("Original prepared probe observation")
    }
    /// On faults retain this same prepared owner; never reconstruct it from a receipt.
    pub(crate) fn poll_close(
        &mut self,
        deadline: Instant,
        recovery: Option<&dyn ReceiptRecoveryControl>,
    ) -> NativeProcessProgress {
        self.closed = true;
        let progress = self.progress();
        if !progress.not_started && !progress.may_release() {
            struct Closing(Instant);
            impl MutationControl for Closing {
                fn checkpoint(&self) -> Result<(), ApiError> {
                    Err(ApiError::new(
                        if Instant::now() >= self.0 {
                            ErrorCode::Timeout
                        } else {
                            ErrorCode::EngineFailed
                        },
                        "Original clip source probe owner is closing",
                    ))
                }
            }
            self.operation.poll(Some(&Closing(deadline)), recovery);
        }
        self.progress()
    }
    pub(crate) fn take_source(
        &mut self,
        outer: &dyn MutationControl,
    ) -> Result<NativeClipSourceLease, ApiError> {
        let control = ProbeControl {
            outer,
            deadline: self.deadline,
        };
        control.checkpoint()?;
        if self.closed || !self.progress().may_release() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original source probe is not available for admission",
            ));
        }
        let raw = self.raw.as_ref().ok_or_else(|| {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Original probe output is not ready or was already consumed",
            )
        })?;
        let facts =
            observation::parse(raw, &self.media_id, &self.source_sha256, &self.tool.sha256)?;
        control.checkpoint()?;
        Ok(NativeClipSourceLease {
            custody: Arc::new(()),
            container: self.container,
            facts,
            path: self.path.clone(),
            raw_probe: self.raw.take().expect("Original probe bytes"),
            _guards: self.guards.take().expect("Original source guards"),
            _tool: self.tool.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Control;
    impl MutationControl for Control {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    #[test]
    #[ignore = "Actual pinned source/probe; original prepared owner before launch and across caught callback panic; fresh evidence"]
    fn prepared_source_probe_retains_media_before_launch_and_across_unwind() {
        use std::fs::{self, OpenOptions};
        use std::sync::atomic::{AtomicUsize, Ordering};
        let root = PathBuf::from(
            std::env::var_os("CLIP_PREPARED_PROBE_EVIDENCE_DIR").expect("Fresh evidence"),
        );
        fs::create_dir(&root).unwrap();
        let source = PathBuf::from(std::env::var_os("CLIP_RENDER_SOURCE").unwrap());
        let executable = PathBuf::from(std::env::var_os("CLIP_RENDER_PROBE").unwrap());
        let pin = std::env::var("CLIP_RENDER_PROBE_SHA256").unwrap();
        let catalog = FfmpegClipSources::new(
            BTreeMap::from([(
                "source".into(),
                FixedClipSource {
                    path: source.clone(),
                    container: ClipContainer::Matroska,
                },
            )]),
            &executable,
            &pin,
            MAX_MEDIA_BYTES,
            Duration::from_secs(120),
        )
        .unwrap();
        let mut closed = catalog.prepare_source_probe("source", &Control).unwrap();
        assert!(closed.progress().not_started);
        assert!(OpenOptions::new().write(true).open(&source).is_err());
        assert!(closed.poll_close(Instant::now(), None).not_started);
        assert_eq!(
            closed.run(&Control).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert!(closed.take_source(&Control).is_err());
        drop(closed);
        let mut unwinding = catalog.prepare_source_probe("source", &Control).unwrap();
        struct PanicOnSecond(AtomicUsize);
        impl MutationControl for PanicOnSecond {
            fn checkpoint(&self) -> Result<(), ApiError> {
                if self.0.fetch_add(1, Ordering::SeqCst) == 1 {
                    panic!("Expected callback panic after original probe launch");
                }
                Ok(())
            }
        }
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            unwinding.run(&PanicOnSecond(AtomicUsize::new(0)))
        }));
        assert!(caught.is_err());
        assert!(!unwinding.progress().not_started);
        assert!(OpenOptions::new().write(true).open(&source).is_err());
        let close_deadline = Instant::now() + Duration::from_secs(3);
        let closed = loop {
            let progress = unwinding.poll_close(close_deadline, None);
            if progress.may_release() {
                break progress;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(closed.child_exit.is_some() && closed.family_empty && closed.receipt_recorded);
        fs::write(
            root.join("unwind-close.json"),
            serde_json::to_vec_pretty(&closed).unwrap(),
        )
        .unwrap();
        assert!(unwinding.take_source(&Control).is_err());
        drop(unwinding);
        let mut prepared = catalog.prepare_source_probe("source", &Control).unwrap();
        assert!(prepared.progress().not_started);
        prepared.run(&Control).unwrap();
        let progress = prepared.progress();
        assert!(progress.may_release());
        assert!(!progress.forced);
        assert_eq!(
            prepared.run(&Control).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        let lease = prepared.take_source(&Control).unwrap();
        assert!(prepared.take_source(&Control).is_err());
        assert!(OpenOptions::new().write(true).open(&source).is_err());
        assert!(OpenOptions::new().write(true).open(&executable).is_err());
        fs::write(
            root.join("facts.json"),
            serde_json::to_vec_pretty(lease.facts()).unwrap(),
        )
        .unwrap();
        fs::write(root.join("raw-probe.json"), lease.raw_probe()).unwrap();
        fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"prepared_without_launch":true,"close_before_launch_prevents_later_run":true,"original_media_guard_retained_across_caught_run_panic":true,"same_original_operation_family_and_receipt_closed_after_unwind":true,"normal_probe_admitted_once":true,"source_and_tool_guard_transferred_to_lease":true,"jobs_owned_attempt_connected":false,"quality_approved":false})).unwrap()).unwrap();
    }
}
