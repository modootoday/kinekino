use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn string(max: usize) -> Value {
    json!({"type":"string","minLength":1,"maxLength":max,"description":format!("Nonblank text; UTF-8 byte budget {max} and control characters are checked by application")})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$","description":"Single stable identifier, excluding dot and dot-dot; never a path"})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[0-9a-f]{64}$"})
}
fn array(items: Value, max: usize) -> Value {
    json!({"type":"array","items":items,"maxItems":max})
}
fn fields() -> Value {
    json!({"type":"object","minProperties":1,"maxProperties":32,"propertyNames":id(),"additionalProperties":string(1024)})
}
pub fn range() -> Value {
    object(
        json!({"start_us":{"type":"string","pattern":"^[0-9]+$"},"end_us":{"type":"string","pattern":"^[0-9]+$"}}),
        &["start_us", "end_us"],
    )
}
fn asset() -> Value {
    object(
        json!({"asset_id":id(),"sha256":sha(),"bytes":{"type":"string","pattern":"^[0-9]+$","description":"1..1073741824 bytes; application range validation"},"media_type":string(128)}),
        &["asset_id", "sha256", "bytes", "media_type"],
    )
}
fn condition() -> Value {
    object(
        json!({"model_id":string(256),"revision":string(256),"precision":string(64),"engine_id":string(256),"engine_revision":string(256),"input_conditions":fields()}),
        &[
            "model_id",
            "revision",
            "precision",
            "engine_id",
            "engine_revision",
            "input_conditions",
        ],
    )
}
fn result() -> Value {
    let check = object(
        json!({"check_id":id(),"outcome":{"enum":["passed","failed","not_run"]},"summary":string(4096),"evidence":array(asset(),8)}),
        &["check_id", "outcome", "summary", "evidence"],
    );
    object(
        json!({"condition":condition(),"assets":{"type":"array","minItems":1,"maxItems":16,"items":asset()},"automated_checks":{"type":"array","minItems":1,"maxItems":64,"items":check},"measured_resources":fields()}),
        &[
            "condition",
            "assets",
            "automated_checks",
            "measured_resources",
        ],
    )
}
pub fn package() -> Value {
    let criterion = object(
        json!({"criterion_id":id(),"how_to_review":string(4096),"expected":string(4096)}),
        &["criterion_id", "how_to_review", "expected"],
    );
    let item = object(
        json!({"item_id":id(),"task":id(),"source":asset(),"range":{"anyOf":[range(),{"type":"null"}],"description":"Null/omitted means explicitly non-temporal; issue_ranges must be empty"},"baseline":{"anyOf":[result(),{"type":"null"}]},"candidate":result(),"criteria":{"type":"array","minItems":1,"maxItems":32,"items":criterion},"known_issues":array(string(4096),64)}),
        &[
            "item_id",
            "task",
            "source",
            "candidate",
            "criteria",
            "known_issues",
        ],
    );
    object(
        json!({"schema_version":{"const":"1"},"package_id":id(),"items":{"type":"array","minItems":1,"maxItems":64,"items":item}}),
        &["schema_version", "package_id", "items"],
    )
}
pub fn payload(command: &str) -> Value {
    match command {
        "reviews.read_text_asset" => object(
            json!({"package_sha256":sha(),"asset_sha256":sha()}),
            &["package_sha256", "asset_sha256"],
        ),
        "reviews.record_decision" => object(
            json!({"package_sha256":sha(),"item_id":id(),"decision_id":id(),"verdict":{"enum":["usable","needs_changes","deferred"]},"note":string(8192),"issue_ranges":array(range(),64)}),
            &[
                "package_sha256",
                "item_id",
                "decision_id",
                "verdict",
                "note",
                "issue_ranges",
            ],
        ),
        "reviews.decisions" => object(
            json!({"package_sha256":sha(),"item_id":id()}),
            &["package_sha256", "item_id"],
        ),
        _ => object(json!({"package_sha256":sha()}), &["package_sha256"]),
    }
}
pub fn example(command: &str) -> Value {
    let sha = "0".repeat(64);
    match command {
        "reviews.read_text_asset" => json!({"package_sha256":sha,"asset_sha256":"1".repeat(64)}),
        "reviews.record_decision" => {
            json!({"package_sha256":sha,"item_id":"sample-001","decision_id":"review-001","verdict":"deferred","note":"Compare the fixed source and baseline before judging.","issue_ranges":[]})
        }
        "reviews.decisions" => json!({"package_sha256":sha,"item_id":"sample-001"}),
        _ => json!({"package_sha256":sha}),
    }
}
pub fn response(command: &str) -> Value {
    let decision = object(
        json!({"schema_version":{"const":"1"},"package_sha256":sha(),"package_id":id(),"task":id(),"item_id":id(),"decision_id":id(),"candidate_condition":condition(),"verdict":{"enum":["usable","needs_changes","deferred"]},"note":string(8192),"issue_ranges":array(range(),64),"review_ready":{"type":"boolean"},"automated_prerequisites_passed":{"type":"boolean"},"sample_review_passed":{"type":"boolean"},"automatic_application_enabled":{"const":false},"scope":{"const":"sample_only"}}),
        &[
            "schema_version",
            "package_sha256",
            "package_id",
            "task",
            "item_id",
            "decision_id",
            "candidate_condition",
            "verdict",
            "note",
            "issue_ranges",
            "review_ready",
            "automated_prerequisites_passed",
            "sample_review_passed",
            "automatic_application_enabled",
            "scope",
        ],
    );
    let stored = object(
        json!({"decision_sha256":sha(),"decision":decision}),
        &["decision_sha256", "decision"],
    );
    match command {
        "reviews.read_text_asset" => object(
            json!({"package_sha256":sha(),"asset_sha256":sha(),"bytes":{"type":"string","pattern":"^[0-9]+$"},"media_type":string(128),"utf8":{"type":"string","maxLength":65536}}),
            &[
                "package_sha256",
                "asset_sha256",
                "bytes",
                "media_type",
                "utf8",
            ],
        ),
        "reviews.record_decision" => stored,
        "reviews.decisions" => array(stored, 32),
        _ => object(
            json!({"package_sha256":sha(),"package":package(),"evidence_verified":{"const":true},"review_ready":{"type":"boolean"},"automatic_application_enabled":{"const":false}}),
            &[
                "package_sha256",
                "package",
                "evidence_verified",
                "review_ready",
                "automatic_application_enabled",
            ],
        ),
    }
}
