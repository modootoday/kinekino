use super::*;
use kinekino_player::application::{
    sdk_execution::{
        CancelDisposition, CompletionNotifier, ExecutionBudget, ExecutionHandle, ExecutionOwner,
        ExecutionSnapshot, ExecutionTerminal, OperationId,
    },
    speaker_annotations::{SpeakerDocumentFiles, SpeakerInput},
};
use std::sync::{Arc, Condvar, Mutex, mpsc};

#[derive(Clone, Copy)]
enum Point {
    BeforePublication,
    AfterPublication,
}
struct Gate {
    root: PathBuf,
    input: PathBuf,
    point: Point,
    entered: mpsc::Sender<()>,
    release: (Mutex<bool>, Condvar),
}
impl Gate {
    fn pause(&self) -> Result<(), ApiError> {
        assert!(
            fs::OpenOptions::new()
                .write(true)
                .open(&self.input)
                .is_err(),
            "original source lease must remain held"
        );
        self.entered.send(()).unwrap();
        let (released, timeout) = self
            .release
            .1
            .wait_timeout_while(
                self.release.0.lock().unwrap(),
                Duration::from_secs(10),
                |r| !*r,
            )
            .unwrap();
        if timeout.timed_out() && !*released {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "test release expired",
            ));
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
struct Checkpoint<'a> {
    original: &'a dyn MutationControl,
    gate: &'a Gate,
}
impl MutationControl for Checkpoint<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if matches!(self.gate.point, Point::BeforePublication)
            && fs::read_dir(&self.gate.root).unwrap().any(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".export-")
            })
        {
            self.gate.pause()?;
        }
        self.original.checkpoint()
    }
}
struct Files(Arc<Gate>);
impl SpeakerDocumentFiles for Files {
    fn retain(
        &self,
        path: &str,
        control: &dyn MutationControl,
    ) -> Result<Box<dyn SpeakerInput>, ApiError> {
        LocalSpeakerFiles.retain(path, control)
    }
    fn publish_new(
        &self,
        path: &str,
        bytes: &[u8],
        input: &dyn SpeakerInput,
        control: &dyn MutationControl,
    ) -> Result<(), ApiError> {
        LocalSpeakerFiles.publish_new(
            path,
            bytes,
            input,
            &Checkpoint {
                original: control,
                gate: &self.0,
            },
        )?;
        if matches!(self.0.point, Point::AfterPublication) {
            self.0.pause()?;
        }
        // A successful atomic publication must not acquire a fallible post-commit checkpoint.
        Ok(())
    }
}
struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn terminal(handle: &ExecutionHandle, id: &OperationId) -> Arc<ExecutionTerminal> {
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        if let ExecutionSnapshot::Terminal(t) = handle.peek(id).unwrap() {
            return t;
        }
        assert!(
            Instant::now() < until,
            "original execution terminal deadline"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn run(point: Point) {
    let root = root();
    let input = root.join("source.json");
    let bytes = serde_json::to_vec(&fixture()).unwrap();
    fs::write(&input, &bytes).unwrap();
    let output = root.join("target.json");
    let (entered, receiver) = mpsc::channel();
    let gate = Arc::new(Gate {
        root: root.clone(),
        input: input.clone(),
        point,
        entered,
        release: (Mutex::new(false), Condvar::new()),
    });
    let release = Release(gate.clone());
    let client = Arc::new(PlayerSdk::default().with_speaker_documents(Files(gate)));
    let owner = ExecutionOwner::start(
        client,
        ExecutionBudget {
            max_operations: 2,
            max_request_bytes: 8192,
            max_total_request_bytes: 16384,
            max_terminal_bytes: 65536,
            max_total_terminal_bytes: 131072,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let handle = owner.handle();
    let request = edit(
        &input,
        &output,
        1,
        json!([{"op":"rename","speaker_id":"s1","label":"explicit user edit"}]),
    );
    let id = handle.submit(request.clone()).unwrap();
    receiver.recv_timeout(Duration::from_secs(10)).unwrap();
    let published_before_cancel = output.exists();
    assert_eq!(
        published_before_cancel,
        matches!(point, Point::AfterPublication)
    );
    let published_hash = published_before_cancel.then(|| sha(&fs::read(&output).unwrap()));
    assert_eq!(handle.cancel(&id).unwrap(), CancelDisposition::Requested);
    drop(release);
    let result = terminal(&handle, &id);
    assert!(result.cancellation_requested);
    assert!(!result.queued_cancelled);
    match point {
        Point::BeforePublication => {
            assert_eq!(
                result.response.error.as_ref().unwrap().code,
                ErrorCode::Cancelled
            );
            assert!(!output.exists());
            assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        }
        Point::AfterPublication => {
            assert!(result.response.ok);
            assert_eq!(
                result.response.data.as_ref().unwrap()["document"]["revision"],
                "2"
            );
            assert_eq!(Some(sha(&fs::read(&output).unwrap())), published_hash);
            assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        }
    }
    assert_eq!(fs::read(&input).unwrap(), bytes);
    assert!(fs::OpenOptions::new().write(true).open(&input).is_ok());
    handle.acknowledge(&id).unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        if owner.close().is_ok() {
            break;
        }
        assert!(Instant::now() < until, "original worker close deadline");
        std::thread::sleep(Duration::from_millis(5));
    }
    fs::write(root.join("EXECUTION.json"),serde_json::to_vec_pretty(&json!({"request":request,"response":result.response.as_ref(),"cancellation_requested":result.cancellation_requested,"queued_cancelled":result.queued_cancelled,"published_before_cancel":published_before_cancel,"input_sha256":sha(&bytes),"output_exists":output.exists(),"worker_closed":true})).unwrap()).unwrap();
    println!("speaker execution evidence {}", root.display());
}
#[test]
fn original_execution_cancel_before_publish_removes_temporary_and_preserves_input() {
    kinekino_platform::install();
    run(Point::BeforePublication);
}
#[test]
fn original_execution_cancel_after_publish_retains_real_success_and_immutable_output() {
    kinekino_platform::install();
    run(Point::AfterPublication);
}
