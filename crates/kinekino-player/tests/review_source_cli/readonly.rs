use super::{fixture::Fixture, process::call};
use serde_json::json;
#[test]
fn readonly_cli_does_not_initialize_missing_user_namespace() {
    kinekino_platform::install();
    let f = Fixture::new();
    let read = f.request(
        "reviews.range_draft.read",
        json!({"case":f.case,"draft_id":"missing"}),
        None,
        None,
    );
    call(&f, "readonly-missing", &read, false, Some("not_found"));
    assert_eq!(std::fs::read_dir(&f.user).unwrap().count(), 0);
    let create = f.request(
        "reviews.association.create",
        json!({"case":f.case,"association_id":"later","media":f.media,"note":"manual"}),
        Some("later"),
        None,
    );
    call(&f, "writer-after-read", &create, true, None);
    assert!(f.user.join("cases").exists());
    f.assert_originals();
}
