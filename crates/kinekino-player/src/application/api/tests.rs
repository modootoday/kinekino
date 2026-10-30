use super::*;

#[test]
fn context_unicode_lengths_and_safe_error_correlation_are_shared() {
    let sdk = PlayerSdk::default();
    for id in ["가".repeat(128), "😀".repeat(128), "a\u{0301}".repeat(64)] {
        assert!(id.len() > 128);
        let mut request = Request::new("version");
        request.context.request_id = id.clone();
        request.context.idempotency_key = Some("키".repeat(128));
        assert!(request.validate().is_ok());
        let response = sdk.execute(request);
        assert!(response.ok);
        assert_eq!(response.request_id, id);
    }
    for id in [
        String::new(),
        "가".repeat(129),
        "\nTEST_SECRET_ID".into(),
        "TEST_SECRET_ID\n".into(),
        "\u{0085}TEST_SECRET_ID".into(),
    ] {
        let mut request = Request::new("version");
        request.context.request_id = id.clone();
        let response = sdk.execute(request);
        assert_eq!(response.request_id, "cli");
        assert_eq!(response.error.as_ref().unwrap().request_id, "cli");
        assert!(
            !serde_json::to_string(&response)
                .unwrap()
                .contains("TEST_SECRET_ID")
        );
    }
    let mut request = Request::new("version");
    request.context.request_id = "요청-상관".into();
    request.context.api_version = "invalid".into();
    let response = sdk.execute(request);
    assert_eq!(response.request_id, "요청-상관");
    assert_eq!(response.error.unwrap().request_id, "요청-상관");
    let mut request = Request::new("version");
    request.context.idempotency_key = Some("키".repeat(129));
    assert!(request.validate().is_err());
}

#[test]
fn schema_catalog_and_unicode_contract_come_from_shared_definitions() {
    let sdk = PlayerSdk::default();
    let mut request = Request::new("schema");
    request.payload = serde_json::json!({"command":"version"});
    let schema = sdk.execute(request).data.unwrap();
    let catalog = serde_json::to_value(ErrorCode::ALL).unwrap();
    assert_eq!(schema["error_codes"], catalog);
    let codes = schema["error_codes"].as_array().unwrap();
    for code in ["source_expired", "cancelled", "internal", "timeout"] {
        assert!(codes.contains(&serde_json::json!(code)));
    }
    assert_eq!(codes.len(), 12);
    assert_eq!(
        schema["request"]["properties"]["context"]["properties"]["request_id"]["maxLength"],
        128
    );
    assert_eq!(
        schema["request"]["properties"]["context"]["properties"]["request_id"]["not"]["pattern"],
        commands::REQUEST_ID_CONTROL_PATTERN
    );
}

#[test]
fn unavailable_commands_preserve_malformed_request_validation() {
    let mut observations = Vec::new();
    let mut failures = Vec::new();
    for command in capabilities::COMMANDS
        .iter()
        .filter(|c| !matches!(**c, "version" | "capabilities" | "schema"))
    {
        let response = PlayerSdk::new(crate::application::subtitles::storage::UnavailableStorage)
            .execute(Request::new(*command));
        // Valid typed requests are covered by sdk_capability_parity and snapshot_export_api_cli.
        let expected = match *command {
            "proposals.inspect_output_review_target"
            | "proposals.record_output_decision"
            | "proposals.output_decisions"
            // Export preflight strictly decodes shape before host availability.
            | "segments.skip_export"
            | "segments.export_list"
            | "segments.export_peek"
            | "segments.export_consume"
            | "production.export_snapshot"
            | crate::application::proposals::subtitle_export::COMMAND
            | crate::application::proposals::card_export::COMMAND
            // Project/asset/review-media commands also decode strict shape before backend availability.
            | "asset_artifacts.inspect"
            | "asset_projects.create"
            | "asset_projects.inspect"
            | "asset_projects.save_revision"
            | "clip_projects.create"
            | "clip_projects.inspect"
            | "clip_projects.save_revision"
            | "clip_sources.inspect"
            | "card_projects.create"
            | "card_projects.inspect"
            | "card_projects.save_revision"
            | "reviews.media.prepare"
            | "reviews.media.inspect" => ErrorCode::InvalidArgument,
            _ => ErrorCode::Unsupported,
        };
        let actual = response.error.as_ref().map(|error| error.code);
        let data_none = response.data.is_none();
        let observation = format!(
            "{command}: expected={expected:?}, actual={actual:?}, ok={}, data_none={data_none}",
            response.ok
        );
        if response.ok || !data_none || actual != Some(expected) {
            failures.push(observation.clone());
        }
        observations.push(observation);
    }
    println!("unavailable dispatch matrix:\n{}", observations.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn discovery_is_shared_serializable_and_version_checked() {
    let request = Request::new("version");
    let decoded = serde_json::from_value(serde_json::to_value(&request).unwrap()).unwrap();
    let first = serde_json::to_value(
        PlayerSdk::new(crate::application::subtitles::storage::UnavailableStorage)
            .execute(request.clone()),
    )
    .unwrap();
    assert_eq!(
        first,
        serde_json::to_value(
            PlayerSdk::new(crate::application::subtitles::storage::UnavailableStorage)
                .execute(decoded)
        )
        .unwrap()
    );
    assert_eq!(first["data"]["product"]["gui_executable"], "player.exe");
    let mut invalid = request;
    invalid.context.api_version = "2".into();
    assert_eq!(
        PlayerSdk::new(crate::application::subtitles::storage::UnavailableStorage)
            .execute(invalid)
            .error
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn schema_and_unknown_payload_are_validated() {
    let mut request = Request::new("schema");
    request.payload = serde_json::json!({"command":"version"});
    assert!(
        PlayerSdk::new(crate::application::subtitles::storage::UnavailableStorage)
            .execute(request)
            .ok
    );
    let mut request = Request::new("version");
    request.payload = serde_json::json!({"token":"secret"});
    let result = serde_json::to_string(
        &PlayerSdk::new(crate::application::subtitles::storage::UnavailableStorage)
            .execute(request),
    )
    .unwrap();
    assert!(!result.contains("secret"));
    assert!(result.contains("invalid_argument"));
}
