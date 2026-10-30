use serde_json::{Value, json};

impl super::CardBundleRequest {
    /// Discovery contract for the explicit review-only bundle executor.
    /// File identities, saved publications and native paths are checked at admission.
    pub fn schema() -> Value {
        let identifier = json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$","not":{"enum":[".",".."]}});
        let sha = json!({"type":"string","pattern":"^[a-f0-9]{64}$"});
        let path = json!({"type":"string","description":"Absolute local file-system path; native path and identity checks apply at admission"});
        json!({
            "type":"object","additionalProperties":false,
            "required":["project","frames","output_directory","review_artifact_only"],
            "properties":{
                "project":{"type":"object","additionalProperties":false,
                    "required":["document_id","publication_revision","publication_sha256"],
                    "properties":{"document_id":identifier,"publication_revision":{"type":"string","pattern":"^[0-9]+$","description":"Decimal u64 string; range checked and must be greater than zero"},"publication_sha256":sha}},
                "frames":{"type":"array","items":{"type":"object","additionalProperties":false,
                    "required":["artifact_id","path","png_sha256"],
                    "properties":{"artifact_id":identifier,"path":path,"png_sha256":sha}},
                    "description":"Exactly the selected saved document frame references; duplicate, foreign and unused inputs are rejected"},
                "output_directory":path,
                "review_artifact_only":{"const":true}
            },
            "description":"Exact immutable saved card publication and explicit PNGs. Current proposed template only; output is an unapproved review artifact. New output directory required; publication never overwrites existing output. Parameter JSON is limited to 64 KiB by Jobs."
        })
    }
}
