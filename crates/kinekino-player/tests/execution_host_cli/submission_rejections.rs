//! Actual normal CLI denial paths, preserving each request and response.
use super::*;
use std::path::Path;
pub(super) fn check(root: &Path, endpoint: &str, submission: &Value, inner: &Request) {
    let reject = |label: &str, command: &str, payload: Value, expected: &str| {
        let directory = root.join(label);
        fs::create_dir(&directory).unwrap();
        let mut request = Request::new(command);
        request.payload = payload;
        fs::write(
            directory.join("request.json"),
            serde_json::to_vec_pretty(&request).unwrap(),
        )
        .unwrap();
        let mut runner = cli::CliRunner::start(endpoint, &request);
        let receipt = runner.finish_preserving(&directory);
        assert_ne!(receipt.code, 0, "{label}: {receipt:?}");
        assert_eq!(receipt.response["ok"], false);
        assert_eq!(
            receipt.response["error"]["code"], expected,
            "{label}: {:?}",
            receipt.response
        );
    };
    let mut different = inner.clone();
    different.command = "capabilities".into();
    reject(
        "id-request-conflict",
        "execution.submit",
        json!({"submission":submission,"request":different}),
        "revision_conflict",
    );
    let mut foreign = submission.clone();
    foreign["host_scope"] = json!(if submission["host_scope"] == "0".repeat(32) {
        "1".repeat(32)
    } else {
        "0".repeat(32)
    });
    reject(
        "foreign-id-inspect",
        "execution.submission.inspect",
        json!({"submission":foreign}),
        "source_expired",
    );
    reject(
        "foreign-id-submit",
        "execution.submit",
        json!({"submission":foreign,"request":inner}),
        "source_expired",
    );
    let mut unknown = submission.clone();
    unknown["submission_id"] = json!("not-admitted");
    reject(
        "unknown-id",
        "execution.submission.inspect",
        json!({"submission":unknown}),
        "not_found",
    );
    let mut empty = submission.clone();
    empty["submission_id"] = json!("");
    reject(
        "empty-id",
        "execution.submit",
        json!({"submission":empty,"request":inner}),
        "invalid_argument",
    );
    let mut control = submission.clone();
    control["submission_id"] = json!("invalid\nidentity");
    reject(
        "control-character-id",
        "execution.submit",
        json!({"submission":control,"request":inner}),
        "invalid_argument",
    );
    let mut extra = submission.clone();
    extra["pid"] = json!(123);
    reject(
        "identity-extra-field",
        "execution.submit",
        json!({"submission":extra,"request":inner}),
        "invalid_argument",
    );
    reject(
        "inspect-extra-field",
        "execution.submission.inspect",
        json!({"submission":submission,"path":"not-authority"}),
        "invalid_argument",
    );
    reject(
        "nested-execution",
        "execution.submit",
        json!({"submission":submission,"request":Request::new("execution.list")}),
        "invalid_argument",
    );
}
