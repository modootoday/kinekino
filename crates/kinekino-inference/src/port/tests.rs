use super::*;
use kinekino_core::port::contract;

#[test]
fn caps_algebra_and_errors() {
    contract::check_capability_table(InferenceCaps::all());
    contract::check_error_mapping(&InferenceError::samples());
    contract::check_unique_kinds(&[
        InferenceKind::HfCloud,
        InferenceKind::LocalPack,
        InferenceKind::Mock,
    ]);
}

#[test]
fn request_roundtrips_as_tagged_json() {
    let r = InferenceRequest::Generate {
        model: "m".into(),
        prompt: "hi".into(),
    };
    let j = serde_json::to_string(&r).unwrap();
    assert!(j.contains("\"task\":\"generate\""));
    assert_eq!(serde_json::from_str::<InferenceRequest>(&j).unwrap(), r);
}
