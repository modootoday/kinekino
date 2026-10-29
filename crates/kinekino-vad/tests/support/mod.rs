use kinekino_media_contracts::api_values::DecimalU64;
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad::protocol::{application::decode_descriptor, domain::ExpectedWorkerContext};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub fn pin(id: &str, bytes: &[u8]) -> ArtifactPin {
    ArtifactPin::new(
        id.into(),
        format!("{:x}", Sha256::digest(bytes)),
        DecimalU64(bytes.len() as u64),
    )
    .unwrap()
}
fn wire_pin(id: &str, bytes: &[u8]) -> Value {
    let p = pin(id, bytes);
    json!({"id":p.id(),"sha256":p.sha256(),"bytes":p.bytes().to_string()})
}
pub fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
pub fn descriptor() -> Value {
    json!({
        "schema_version":"1",
        "profile": {
            "id":"fixture-only","repo":"onnx-community/silero-vad","revision":"1".repeat(40),
            "precision":"fp32","provider":"CPUExecutionProvider","intra_threads":2,"inter_threads":1,
            "license_id":"MIT","weights":wire_pin("model",b"synthetic-model"),
            "license_document":wire_pin("license",b"synthetic-license"),"runtime":wire_pin("runtime",b"synthetic-runtime")
        },
        "input": {
            "wav":wire_pin("wav",b"descriptor-only-not-a-real-WAV"),"pcm":wire_pin("pcm",&[0;1026]),
            "sample_rate":16000,"channels":1,"bits_per_sample":16,"samples":"513","start_sample":"0","end_sample":"513"
        }
    })
}
pub fn expected(descriptor: &Value) -> ExpectedWorkerContext {
    let raw = bytes(descriptor);
    let decoded = decode_descriptor(raw.as_slice(), &pin("descriptor", &raw)).unwrap();
    ExpectedWorkerContext::new(
        pin("request", b"synthetic original request, not a job identity"),
        decoded,
    )
}
pub fn complete(descriptor: &Value, expected: &ExpectedWorkerContext) -> Value {
    json!({"schema_version":"1","request_sha256":expected.request().sha256(),
        "descriptor_sha256":expected.descriptor().raw().sha256(),"context":descriptor,
        "kind":"complete","processed_samples":"513","scores":[0.1,0.9],"failure":null,"actual_runtime":null})
}
pub fn module(name: &str) -> Value {
    json!({"name":name,"sha256":"a".repeat(64),"bytes":"4","canonical_path":null})
}
