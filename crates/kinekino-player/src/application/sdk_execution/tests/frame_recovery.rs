//! Real SDK/native probe receipt fault with a test-only checkpoint decorator.
use super::*;
use crate::{
    application::{
        api::PlayerSdk,
        frame_capture::{
            CapturedFrame, FrameCaptureCapability, FrameCapturePort, FrameExecutionContext,
        },
        review_source::MutationControl,
    },
    domain::frame_capture::CaptureFrame,
    infrastructure::{
        content_digest, frame_capture::FfmpegFrameCapture, subtitle_files::SubtitleFiles,
    },
};
use std::{
    cell::Cell,
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
const SENTINEL: &[u8] = b"original SDK receipt collision evidence";
fn roots() -> BTreeSet<PathBuf> {
    let prefix = format!("frame-owned-{}-", std::process::id());
    fs::read_dir(std::env::temp_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&prefix)
        })
        .collect()
}
struct Fault<'a> {
    collisions: usize,
    original: &'a dyn MutationControl,
    known: &'a BTreeSet<PathBuf>,
    sent: &'a mpsc::Sender<PathBuf>,
    created: Cell<bool>,
}
impl MutationControl for Fault<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if !self.created.get()
            && let Some(root) = roots().difference(self.known).next()
        {
            for index in 0..self.collisions {
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join(format!("receipt-{index:04}.json")))
                    .unwrap();
                file.write_all(SENTINEL).unwrap();
                file.sync_all().unwrap();
            }
            self.created.set(true);
            self.sent.send(root.clone()).unwrap();
        }
        self.original.checkpoint()
    }
}
struct Port {
    collisions: usize,
    native: FfmpegFrameCapture,
    known: BTreeSet<PathBuf>,
    sent: mpsc::Sender<PathBuf>,
}
impl FrameCapturePort for Port {
    fn capability(&self) -> FrameCaptureCapability {
        self.native.capability()
    }
    fn capture(&self, request: &CaptureFrame, timeout: u64) -> Result<CapturedFrame, ApiError> {
        self.native.capture(request, timeout)
    }
    fn capture_recoverable(
        &self,
        request: &CaptureFrame,
        timeout: u64,
        context: FrameExecutionContext<'_>,
    ) -> Result<CapturedFrame, ApiError> {
        assert!(
            context.recovery.is_some(),
            "original SDK recovery port must arrive"
        );
        let fault = Fault {
            collisions: self.collisions,
            original: context.control.unwrap(),
            known: &self.known,
            sent: &self.sent,
            created: Cell::new(false),
        };
        self.native.capture_recoverable(
            request,
            timeout,
            FrameExecutionContext {
                control: Some(&fault),
                recovery: context.recovery,
            },
        )
    }
}
fn sha(path: &Path) -> String {
    content_digest::sha256(&mut fs::File::open(path).unwrap(), 512 * 1024 * 1024)
        .unwrap()
        .1
}
#[test]
#[ignore = "Central allocation: real existing local media and probe, original SDK receipt fault; no inference/GUI/download"]
fn original_frame_sdk_receipt_retry_retains_source_guard_and_first_error() {
    run(1);
}
#[test]
#[ignore = "Central allocation: two real immutable receipt collisions; original SDK/native probe"]
fn repeated_frame_receipt_failure_requires_new_generation_without_automatic_retry() {
    run(2);
}
fn run(collisions: usize) {
    let variable = |key| std::env::var(key).expect("explicit pinned frame recovery configuration");
    let media = PathBuf::from(variable("FRAME_HOST_MEDIA"));
    let original_sha = "0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483";
    assert_eq!(sha(&media), original_sha);
    let native = FfmpegFrameCapture::pinned(
        Path::new(&variable("DISCOVERY_FRAME_FFMPEG")),
        &variable("DISCOVERY_FRAME_FFMPEG_SHA256"),
        Path::new(&variable("DISCOVERY_FRAME_FFPROBE")),
        &variable("DISCOVERY_FRAME_FFPROBE_SHA256"),
    )
    .unwrap();
    assert!(native.capability().available);
    let (sent, received) = mpsc::channel();
    let client = Arc::new(PlayerSdk::new(SubtitleFiles).with_frame_capture(Port {
        collisions,
        native,
        known: roots(),
        sent,
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
    let output = std::env::temp_dir().join(format!("frame-recovery-{}.png", std::process::id()));
    assert!(!output.exists());
    let mut request = Request::new("production.capture_frame");
    request.payload =
        serde_json::json!({"source_path":media,"source_pts_us":"500000","output_path":output});
    let id = handle.submit(request).unwrap();
    let root = received.recv_timeout(Duration::from_secs(5)).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(5);
    let pending = loop {
        if let Some(pending) = handle.receipt_recovery(&id).unwrap() {
            break pending;
        }
        assert!(Instant::now() < cutoff);
        assert!(matches!(
            handle.peek(&id).unwrap(),
            ExecutionSnapshot::Running { .. }
        ));
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(pending.generation(), 1);
    assert!(!pending.retry_requested);
    assert_eq!(fs::read(root.join("receipt-0000.json")).unwrap(), SENTINEL);
    assert!(!root.join(format!("receipt-{collisions:04}.json")).exists());
    use std::os::windows::fs::OpenOptionsExt;
    assert!(
        fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&media)
            .is_err(),
        "original source guard remains held while receipt pending"
    );
    assert_eq!(handle.cancel(&id).unwrap(), CancelDisposition::Requested);
    assert!(matches!(
        handle.peek(&id).unwrap(),
        ExecutionSnapshot::Running { .. }
    ));
    assert_eq!(
        handle.retry_receipt(&id, &pending.token).unwrap(),
        CleanupRetryDisposition::Requested
    );
    let mut previous = pending.token;
    for expected in 2..=collisions {
        let cutoff = Instant::now() + Duration::from_secs(3);
        let pending = loop {
            if let Some(pending) = handle.receipt_recovery(&id).unwrap()
                && usize::from(pending.generation()) == expected
            {
                break pending;
            }
            assert!(Instant::now() < cutoff);
            assert!(matches!(
                handle.peek(&id).unwrap(),
                ExecutionSnapshot::Running { .. }
            ));
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(!pending.retry_requested);
        assert!(!pending.attempt_budget_exhausted);
        assert_eq!(
            handle.retry_receipt(&id, &previous).unwrap_err().code,
            ErrorCode::SourceExpired
        );
        assert!(!root.join(format!("receipt-{collisions:04}.json")).exists());
        assert!(
            fs::OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(&media)
                .is_err()
        );
        assert_eq!(
            handle.retry_receipt(&id, &pending.token).unwrap(),
            CleanupRetryDisposition::Requested
        );
        previous = pending.token;
    }
    let done = terminal(&handle, &id);
    assert!(done.cancellation_requested);
    assert_eq!(
        done.response.error.as_ref().unwrap().code,
        ErrorCode::EngineFailed
    );
    let details = &done.response.error.as_ref().unwrap().details["kinekino_process"];
    assert_eq!(details["physical_closed"], true);
    assert_eq!(details["receipt_recorded"], true);
    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(format!("receipt-{collisions:04}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["child_exit"], 0);
    assert_eq!(receipt["family"], "Empty");
    assert_eq!(receipt["forced"], false);
    assert_eq!(fs::read(root.join("receipt-0000.json")).unwrap(), SENTINEL);
    for index in 0..collisions {
        assert_eq!(
            fs::read(root.join(format!("receipt-{index:04}.json"))).unwrap(),
            SENTINEL
        );
    }
    assert!(!output.exists());
    drop(
        fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&media)
            .unwrap(),
    );
    assert_eq!(sha(&media), original_sha);
    fs::write(
        root.join("sdk-response.json"),
        serde_json::to_vec_pretty(done.response.as_ref()).unwrap(),
    )
    .unwrap();
    handle.acknowledge(&id).unwrap();
    let next = handle.submit(Request::new("version")).unwrap();
    assert!(terminal(&handle, &next).response.ok);
    handle.acknowledge(&next).unwrap();
    close(&handle);
}
