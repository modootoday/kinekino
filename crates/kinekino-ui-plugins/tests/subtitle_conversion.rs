use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_subtitles::application::{convert, storage::SubtitleStorage};
use kinekino_ui_plugins::application::subtitle_conversion::{
    SubtitleConversionAdapter, unavailable_conversion,
};
use serde_json::{Value, json};
use std::cell::RefCell;

struct MemoryFiles {
    output: RefCell<Option<(String, String)>>,
}

impl SubtitleStorage for MemoryFiles {
    fn read_utf8(&self, path: &str) -> Result<String, ApiError> {
        assert_eq!(path, "source.vtt");
        Ok("WEBVTT\n\n00:00.000 --> 00:01.000\n<v Speaker>こんにちは\n".into())
    }

    fn publish_new(&self, path: &str, text: &str) -> Result<(), ApiError> {
        assert!(self.output.borrow().is_none());
        *self.output.borrow_mut() = Some((path.into(), text.into()));
        Ok(())
    }
}

fn request() -> Value {
    json!({"input_path":"source.vtt","output_path":"new.srt",
        "input_format":"vtt","output_format":"srt",
        "speaker_policy":"omit","allow_loss":false})
}

#[test]
fn canonical_use_case_retains_loss_consent_and_new_output_semantics() {
    let storage = MemoryFiles {
        output: RefCell::new(None),
    };
    // Contract harness only. Production host injects ApplicationClient, not storage.
    let adapter = SubtitleConversionAdapter::new(|request| {
        let result = convert::execute(&storage, &serde_json::to_value(request).unwrap())?;
        Ok(serde_json::from_value(result).unwrap())
    });
    let error = adapter.dispatch(request()).unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert!(
        error.details["losses"]
            .as_array()
            .is_some_and(|v| !v.is_empty())
    );
    assert!(storage.output.borrow().is_none());

    let mut accepted = request();
    accepted["allow_loss"] = json!(true);
    let result = adapter.dispatch(accepted).unwrap();
    assert_eq!(result["output_path"], "new.srt");
    assert_eq!(result["overwrite"], false);
    assert_eq!(result["cue_count"], 1);
    let output = storage.output.borrow();
    let (path, text) = output.as_ref().unwrap();
    assert_eq!(path, "new.srt");
    assert!(text.contains("こんにちは"));
}

#[test]
fn sdk_error_is_preserved_including_context_and_remediation() {
    let adapter = SubtitleConversionAdapter::new(|_| {
        let mut error = ApiError::new(ErrorCode::RevisionConflict, "Output already exists");
        error.details = json!({"resource":"new.srt"});
        error.request_id = "host-request-17".into();
        error.remediation = "Choose a new output path".into();
        error.retryable = false;
        Err(error)
    });
    let error = adapter.dispatch(request()).unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert_eq!(error.request_id, "host-request-17");
    assert_eq!(error.details, json!({"resource":"new.srt"}));
    assert_eq!(error.remediation, "Choose a new output path");
    assert!(!error.retryable);
}

#[test]
fn invalid_payload_never_reaches_sdk_and_missing_transport_is_unsupported() {
    let adapter = SubtitleConversionAdapter::new(|_| panic!("Invalid request reached SDK"));
    let mut invalid = request();
    invalid["arbitrary_command"] = json!("execute");
    assert_eq!(
        adapter.dispatch(invalid).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let unavailable = SubtitleConversionAdapter::new(unavailable_conversion);
    assert_eq!(
        unavailable.dispatch(request()).unwrap_err().code,
        ErrorCode::Unsupported
    );
}
