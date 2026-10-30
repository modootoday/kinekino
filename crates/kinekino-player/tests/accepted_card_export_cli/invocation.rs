use super::owned_process_evidence;
use kinekino_player::application::{
    api::{Request, RequestContext},
    proposals::card_export::{COMMAND, ExportCardReviewPng},
};
use serde_json::{Value, json};
use std::{fs, path::Path, time::Duration};

pub(super) fn invoke(
    executable: &Path,
    root: &Path,
    store: &Path,
    label: &str,
    input: ExportCardReviewPng,
    mut context: RequestContext,
    expected_exit: i32,
) -> Value {
    context.request_id = label.into();
    let mut request = Request::new(COMMAND);
    request.context = context;
    request.payload = serde_json::to_value(input).unwrap();
    let input_path = root.join(format!("{label}.request.json"));
    fs::write(&input_path, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    let call_root = root.join(label);
    fs::create_dir(&call_root).unwrap();
    let result = owned_process_evidence::run_process_with_arguments(
        executable.to_path_buf(),
        &call_root,
        &[],
        vec![
            "--input".into(),
            input_path.into_os_string(),
            "--proposal-root".into(),
            store.as_os_str().into(),
            "--json".into(),
        ],
        Duration::from_secs(15),
        false,
    );
    let output = match result {
        Ok(output) => output,
        Err(failure) => {
            // Preserve original ownership before fallible diagnostics/unwind; never renew or finish again.
            let retained = Box::leak(failure);
            let diagnostic = owned_process_evidence::write_json(
                &call_root.join("failure.json"),
                &retained.observation(),
            );
            panic!("original CLI failure retained: {retained:?}; diagnostic={diagnostic:?}");
        }
    };
    fs::write(root.join(format!("{label}.stdout.json")), &output.stdout).unwrap();
    fs::write(root.join(format!("{label}.stderr.raw")), &output.stderr).unwrap();
    fs::write(root.join(format!("{label}.status.json")),serde_json::to_vec(&json!({"receipt":owned_process_evidence::receipt_json(&output.receipt),"elapsed_ms":output.elapsed_ms,"stdout":output.stdout_evidence,"stderr":output.stderr_evidence})).unwrap()).unwrap();
    assert_eq!(
        output.receipt.child_exit,
        Some(expected_exit as u32),
        "{label}: original response preserved"
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["request_id"], label);
    response
}
