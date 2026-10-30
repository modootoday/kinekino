//! Select the existing callback-free start component from original display observations.
use meari::domain::{Value, node_slots::NodeSlot, source_modules::ComponentReference};
use std::collections::BTreeMap;

pub fn component_reference() -> ComponentReference {
    ComponentReference {
        module: "viewing/start-screen.tsx"
            .to_owned()
            .try_into()
            .expect("fixed start screen module path"),
        export: "ViewingStart".into(),
    }
}

/// The original document factory issues this slot; no visibility CSS or hidden focus owner.
pub fn compose_slot(
    root: &mut BTreeMap<String, Value>,
    make: impl FnOnce(ComponentReference, BTreeMap<String, Value>) -> Result<Value, String>,
) -> Result<(), String> {
    let observed = match root.get("nativeObservationPresent") {
        None => return Ok(()), // Callback-free ViewingPlayer owns its inline start component.
        Some(Value::Bool(observed)) => *observed,
        _ => return Err("Start screen requires the original boolean observation".into()),
    };
    let slot = if observed {
        Value::NodeSlot(NodeSlot::Empty)
    } else {
        let mut labels = BTreeMap::new();
        for name in ["productName", "connectionLabel"] {
            let Some(Value::String(value)) = root.get(name) else {
                return Err(format!("Start screen requires original {name}"));
            };
            labels.insert(name.into(), Value::String(value.clone()));
        }
        make(component_reference(), labels)?
    };
    root.insert("startScreen".into(), slot);
    Ok(())
}
