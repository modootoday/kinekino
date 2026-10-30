//! Real pinned local decoder with test-only gates around the original publication.
use super::*;
use crate::{
    application::{
        api::PlayerSdk,
        frame_capture::{CapturedFrame, FrameCaptureCapability, FrameCapturePort},
        review_source::MutationControl,
    },
    domain::frame_capture::CaptureFrame,
    infrastructure::{
        content_digest, frame_capture::FfmpegFrameCapture, subtitle_files::SubtitleFiles,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

struct Gate {
    before: bool,
    directory: PathBuf,
    entered: mpsc::Sender<()>,
    release: (Mutex<bool>, Condvar),
    reached: AtomicBool,
}
impl Gate {
    fn wait(&self) -> Result<(), ApiError> {
        self.reached.store(true, Ordering::SeqCst);
        self.entered
            .send(())
            .map_err(|_| internal("Publication observer gone"))?;
        let (released, timeout) = self
            .release
            .1
            .wait_timeout_while(
                self.release.0.lock().unwrap(),
                Duration::from_secs(10),
                |released| !*released,
            )
            .unwrap();
        if timeout.timed_out() && !*released {
            return Err(internal("Publication gate release expired"));
        }
        Ok(())
    }
}
struct Release(Arc<Gate>);
impl Drop for Release {
    fn drop(&mut self) {
        *self.0.release.0.lock().unwrap() = true;
        self.0.release.1.notify_all();
    }
}
struct Guard<'a> {
    original: &'a dyn MutationControl,
    gate: &'a Gate,
}
impl MutationControl for Guard<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        let temporary = fs::read_dir(&self.gate.directory).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".export-")
        });
        if self.gate.before && temporary && !self.gate.reached.load(Ordering::SeqCst) {
            self.gate.wait()?;
        }
        self.original.checkpoint()
    }
}
struct Port {
    native: FfmpegFrameCapture,
    gate: Arc<Gate>,
}
impl FrameCapturePort for Port {
    fn capability(&self) -> FrameCaptureCapability {
        self.native.capability()
    }
    fn cooperative_cancel_supported(&self) -> bool {
        true
    }
    fn capture(&self, _: &CaptureFrame, _: u64) -> Result<CapturedFrame, ApiError> {
        Err(internal("Test requires original SDK execution control"))
    }
    fn capture_controlled(
        &self,
        request: &CaptureFrame,
        timeout: u64,
        control: &dyn MutationControl,
    ) -> Result<CapturedFrame, ApiError> {
        let result = self.native.capture_controlled(
            request,
            timeout,
            &Guard {
                original: control,
                gate: &self.gate,
            },
        );
        if !self.gate.before && result.is_ok() {
            self.gate.wait()?;
        }
        result
    }
}
fn sha(path: &Path) -> String {
    content_digest::sha256(&mut fs::File::open(path).unwrap(), 512 * 1024 * 1024)
        .unwrap()
        .1
}

#[test]
#[ignore = "Central allocation: existing pinned local media and real decoder; no GUI/inference/download"]
fn real_frame_cancel_before_publication_and_after_saved_success_preserves_original_result() {
    let variable = |key| std::env::var(key).expect("explicit frame publication configuration");
    let media = PathBuf::from(variable("FRAME_HOST_MEDIA"));
    let media_sha = "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483";
    let png_sha = "0b73162584f488c517833254e029ccb4d26f6c966d126d7de7e63f1dd7b9de2a";
    assert_eq!(sha(&media), media_sha);
    for before in [true, false] {
        let directory =
            std::env::temp_dir().join(format!("frame-publication-{}-{before}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let output = directory.join("frame.png");
        let (entered, receive) = mpsc::channel();
        let gate = Arc::new(Gate {
            before,
            directory: directory.clone(),
            entered,
            release: (Mutex::new(false), Condvar::new()),
            reached: AtomicBool::new(false),
        });
        let release = Release(gate.clone());
        let native = FfmpegFrameCapture::pinned(
            Path::new(&variable("DISCOVERY_FRAME_FFMPEG")),
            &variable("DISCOVERY_FRAME_FFMPEG_SHA256"),
            Path::new(&variable("DISCOVERY_FRAME_FFPROBE")),
            &variable("DISCOVERY_FRAME_FFPROBE_SHA256"),
        )
        .unwrap();
        let client = Arc::new(PlayerSdk::new(SubtitleFiles).with_frame_capture(Port {
            native,
            gate: gate.clone(),
        }));
        let owner = ExecutionOwner::start(
            client,
            budget(),
            Arc::new(Notify {
                signals: AtomicUsize::new(0),
                fail: false,
            }),
        )
        .unwrap();
        let handle = owner.handle();
        let mut request = Request::new("production.capture_frame");
        request.context.timeout_ms = Some(30000);
        request.payload =
            serde_json::json!({"source_path":media,"source_pts_us":"500000","output_path":output});
        let id = handle.submit(request).unwrap();
        receive.recv_timeout(Duration::from_secs(10)).unwrap();
        if before {
            assert!(!output.exists());
            let temporary = fs::read_dir(&directory)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            assert_eq!(sha(&temporary), png_sha);
        } else {
            assert_eq!(sha(&output), png_sha);
            assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        }
        assert_eq!(handle.cancel(&id).unwrap(), CancelDisposition::Requested);
        drop(release);
        let done = terminal(&handle, &id);
        assert!(done.cancellation_requested);
        assert!(!done.queued_cancelled);
        done.validate_response().unwrap();
        if before {
            assert_eq!(
                done.response.error.as_ref().unwrap().code,
                ErrorCode::Cancelled
            );
            assert!(!output.exists());
            assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        } else {
            assert!(done.response.ok);
            assert_eq!(
                done.response.data.as_ref().unwrap()["output_sha256"],
                png_sha
            );
            assert_eq!(sha(&output), png_sha);
        }
        assert_eq!(sha(&media), media_sha);
        fs::write(
            directory.join("response.json"),
            serde_json::to_vec_pretty(done.response.as_ref()).unwrap(),
        )
        .unwrap();
        handle.acknowledge(&id).unwrap();
        let next = handle.submit(Request::new("version")).unwrap();
        assert!(terminal(&handle, &next).response.ok);
        handle.acknowledge(&next).unwrap();
        close(&handle);
    }
}
