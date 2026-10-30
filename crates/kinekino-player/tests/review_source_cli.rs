#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "review_source_cli/combined.rs"]
mod combined;
#[path = "review_source_cli/fixture.rs"]
mod fixture;
#[path = "review_source_cli/process.rs"]
mod process;
#[path = "review_source_cli/readonly.rs"]
mod readonly;
use fixture::Fixture;
use process::call;
use serde_json::json;

#[test]
fn normal_cli_five_review_commands_reopen_replay_and_preserve_originals() {
    kinekino_platform::install();
    let f = Fixture::new();
    let payload = json!({"case":f.case,"association_id":"association","media":f.media,"note":"manual association"});
    let create = f.request(
        "reviews.association.create",
        payload.clone(),
        Some("create"),
        None,
    );
    let first = call(&f, "01-create", &create, true, None);
    let verify_without_adapter = f.request(
        "reviews.association.revise",
        json!({"case":f.case,"association_id":"association","media":f.media,"note":"verify","timeline_evidence":{"asset_id":"unconfigured-proof","sha256":"a".repeat(64)}}),
        Some("verify-unconfigured"),
        Some("1"),
    );
    call(
        &f,
        "01a-verify-unconfigured",
        &verify_without_adapter,
        false,
        Some("unsupported"),
    );
    let replay = call(&f, "02-replay", &create, true, None);
    assert_eq!(first["data"], replay["data"]);
    assert_eq!(
        first["data"]["receipt"]["body"]["association"]["revision"],
        "1"
    );
    let mut changed = create.clone();
    changed["payload"]["note"] = "changed same key".into();
    call(
        &f,
        "03-key-conflict",
        &changed,
        false,
        Some("revision_conflict"),
    );
    let revise = f.request(
        "reviews.association.revise",
        payload,
        Some("revise"),
        Some("1"),
    );
    let revised = call(&f, "04-revise", &revise, true, None);
    assert_eq!(
        revised["data"]["receipt"]["body"]["association"]["revision"],
        "2"
    );
    let mut stale = revise;
    stale["context"]["idempotency_key"] = "stale".into();
    call(&f, "05-stale", &stale, false, Some("revision_conflict"));
    let inspect = f.request(
        "reviews.association.inspect",
        json!({"case":f.case,"association_id":"association"}),
        None,
        None,
    );
    assert_eq!(
        call(&f, "06-inspect", &inspect, true, None)["data"]["stored"]["association"]["revision"],
        "2"
    );
    let draft = json!({"draft_id":"draft","association_id":"association","association_revision":"1","draft_revision":"1","authored_by":"user","ranges":[{"start_us":"1","end_us":"3"}],"note":"manual range"});
    let save = f.request(
        "reviews.range_draft.save",
        json!({"case":f.case,"draft":draft}),
        Some("save"),
        None,
    );
    let saved = call(&f, "07-save", &save, true, None);
    assert_eq!(
        saved["data"],
        call(&f, "08-save-replay", &save, true, None)["data"]
    );
    let read = f.request(
        "reviews.range_draft.read",
        json!({"case":f.case,"draft_id":"draft"}),
        None,
        None,
    );
    let view = call(&f, "09-read", &read, true, None);
    assert_eq!(view["data"]["stored"]["association"]["revision"], "1");
    assert_eq!(view["data"]["media_check"], "not_checked");
    assert_eq!(view["data"]["playback_ready"], false);
    let mut bad = read.clone();
    bad["context"]["idempotency_key"] = "forbidden".into();
    call(&f, "10-read-context", &bad, false, Some("invalid_argument"));
    let mut update = save;
    update["context"]["expected_revision"] = "1".into();
    update["context"]["idempotency_key"] = "save2".into();
    update["payload"]["draft"]["draft_revision"] = "2".into();
    update["payload"]["draft"]["note"] = "updated".into();
    call(&f, "11-update", &update, true, None);
    assert_eq!(
        call(&f, "12-reopen", &read, true, None)["data"]["stored"]["draft"]["note"],
        "updated"
    );
    for (index, name) in kinekino_player::application::review_source::api::COMMANDS
        .iter()
        .enumerate()
    {
        let query = f.request("schema", json!({"command":name}), None, None);
        assert_eq!(
            call(&f, &format!("schema-{index}"), &query, true, None)["data"]["status"],
            "supported"
        );
    }
    let caps = f.request("capabilities", json!({}), None, None);
    let capability = call(&f, "capabilities", &caps, true, None);
    assert_eq!(
        capability["data"]["review_source"]["strong_cancel_available"],
        false
    );
    assert_eq!(
        capability["data"]["review_source"]["storage_limits"]["max_case_transactions"],
        "4096"
    );
    assert_eq!(
        capability["data"]["review_source"]["storage_limits"]["max_transaction_bytes"],
        "262144"
    );
    assert_eq!(
        capability["data"]["review_source"]["committed_key_expiry"],
        "none"
    );
    let mut bad = create;
    bad["payload"]["timeline_status"] = "verified".into();
    call(
        &f,
        "13-forged-verified",
        &bad,
        false,
        Some("invalid_argument"),
    );
    println!("Preserved actual CLI evidence: {}", f.root.display());
}
