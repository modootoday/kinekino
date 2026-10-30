//! Strict original factory checks; no replacement schema, SDK or callback authority.
use meari::domain::{
    Value,
    host_callbacks::{CallbackReference, CallbackSignature},
};
use std::collections::BTreeMap;

pub(crate) fn assert_required_caption_fields(
    original: &BTreeMap<String, Value>,
    mut accepts: impl FnMut(BTreeMap<String, Value>) -> bool,
) {
    assert!(
        accepts(original.clone()),
        "original factory must accept the positive control"
    );
    for name in [
        "captionMinimizeDisabled",
        "captionMaximizeDisabled",
        "captionCloseDisabled",
        "captionMaximizeLabel",
        "captionMaximizeIconClass",
        "captionStatus",
        "captionStatusError",
        "onWindowMinimize",
        "onWindowToggleMaximize",
        "onWindowUserClose",
    ] {
        assert!(
            original.contains_key(name),
            "original producer omitted {name}"
        );
        let mut missing = original.clone();
        missing.remove(name);
        assert!(!accepts(missing), "factory accepted missing {name}");
        let mut wrong = original.clone();
        let wrong_value = if matches!(original.get(name), Some(Value::Bool(_))) {
            Value::String("wrong caption flag type".into())
        } else {
            Value::Bool(false)
        };
        wrong.insert(name.into(), wrong_value);
        assert!(!accepts(wrong), "factory accepted wrong type for {name}");
        if name.starts_with("onWindow") {
            let mut wrong_signature = original.clone();
            wrong_signature.insert(
                name.into(),
                Value::CallbackReference(CallbackReference {
                    binding: name.into(),
                    signature: CallbackSignature::Text,
                }),
            );
            assert!(
                !accepts(wrong_signature),
                "factory accepted wrong callback signature for {name}"
            );
        }
    }
}
