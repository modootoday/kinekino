//! Manual component status from common facts; no typed-editor props or policy copy.
use super::super::{panel_model::PanelLabels, review_feedback::ReviewFeedback};
use meari::domain::Value;
use std::collections::BTreeMap;

pub fn apply_feedback(
    feedback: &ReviewFeedback<'_>,
    props: &mut BTreeMap<String, Value>,
    labels: &PanelLabels,
) {
    let rendered = feedback.render(labels);
    props.insert(
        "resultLabel".into(),
        Value::String(format!(
            "{}{}{}",
            rendered.revision, labels.feedback.separator, rendered.result
        )),
    );
}
