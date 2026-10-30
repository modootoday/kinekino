use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub struct Fixture {
    pub root: PathBuf,
    pub evidence: PathBuf,
    pub user: PathBuf,
    pub case: Value,
    pub media: Value,
    pub originals: Vec<(PathBuf, String)>,
}
impl Fixture {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "review-source-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&root).unwrap();
        let evidence = root.join("evidence");
        let user = root.join("user");
        fs::create_dir(&evidence).unwrap();
        fs::create_dir(&user).unwrap();
        for part in ["packages", "objects", "decisions"] {
            fs::create_dir(evidence.join(part)).unwrap();
        }
        let request = br#"{"path":"Z:/must-not-open/private.mp4"}"#;
        let video = b"synthetic declared media; not decoded or PTS verified";
        let request_sha = sha(request);
        let video_sha = sha(video);
        let mut originals = vec![];
        for (hash, bytes) in [
            (&request_sha, request.as_slice()),
            (&video_sha, video.as_slice()),
        ] {
            let path = evidence.join("objects").join(hash);
            fs::write(&path, bytes).unwrap();
            originals.push((path, hash.clone()));
        }
        let body=serde_json::to_vec(&json!({"schema_version":"1","package_id":"cli-fixture","items":[{
            "item_id":"item","task":"asr","source":{"asset_id":"request","sha256":request_sha,"bytes":request.len().to_string(),"media_type":"application/json"},"range":null,"baseline":null,
            "candidate":{"condition":{"model_id":"fixture","revision":"r1","precision":"fp32","engine_id":"fixture","engine_revision":"r1","input_conditions":{"sample":"synthetic"}},
                "assets":[{"asset_id":"video","sha256":video_sha,"bytes":video.len().to_string(),"media_type":"video/mp4"}],
                "automated_checks":[{"check_id":"quality","outcome":"not_run","summary":"Not approved","evidence":[]}],"measured_resources":{"kind":"synthetic"}},
            "criteria":[{"criterion_id":"compare","how_to_review":"Compare source","expected":"Faithful output"}],"known_issues":[]
        }]})).unwrap();
        let package_sha = sha(&body);
        let package = evidence
            .join("packages")
            .join(format!("{package_sha}.json"));
        fs::write(&package, body).unwrap();
        originals.push((package, package_sha.clone()));
        Self {
            root,
            evidence,
            user,
            case: json!({"package_sha256":package_sha,"item_id":"item","source_request_sha256":request_sha}),
            media: json!({"asset_id":"video","sha256":video_sha}),
            originals,
        }
    }
    pub fn assert_originals(&self) {
        for (path, hash) in &self.originals {
            assert_eq!(&sha(&fs::read(path).unwrap()), hash);
        }
        assert_eq!(
            fs::read_dir(self.evidence.join("decisions"))
                .unwrap()
                .count(),
            0
        );
        assert!(!self.evidence.join("cases").exists());
    }
    pub fn request(
        &self,
        command: &str,
        payload: Value,
        key: Option<&str>,
        revision: Option<&str>,
    ) -> Value {
        json!({"context":{"request_id":"actual-cli","api_version":"1","expected_revision":revision,"idempotency_key":key,"timeout_ms":null},"command":command,"payload":payload})
    }
}
