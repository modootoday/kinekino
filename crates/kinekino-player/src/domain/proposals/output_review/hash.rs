use super::super::ProposalPayload;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const OUTPUT_HASH_FORMAT: &str = "accepted_typed_output_v1";

/// Hashes typed output, never rendered media, a project capture or reviewer metadata.
pub fn accepted_output_sha256(output: &ProposalPayload) -> Result<String, &'static str> {
    canonical_sha256(&serde_json::json!({"hash_format": OUTPUT_HASH_FORMAT, "output": output}))
}
pub(super) fn raw_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn canonical_sha256(value: &impl Serialize) -> Result<String, &'static str> {
    let value =
        serde_json::to_value(value).map_err(|_| "Cannot serialize output review identity")?;
    let mut bytes = Vec::new();
    encode(&value, &mut bytes)?;
    Ok(raw_sha256(&bytes))
}
fn encode(value: &Value, bytes: &mut Vec<u8>) -> Result<(), &'static str> {
    match value {
        Value::Object(object) => {
            bytes.push(b'{');
            let mut keys: Vec<_> = object.keys().collect();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    bytes.push(b',');
                }
                bytes.extend(serde_json::to_vec(key).map_err(|_| "Cannot encode object key")?);
                bytes.push(b':');
                encode(&object[key], bytes)?;
            }
            bytes.push(b'}');
        }
        Value::Array(array) => {
            bytes.push(b'[');
            for (index, item) in array.iter().enumerate() {
                if index > 0 {
                    bytes.push(b',');
                }
                encode(item, bytes)?;
            }
            bytes.push(b']');
        }
        Value::Number(_) => return Err("JSON numbers require a new output review hash version"),
        _ => bytes
            .extend(serde_json::to_vec(value).map_err(|_| "Cannot encode output review value")?),
    }
    Ok(())
}
