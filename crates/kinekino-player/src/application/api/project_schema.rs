use super::{ApiError, ErrorCode};
use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal u64; range checked"})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$","not":{"enum":[".",".."]}})
}
fn nullable(schema: Value) -> Value {
    json!({"anyOf":[schema,{"type":"null"}]})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn watching_source() -> Value {
    let local = object(
        json!({"kind":{"const":"local_video"},"source_id":id(),"path":{"type":"string","minLength":1,"maxLength":32768,"description":"Selected absolute local video locator; never opened by project storage"}}),
        &["kind", "source_id", "path"],
    );
    let episode = object(
        json!({"kind":{"const":"episode_page"},"source_id":id(),"episode_url":{"type":"string","minLength":1,"maxLength":8192,"description":"https catalog.example.test/e/... without credentials, query, fragment or nondefault port"}}),
        &["kind", "source_id", "episode_url"],
    );
    json!({"oneOf":[local,episode]})
}
fn watching_selection() -> Value {
    object(
        json!({"label":{"type":"string","minLength":1,"maxLength":1024},"source":watching_source()}),
        &["label", "source"],
    )
}
fn watching() -> Value {
    nullable(watching_selection())
}
fn working() -> Value {
    fn allow_incomplete(value: &mut Value) {
        match value {
            Value::Object(properties) => {
                properties.remove("minItems");
                properties.remove("minLength");
                for value in properties.values_mut() {
                    allow_incomplete(value);
                }
            }
            Value::Array(values) => {
                for value in values {
                    allow_incomplete(value);
                }
            }
            _ => {}
        }
    }
    let mut value = crate::application::proposals::schema::payload_type();
    allow_incomplete(&mut value);
    nullable(value)
}
fn content() -> Value {
    let live = object(
        json!({"origin":{"const":"live"},"proposal_id":id(),"draft_revision":decimal(),"working_draft":working()}),
        &["origin", "proposal_id", "draft_revision"],
    );
    let saved = object(
        json!({"origin":{"const":"saved"},"proposal_id":id(),"capture_sha256":sha(),"working_draft":working()}),
        &["origin", "proposal_id", "capture_sha256"],
    );
    object(
        json!({"watching":watching(),"proposals":{"type":"array","maxItems":32,"items":{"oneOf":[live,saved]}}}),
        &["proposals"],
    )
}
fn response() -> Value {
    let origin = object(
        json!({"schema_version":{"const":"proposal-origin/1"},"origin_id":{"type":"string","pattern":"^[a-f0-9]{32}$"}}),
        &["schema_version", "origin_id"],
    );
    let snapshot = object(
        json!({"capture_sha256":sha(),"captured":crate::application::proposals::schema::response_type("proposals.inspect"),"working_draft":working(),"working_state":{"enum":["absent","valid","needs_input"]},"proposal_origin":origin}),
        &[
            "capture_sha256",
            "captured",
            "working_draft",
            "working_state",
        ],
    );
    let counts = object(
        json!({"needs_repair":{"type":"integer","minimum":0},"editable":{"type":"integer","minimum":0},"accepted_output":{"type":"integer","minimum":0},"working_needs_input":{"type":"integer","minimum":0}}),
        &[
            "needs_repair",
            "editable",
            "accepted_output",
            "working_needs_input",
        ],
    );
    let mut project = object(
        json!({"schema_version":{"enum":["1","2"]},"project_id":id(),"revision":decimal(),"parent_revision":nullable(decimal()),"title":{"type":"string","minLength":1,"maxLength":1024},"watching":watching(),"proposals":{"type":"array","maxItems":32,"items":snapshot},"work_state":counts}),
        &[
            "schema_version",
            "project_id",
            "revision",
            "parent_revision",
            "title",
            "watching",
            "proposals",
            "work_state",
        ],
    );
    project["allOf"] = json!([{
        "if":{"properties":{"schema_version":{"const":"1"}}},
        "then":{"properties":{"proposals":{"items":{"not":{"required":["proposal_origin"]}}}}}
    }]);
    object(
        json!({"revision_sha256":sha(),"latest_revision":decimal(),"project":project}),
        &["revision_sha256", "latest_revision", "project"],
    )
}

pub(super) fn describe(input: &Value, available: bool) -> Result<Value, ApiError> {
    let command = input["command"]
        .as_str()
        .filter(|command| super::projects::is_command(command))
        .filter(|_| input.as_object().is_some_and(|v| v.len() == 1))
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "schema requires one project command",
            )
        })?;
    let payload = if command == "projects.inspect" {
        object(
            json!({"project_id":id(),"revision":nullable(decimal())}),
            &["project_id"],
        )
    } else if command == "projects.reconnect_source" {
        object(
            json!({"project_id":id(),"expected_source":watching_source(),"replacement":watching_selection()}),
            &["project_id", "expected_source", "replacement"],
        )
    } else {
        object(
            json!({"project_id":id(),"title":{"type":"string","minLength":1,"maxLength":1024},"content":content()}),
            &["project_id", "title", "content"],
        )
    };
    let mut context = super::schema::context_schema();
    context["properties"]["timeout_ms"] = json!({"type":"null"});
    context["properties"]["idempotency_key"] = json!({"type":"null"});
    let requires_revision = matches!(
        command,
        "projects.save_revision" | "projects.reconnect_source"
    );
    context["properties"]["expected_revision"] = if requires_revision {
        decimal()
    } else {
        json!({"type":"null"})
    };
    if requires_revision {
        context["required"] = json!(["request_id", "api_version", "expected_revision"]);
    }
    let response = || {
        if command == "projects.reconnect_source" {
            object(
                json!({"state":response(),"scope":{"const":"watching_locator_only"},"content_identity_verified":{"const":false},"source_opened":{"const":false},"playback_rebound":{"const":false}}),
                &[
                    "state",
                    "scope",
                    "content_identity_verified",
                    "source_opened",
                    "playback_rebound",
                ],
            )
        } else {
            response()
        }
    };
    Ok(
        json!({"schema_version":"1","command":command,"status":if available {"supported"} else {"unsupported"},"request":object(json!({"context":context,"command":{"const":command},"payload":payload}), &["context","command","payload"]),"response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":nullable(response()),"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),"error_codes":ErrorCode::ALL,"constraints":{"nonempty_context":true,"create_origins":["live"],"save_origins":["live","saved"],"record_max_bytes":crate::domain::projects::MAX_RECORD_BYTES,"working_total_max_bytes":crate::domain::projects::MAX_WORKING_BYTES,"saved_capture_requires_expected_project_match":true,"working_draft_may_need_input":true},"project_current_revision_verified":false,"media_binding_verified":false,"automatic_application_enabled":false}),
    )
}
pub(super) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"]
                .as_str()
                .is_some_and(super::projects::is_command)
            {
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["projects"] = json!({"backend_available":available,"schema_version":"1","immutable_revisions":true,"nonempty_context":true,"proposal_backend_required_for_live_capture":true,"saved_capture_independent":true,"record_max_bytes":crate::domain::projects::MAX_RECORD_BYTES,"working_total_max_bytes":crate::domain::projects::MAX_WORKING_BYTES,"project_current_revision_verified":false,"media_binding_verified":false,"automatic_application_enabled":false});
    value["projects"]["supported_record_schema_versions"] = json!(["1", "2"]);
    value["projects"]["reconnection"] = json!({"available":available,"scope":"watching_locator_only","content_identity_verified":false,"source_opened":false,"playback_rebound":false,"in_flight_cancel":false});
    value["projects"]["capture_origin"] = json!({
        "producer":"proposal_backend",
        "legacy_absent_origin_read_only":true,
        "origin_alone_grants_mutation":false,
        "saved_origin_preserved":true
    });
}

#[cfg(test)]
mod origin_schema_tests {
    use super::*;

    #[test]
    fn origin_is_response_only_and_v1_cannot_contain_it() {
        let schema = describe(&json!({"command":"projects.create"}), true).unwrap();
        let request = &schema["request"]["properties"]["payload"]["properties"]["content"];
        let live = &request["properties"]["proposals"]["items"]["oneOf"][0];
        assert_eq!(live["additionalProperties"], false);
        assert!(live["properties"].get("proposal_origin").is_none());
        let response = response();
        let project = &response["properties"]["project"];
        assert_eq!(
            project["properties"]["schema_version"]["enum"],
            json!(["1", "2"])
        );
        assert_eq!(
            project["allOf"][0]["then"]["properties"]["proposals"]["items"]["not"]["required"],
            json!(["proposal_origin"])
        );
        let snapshot = &project["properties"]["proposals"]["items"];
        assert_eq!(
            snapshot["properties"]["proposal_origin"]["additionalProperties"],
            false
        );
        assert!(
            !snapshot["required"]
                .as_array()
                .unwrap()
                .contains(&json!("proposal_origin"))
        );
    }
}
