//! Narrow display/value feedback selection; library guards remain authoritative.
use meari::domain::{Node, NodeKind, Value};
use meari_native::infrastructure::native_host::HostStamp;
use std::collections::{BTreeMap, BTreeSet};

const DISPLAY_IDS: [&str; 6] = [
    "native-range-position",
    "native-range-volume",
    "native-range-position-raw",
    "native-range-volume-raw",
    "native-range-seek-preview",
    "native-range-volume-preview",
];
const FEEDBACK_PROPS: [&str; 8] = [
    "nativeSeekRangeValue",
    "nativeVolumeRangeValue",
    "nativeSeekPreviewLabel",
    "nativeVolumePreviewLabel",
    "nativePositionLabel",
    "nativeVolumeLabel",
    "nativePositionUs",
    "nativeVolumePercent",
];

#[derive(Default)]
pub(crate) struct RangeFeedback {
    previous: Option<(HostStamp, BTreeMap<String, Value>)>,
    failure: Option<String>,
}
impl RangeFeedback {
    pub(crate) fn retained(
        &self,
        stamp: &HostStamp,
        props: &BTreeMap<String, Value>,
    ) -> Result<bool, String> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let Some((previous_stamp, previous)) = &self.previous else {
            return Ok(false);
        };
        if previous_stamp.host_id != stamp.host_id
            || previous_stamp.owner != stamp.owner
            || previous_stamp.definition_revision != stamp.definition_revision
            || props.get("nativeSeekRangeDisabled") != Some(&Value::Bool(false))
            || props.get("nativeVolumeRangeDisabled") != Some(&Value::Bool(false))
            || previous.len() != props.len()
        {
            return Ok(false);
        }
        // Original instance/artifact/duration, admission, manual callback captures and every
        // other prop must compare exactly before choosing an update that retains exports.
        Ok(previous.iter().all(|(key, old)| {
            props.get(key).is_some_and(|new| {
                new == old
                    || (FEEDBACK_PROPS.contains(&key.as_str())
                        && matches!((old, new), (Value::String(_), Value::String(_))))
            })
        }))
    }
    pub(crate) fn committed(&mut self, stamp: HostStamp, props: BTreeMap<String, Value>) {
        self.previous = Some((stamp, props));
    }
    pub(crate) fn failed(&mut self, error: String) {
        self.failure.get_or_insert(error);
    }
}

pub(crate) fn display_paths(root: &Node) -> Result<BTreeSet<String>, String> {
    fn visit(node: &Node, path: &str, found: &mut BTreeMap<String, String>) -> Result<(), String> {
        if let NodeKind::Element {
            tag,
            attributes,
            children,
        } = &node.kind
        {
            for attribute in attributes {
                if attribute.name == "id"
                    && let Value::String(id) = &attribute.value
                    && DISPLAY_IDS.contains(&id.as_str())
                    && (tag != "span"
                        || children.len() != 1
                        || !matches!(&children[0].kind, NodeKind::Text(Value::String(_)))
                        || found.insert(id.clone(), format!("{path}/0")).is_some())
                {
                    return Err("Original range display identity/shape differs".into());
                }
            }
            for (index, child) in children.iter().enumerate() {
                visit(child, &format!("{path}/{index}"), found)?;
            }
        }
        Ok(())
    }
    let mut found = BTreeMap::new();
    visit(root, "0", &mut found)?;
    if found.len() != DISPLAY_IDS.len() {
        return Err("Original range display nodes missing".into());
    }
    Ok(found.into_values().collect())
}
