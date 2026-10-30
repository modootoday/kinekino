//! Typed presentation rows; origin presence enables an entry affordance, never SDK authority.
use meari::domain::{
    Prop, PropType, Value,
    array_value::ArrayValue,
    host_callbacks::{CallbackReference, CallbackSignature},
    record_value::RecordValue,
};
use std::collections::BTreeMap;

pub struct CaptureChoiceRow {
    pub key: String,
    pub number: String,
    pub front: String,
    pub back: String,
    pub label: String,
    pub editable: bool,
}
pub fn props(rows: Vec<CaptureChoiceRow>, visible: bool) -> BTreeMap<String, Value> {
    let labels = super::capture_labels::CaptureLabels::korean();
    let schema: Vec<_> = ["key", "number", "front", "back", "label"]
        .into_iter()
        .map(|name| Prop {
            name: name.into(),
            kind: PropType::String,
        })
        .chain(std::iter::once(Prop {
            name: "disabled".into(),
            kind: PropType::Bool,
        }))
        .collect();
    let items = rows
        .into_iter()
        .map(|row| {
            Value::Record(
                RecordValue::new(
                    schema.clone(),
                    BTreeMap::from([
                        ("key".into(), Value::String(row.key)),
                        ("number".into(), Value::String(row.number)),
                        ("front".into(), Value::String(row.front)),
                        ("back".into(), Value::String(row.back)),
                        ("label".into(), Value::String(row.label)),
                        ("disabled".into(), Value::Bool(!row.editable)),
                    ]),
                )
                .expect("fixed Card choice schema and matching typed fields"),
            )
        })
        .collect();
    BTreeMap::from([
        (
            "capturedCardsLabel".into(),
            Value::String(labels.choice_label),
        ),
        (
            "capturedCardsHint".into(),
            Value::String(labels.choice_hint),
        ),
        ("capturedCardsVisible".into(), Value::Bool(visible)),
        (
            "capturedCards".into(),
            Value::Array(ArrayValue {
                element: PropType::Record(schema),
                items,
            }),
        ),
        ("capturedCardsSelected".into(), Value::String(String::new())),
        (
            "onCapturedCardEdit".into(),
            Value::CallbackReference(CallbackReference {
                binding: "onCapturedCardEdit".into(),
                signature: CallbackSignature::Text,
            }),
        ),
    ])
}

/// Consume internal row projection before the exact Viewing root props are mounted.
/// The fixed component is created only for visible choices; malformed inputs fail closed.
pub fn compose_slot(
    props: &mut BTreeMap<String, Value>,
    make: impl FnOnce(
        meari::domain::source_modules::ComponentReference,
        BTreeMap<String, Value>,
    ) -> Result<Value, String>,
) -> Result<(), String> {
    let Some(visible) = props.remove("capturedCardsVisible") else {
        return Ok(());
    };
    let Value::Bool(visible) = visible else {
        return Err("Captured choice visibility must be boolean".into());
    };
    let mut rail = BTreeMap::new();
    for (source, target) in [
        ("capturedCardsLabel", "label"),
        ("capturedCardsHint", "hint"),
        ("capturedCardsSelected", "selectedKey"),
        ("capturedCards", "items"),
        ("onCapturedCardEdit", "onSelect"),
    ] {
        let value = props
            .remove(source)
            .ok_or_else(|| format!("Missing captured choice prop: {source}"))?;
        let valid = match target {
            "label" | "hint" | "selectedKey" => matches!(&value, Value::String(_)),
            "items" => matches!(&value, Value::Array(_)),
            "onSelect" => {
                matches!(&value, Value::CallbackReference(reference) if reference.signature == CallbackSignature::Text)
            }
            _ => false,
        };
        if !valid {
            return Err(format!("Invalid captured choice prop: {source}"));
        }
        rail.insert(target.into(), value);
    }
    let slot = if visible {
        make(
            meari::domain::source_modules::ComponentReference {
                module: "studio/cards/selection-rail.tsx"
                    .to_owned()
                    .try_into()
                    .map_err(|error| format!("{error:?}"))?,
                export: "CardSelectionRail".into(),
            },
            rail,
        )?
    } else {
        Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty)
    };
    props.insert("capturedCardChoices".into(), slot);
    Ok(())
}

#[cfg(test)]
mod slot_tests {
    use super::*;
    #[test]
    fn absent_projection_does_not_issue_a_child() {
        let mut values = BTreeMap::new();
        compose_slot(&mut values, |_, _| {
            panic!("absent projection must not mount")
        })
        .unwrap();
        assert!(values.is_empty());
    }
    #[test]
    fn invisible_projection_is_explicit_empty_and_consumes_internal_props() {
        let mut values = props(Vec::new(), false);
        compose_slot(&mut values, |_, _| {
            panic!("invisible projection must not mount")
        })
        .unwrap();
        assert_eq!(values.len(), 1);
        assert!(matches!(
            values.get("capturedCardChoices"),
            Some(Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty))
        ));
    }
    #[test]
    fn visible_projection_issues_exact_fixed_rail_with_readonly_rows() {
        let mut values = props(
            vec![CaptureChoiceRow {
                key: "legacy".into(),
                number: "1".into(),
                front: "front".into(),
                back: "back".into(),
                label: "legacy card".into(),
                editable: false,
            }],
            true,
        );
        let mut issued = false;
        compose_slot(&mut values, |reference, rail| {
            issued = true;
            assert_eq!(reference.module.as_str(), "studio/cards/selection-rail.tsx");
            assert_eq!(reference.export, "CardSelectionRail");
            assert_eq!(rail.len(), 5);
            let Value::Array(items) = &rail["items"] else {
                panic!("typed rows")
            };
            let Value::Record(row) = &items.items[0] else {
                panic!("typed row")
            };
            assert_eq!(row.fields()["disabled"], Value::Bool(true));
            Ok(Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty))
        })
        .unwrap();
        assert!(issued);
        assert_eq!(values.len(), 1);
    }
    #[test]
    fn malformed_visible_projection_is_an_error_without_mount() {
        let mut values = props(Vec::new(), true);
        values.insert("capturedCardsLabel".into(), Value::Bool(false));
        assert!(
            compose_slot(&mut values, |_, _| panic!(
                "invalid projection must not mount"
            ))
            .is_err()
        );
    }
}
