use super::*;
pub(super) fn raw_reference() -> ComponentReference {
    ComponentReference {
        module: path("studio/raw-review-panel.tsx"),
        export: "RawReviewPanel".into(),
    }
}

pub(super) fn workspace_reference() -> ComponentReference {
    ComponentReference {
        module: path("studio/review-workspace.tsx"),
        export: "ReviewWorkspace".into(),
    }
}
pub(super) fn sources() -> BTreeMap<BuildPath, String> {
    BTreeMap::from([
        (
            raw_reference().module,
            include_str!("../../../web_ui/studio/raw-review-panel.tsx").into(),
        ),
        (
            workspace_reference().module,
            include_str!("../../../web_ui/studio/review-workspace.tsx").into(),
        ),
    ])
}
pub(super) fn split(props: &mut BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let keys: Vec<_> = props
        .keys()
        .filter(|key| key.starts_with("rawReview"))
        .cloned()
        .collect();
    let mut raw = BTreeMap::new();
    for key in keys {
        if let Some(value) = props.remove(&key) {
            raw.insert(key, value);
        }
    }
    if raw.is_empty() {
        super::super::raw_review_panel::project_component_props(&mut raw, None, false, None, None);
        // Native12 registration is a mount prerequisite. This enables input, not reviewed data.
        raw.insert("rawReviewFieldsDisabled".into(), Value::Bool(false));
    }
    raw
}

#[cfg(test)]
mod tests {
    use super::*;
    use meari::domain::host_callbacks::{CallbackReference, CallbackSignature};

    #[test]
    fn reopened_raw_review_rejects_changed_fixed_sibling_declarations() {
        let entry = path("raw-review-editor-fixture.tsx");
        let mut inventory = sources();
        inventory.insert(
            entry.clone(),
            "export function Editor(){return <article>Original</article>;}".into(),
        );
        inventory.insert(
            raw_reference().module,
            "export function RawReviewPanel(){return <aside>Foreign replacement</aside>;}".into(),
        );
        let editor = meari::parse_module_graph(&entry, &inventory).unwrap();
        let error = match FrameComposition::new("Product".into(), "Original proposal".into())
            .prepare(editor, &mut inventory)
        {
            Ok(_) => panic!("changed fixed sibling must be rejected"),
            Err(error) => error,
        };
        assert_eq!(error.file, path("studio/frame.tsx"));
        assert_eq!(
            error.diagnostic.message,
            "Editor graph collides with the fixed frame inventory"
        );
    }

    #[test]
    fn reopened_raw_review_is_an_owned_sibling_without_polluting_typed_editor_props() {
        let entry = path("raw-review-editor-fixture.tsx");
        let mut inventory = sources();
        inventory.insert(entry.clone(), "type Props={title:string;onRefresh:()=>void};export function Editor(props:Props){return <article><h1>{props.title}</h1><button onClick={() => props.onRefresh()}>Refresh</button></article>;}".into());
        let editor = meari::parse_module_graph(&entry, &inventory).unwrap();
        let (graph, frame) = FrameComposition::new("Product".into(), "Original proposal".into())
            .prepare(editor, &mut inventory)
            .unwrap();
        assert!(frame.raw_review_enabled());
        let mut props = BTreeMap::from([
            (
                "title".into(),
                Value::String("Original editable proposal".into()),
            ),
            (
                "onRefresh".into(),
                Value::CallbackReference(CallbackReference {
                    binding: "originalRefresh".into(),
                    signature: CallbackSignature::Action,
                }),
            ),
        ]);
        let mut raw = BTreeMap::new();
        super::super::super::raw_review_panel::project_component_props(
            &mut raw, None, false, None, None,
        );
        raw.insert("rawReviewPackage".into(), Value::String("a".repeat(64)));
        raw.insert(
            "rawReviewItem".into(),
            Value::String("original-item".into()),
        );
        use meari::runtime::IntoValue;
        raw.insert(
            "rawReviewIssues".into(),
            vec!["원 후보의 알려진 문제".to_owned()].into_value(),
        );
        raw.insert(
            "rawReviewRangeStatus".into(),
            Value::String("시간 범위 미제공 · 영상 PTS 미확인".into()),
        );
        raw.insert(
            "rawReviewAttempt".into(),
            Value::String("마지막 명시 기록 요청: original-decision".into()),
        );
        raw.insert(
            "rawReviewHistory".into(),
            vec!["판정 ID: original-decision · 보류".to_owned()].into_value(),
        );
        for invalid in [None, Some(Value::Bool(false))] {
            let mut rejected = props.clone();
            let mut invalid_raw = raw.clone();
            invalid_raw.remove("rawReviewRangeStatus");
            if let Some(value) = invalid {
                invalid_raw.insert("rawReviewRangeStatus".into(), value);
            }
            rejected.extend(invalid_raw);
            assert!(
                frame
                    .mount(
                        graph.clone(),
                        rejected,
                        "raw-reopen-invalid-range".into(),
                        1
                    )
                    .is_err()
            );
        }
        props.extend(raw);
        let program = frame
            .mount(graph, props, "raw-reopen-original-factory".into(), 1)
            .unwrap();
        let projection = program.project().unwrap();
        let handlers = projection.callback_bindings();
        for binding in [
            "originalRefresh",
            "rawReviewOnOpenPackage",
            "rawReviewOnOpenItem",
            "rawReviewOnReadSource",
            "rawReviewOnReadCandidate",
            "rawReviewOnHistory",
        ] {
            assert!(
                handlers
                    .values()
                    .any(|handler| handler.reference.binding == binding)
            );
        }
        for binding in [
            "rawReviewOnRecord",
            "rawReviewOnUsable",
            "rawReviewOnNeedsChanges",
            "rawReviewOnDeferred",
        ] {
            assert!(
                !handlers
                    .values()
                    .any(|handler| handler.reference.binding == binding),
                "reopening must not expose an automatic or implicit quality write"
            );
        }
        let text = serde_json::to_string(&projection.component).unwrap();
        assert!(text.contains("Original editable proposal"));
        assert!(text.contains(&"a".repeat(64)));
        assert!(text.contains("original-item"));
        assert!(text.contains("원 후보의 알려진 문제"));
        assert!(text.contains("시간 범위 미제공"));
        assert!(text.contains("판정 ID: original-decision"));
        assert!(text.contains("마지막 명시 기록 요청: original-decision"));
    }
}
