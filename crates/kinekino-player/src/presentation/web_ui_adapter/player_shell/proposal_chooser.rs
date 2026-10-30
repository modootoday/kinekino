//! Pure display input. Opaque row strings and callback references carry no SDK authority.
use meari::domain::{
    Prop, PropType, Value,
    array_value::ArrayValue,
    host_callbacks::{CallbackReference, CallbackSignature},
    record_value::RecordValue,
    source_modules::ComponentReference,
};
use std::collections::BTreeMap;

pub struct ChoiceRow {
    pub key: String,
    pub title: String,
    pub detail: String,
    pub disabled: bool,
}

pub struct OpenAffordance {
    pub key: String,
    pub disabled: bool,
}

pub struct ChooserView {
    pub documents: Vec<ChoiceRow>,
    pub items: Vec<ChoiceRow>,
    pub selected_document_key: String,
    pub selected_item_key: String,
    pub status: String,
    pub error: String,
    pub list_disabled: bool,
    pub next_disabled: bool,
    pub open: Option<OpenAffordance>,
}

impl Default for ChooserView {
    fn default() -> Self {
        Self {
            documents: Vec::new(),
            items: Vec::new(),
            selected_document_key: String::new(),
            selected_item_key: String::new(),
            status: String::new(),
            error: String::new(),
            list_disabled: true,
            next_disabled: true,
            open: None,
        }
    }
}

/// Labels are supplied together; another locale need not change source or SDK state.
pub struct ChooserLabels {
    pub label: String,
    pub documents: String,
    pub items: String,
    pub list: String,
    pub next: String,
    pub open: String,
}

impl ChooserLabels {
    pub fn korean() -> Self {
        Self {
            label: "저장된 제안".into(),
            documents: "제안 문서 선택".into(),
            items: "제안 항목 선택".into(),
            list: "제안 목록 새로 조회".into(),
            next: "다음 제안 목록".into(),
            open: "선택한 제안 열기".into(),
        }
    }
}

pub fn component_reference() -> ComponentReference {
    ComponentReference {
        module: "viewing/proposal-chooser.tsx"
            .to_owned()
            .try_into()
            .expect("fixed chooser module path"),
        export: "ProposalChooser".into(),
    }
}

pub fn props(view: ChooserView, labels: ChooserLabels) -> Result<BTreeMap<String, Value>, String> {
    let (open_key, open_disabled) = view
        .open
        .map_or((String::new(), true), |open| (open.key, open.disabled));
    let mut values = BTreeMap::from([
        ("documents".into(), rows(view.documents)?),
        ("items".into(), rows(view.items)?),
        ("listDisabled".into(), Value::Bool(view.list_disabled)),
        ("nextDisabled".into(), Value::Bool(view.next_disabled)),
        ("openDisabled".into(), Value::Bool(open_disabled)),
    ]);
    for (name, value) in [
        ("label", labels.label),
        ("documentsLabel", labels.documents),
        ("itemsLabel", labels.items),
        ("listLabel", labels.list),
        ("nextLabel", labels.next),
        ("openLabel", labels.open),
        ("status", view.status),
        ("error", view.error),
        ("selectedDocumentKey", view.selected_document_key),
        ("selectedItemKey", view.selected_item_key),
        ("openKey", open_key),
    ] {
        values.insert(name.into(), Value::String(value));
    }
    for (name, signature) in [
        ("onChooserList", CallbackSignature::Action),
        ("onChooserNext", CallbackSignature::Action),
        ("onChooserDocument", CallbackSignature::Text),
        ("onChooserItem", CallbackSignature::Text),
        ("onOpenLiveProposal", CallbackSignature::Text),
    ] {
        values.insert(
            name.into(),
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature,
            }),
        );
    }
    Ok(values)
}

fn rows(rows: Vec<ChoiceRow>) -> Result<Value, String> {
    let schema: Vec<_> = ["key", "title", "detail"]
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
            RecordValue::new(
                schema.clone(),
                BTreeMap::from([
                    ("key".into(), Value::String(row.key)),
                    ("title".into(), Value::String(row.title)),
                    ("detail".into(), Value::String(row.detail)),
                    ("disabled".into(), Value::Bool(row.disabled)),
                ]),
            )
            .map(Value::Record)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let array = ArrayValue {
        element: PropType::Record(schema),
        items,
    };
    array.validate()?;
    Ok(Value::Array(array))
}

const ROOT_PROJECTION: &str = "proposalChooserProjection";

/// Stage typed display data only. The original workspace factory issues the owned slot.
pub fn stage_root(
    root: &mut BTreeMap<String, Value>,
    component: BTreeMap<String, Value>,
) -> Result<(), String> {
    let schema = component
        .iter()
        .map(|(name, value)| {
            let kind = match value {
                Value::String(_) => PropType::String,
                Value::Bool(_) => PropType::Bool,
                Value::Array(array) => PropType::Array(Box::new(array.element.clone())),
                Value::CallbackReference(reference) => PropType::Callback(reference.signature),
                _ => return Err("Unsupported chooser display field".to_string()),
            };
            Ok(Prop {
                name: name.clone(),
                kind,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let record = RecordValue::new(schema, component)?;
    root.insert(ROOT_PROJECTION.into(), Value::Record(record));
    Ok(())
}

/// Read only after this exact staged input committed through the original workspace factory.
pub fn staged_props(root: &BTreeMap<String, Value>) -> Option<&BTreeMap<String, Value>> {
    match root.get(ROOT_PROJECTION) {
        Some(Value::Record(record)) => Some(record.fields()),
        _ => None,
    }
}

pub fn compose_slot(
    root: &mut BTreeMap<String, Value>,
    make: impl FnOnce(ComponentReference, BTreeMap<String, Value>) -> Result<Value, String>,
) -> Result<(), String> {
    let Some(value) = root.remove(ROOT_PROJECTION) else {
        return Ok(());
    };
    let Value::Record(record) = value else {
        return Err("Chooser display projection must be a typed record".into());
    };
    record.validate()?;
    let slot = make(component_reference(), record.fields)?;
    root.insert("proposalChooser".into(), slot);
    Ok(())
}

#[cfg(test)]
#[path = "proposal_chooser/tests.rs"]
mod tests;
