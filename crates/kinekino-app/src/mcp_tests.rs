use super::*;

/// Fake API: `version` is read-only, `files.write` writes.
fn fake(request: Value) -> Value {
    let command = request["command"].as_str().unwrap_or_default();
    let data = match command {
        "capabilities" => json!({"commands": [
            {"command": "version", "status": "supported"},
            {"command": "files.write", "status": "supported"},
            {"command": "jobs.list", "status": "unsupported"},
        ]}),
        "schema" => json!({"request": {"properties": {"payload": {
            "type": "object", "additionalProperties": false, "properties": {}
        }}}}),
        _ => json!({"echo": request["payload"]}),
    };
    json!({"ok": true, "data": data, "request_id": request["context"]["request_id"]})
}

fn names(server: &mut Server) -> Vec<String> {
    let reply = server
        .handle(json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}))
        .unwrap();
    reply["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn read_only_policy_hides_writes_and_unsupported() {
    let mut server = Server::new(McpSettings::default(), fake);
    assert_eq!(names(&mut server), ["version"]);
    let reply = server
        .handle(json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"files.write","arguments":{}}}))
        .unwrap();
    assert_eq!(reply["error"]["code"], -32602);
}

#[test]
fn writes_require_confirm_when_allowed() {
    let settings = McpSettings::default().set("allowed_tools", "all").unwrap();
    let mut server = Server::new(settings, fake);
    assert_eq!(names(&mut server), ["version", "files.write"]);
    let call = |args: Value| json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"files.write","arguments":args}});
    assert_eq!(
        server.handle(call(json!({"a":1}))).unwrap()["error"]["code"],
        -32602
    );
    let reply = server.handle(call(json!({"a":1,"confirm":true}))).unwrap();
    assert_eq!(reply["result"]["isError"], false);
    assert_eq!(
        reply["result"]["structuredContent"]["data"]["echo"],
        json!({"a":1})
    );
}

#[test]
fn protocol_basics() {
    let mut server = Server::new(McpSettings::default(), fake);
    let init = server
        .handle(json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}))
        .unwrap();
    assert_eq!(init["result"]["protocolVersion"], "2024-11-05");
    assert!(
        server
            .handle(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .is_none()
    );
    assert_eq!(
        server
            .handle(json!({"jsonrpc":"2.0","id":9,"method":"nope"}))
            .unwrap()["error"]["code"],
        -32601
    );
    let mut out = Vec::new();
    server
        .serve(
            &b"not json\n\n{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"ping\"}\n"[..],
            &mut out,
        )
        .unwrap();
    let lines: Vec<Value> = String::from_utf8(out)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines[0]["error"]["code"], -32700);
    assert_eq!(lines[1], json!({"jsonrpc":"2.0","id":5,"result":{}}));
}

#[test]
fn settings_keys() {
    assert!(McpSettings::default().set("enabled", "yes").is_err());
    assert!(McpSettings::default().set("allowed_tools", "some").is_err());
    assert!(
        McpSettings::default()
            .set("enabled", "true")
            .unwrap()
            .enabled
    );
    assert!(is_read_only("reviews.playback.status") && !is_read_only("subtitles.convert"));
}
