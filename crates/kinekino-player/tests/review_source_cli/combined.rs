use super::{fixture::Fixture, process::call};
use serde_json::json;
#[test]
fn combined_cli_save_publishes_one_envelope_and_reopens_both_objects() {
    kinekino_platform::install();
    let f = Fixture::new();
    let association = json!({"association_id":"combined","case":f.case,"media":f.media,"revision":"1","timeline_status":"unverified","evidence_refs":[],"note":"manual association"});
    let draft = json!({"draft_id":"combined-draft","association_id":"combined","association_revision":"1","draft_revision":"1","authored_by":"user","ranges":[],"note":"empty user draft is intentional"});
    let save = f.request(
        "reviews.range_draft.save",
        json!({"case":f.case,"draft":draft,"create_association":association}),
        Some("combined-key"),
        None,
    );
    let receipt = call(&f, "combined-save", &save, true, None);
    assert_eq!(
        receipt["data"],
        call(&f, "combined-replay", &save, true, None)["data"]
    );
    let inspect = f.request(
        "reviews.association.inspect",
        json!({"case":f.case,"association_id":"combined","revision":"1"}),
        None,
        None,
    );
    assert_eq!(
        call(&f, "combined-association", &inspect, true, None)["data"]["stored"]["association"],
        association
    );
    let read = f.request(
        "reviews.range_draft.read",
        json!({"case":f.case,"draft_id":"combined-draft"}),
        None,
        None,
    );
    assert_eq!(
        call(&f, "combined-draft", &read, true, None)["data"]["stored"]["draft"],
        draft
    );
    let case_directory = std::fs::read_dir(f.user.join("cases"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let files: Vec<_> = std::fs::read_dir(case_directory.join("transactions"))
        .unwrap()
        .map(|file| file.unwrap().path())
        .collect();
    assert_eq!(
        files.len(),
        1,
        "combined mutation must not split authority into multiple files"
    );
    let stored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap();
    assert_eq!(stored["association"], association);
    assert_eq!(stored["draft"], draft);
    assert_eq!(stored, receipt["data"]["receipt"]["body"]);
    f.assert_originals();
}
