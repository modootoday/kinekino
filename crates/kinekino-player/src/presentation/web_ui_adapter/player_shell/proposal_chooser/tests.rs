use super::*;

fn array(values: &BTreeMap<String, Value>, name: &str) -> ArrayValue {
    let Value::Array(array) = &values[name] else {
        panic!("typed chooser array")
    };
    array.clone()
}

#[test]
fn initial_empty_display_has_no_selected_or_open_key_and_keeps_typed_empty_rows() {
    let values = props(ChooserView::default(), ChooserLabels::korean()).unwrap();
    for name in ["selectedDocumentKey", "selectedItemKey", "openKey"] {
        assert_eq!(values[name], Value::String(String::new()));
    }
    for name in ["listDisabled", "nextDisabled", "openDisabled"] {
        assert_eq!(values[name], Value::Bool(true));
    }
    for name in ["documents", "items"] {
        let array = array(&values, name);
        assert!(array.items.is_empty());
        assert!(matches!(array.element, PropType::Record(_)));
        array.validate().unwrap();
    }
}

#[test]
fn rows_preserve_opaque_keys_literal_text_order_and_host_disabled_states() {
    let literal = "<script>not markup</script> & \"日本語\"";
    let values = props(
        ChooserView {
            documents: vec![
                ChoiceRow {
                    key: "opaque-document-second".into(),
                    title: literal.into(),
                    detail: "문서 상태".into(),
                    disabled: true,
                },
                ChoiceRow {
                    key: "opaque-document-first".into(),
                    title: "첫 번째를 자동 선택하지 않음".into(),
                    detail: String::new(),
                    disabled: false,
                },
            ],
            items: vec![ChoiceRow {
                key: "opaque-item".into(),
                title: "항목".into(),
                detail: "저장된 텍스트".into(),
                disabled: false,
            }],
            list_disabled: false,
            ..ChooserView::default()
        },
        ChooserLabels::korean(),
    )
    .unwrap();
    let documents = array(&values, "documents");
    documents.validate().unwrap();
    let Value::Record(first) = &documents.items[0] else {
        panic!("typed row")
    };
    assert_eq!(
        first.fields()["key"],
        Value::String("opaque-document-second".into())
    );
    assert_eq!(first.fields()["title"], Value::String(literal.into()));
    assert_eq!(first.fields()["disabled"], Value::Bool(true));
    assert_eq!(values["selectedDocumentKey"], Value::String(String::new()));
    assert_eq!(values["selectedItemKey"], Value::String(String::new()));
    assert_eq!(values["openDisabled"], Value::Bool(true));
    assert_eq!(array(&values, "items").items.len(), 1);
}

#[test]
fn supplied_open_affordance_and_status_error_are_independent_display_inputs() {
    let values = props(
        ChooserView {
            selected_document_key: "document-choice".into(),
            selected_item_key: "item-choice".into(),
            status: "처리 결과를 유지합니다".into(),
            error: "ACK 오류".into(),
            list_disabled: true,
            next_disabled: false,
            open: Some(OpenAffordance {
                key: "host-open-key".into(),
                disabled: true,
            }),
            ..ChooserView::default()
        },
        ChooserLabels::korean(),
    )
    .unwrap();
    assert_eq!(
        values["selectedDocumentKey"],
        Value::String("document-choice".into())
    );
    assert_eq!(
        values["selectedItemKey"],
        Value::String("item-choice".into())
    );
    assert_eq!(values["openKey"], Value::String("host-open-key".into()));
    assert_eq!(values["openDisabled"], Value::Bool(true));
    assert_eq!(values["nextDisabled"], Value::Bool(false));
    assert_eq!(
        values["status"],
        Value::String("처리 결과를 유지합니다".into())
    );
    assert_eq!(values["error"], Value::String("ACK 오류".into()));
}

#[test]
fn callbacks_reference_original_binding_names_and_exact_signatures_only() {
    let values = props(ChooserView::default(), ChooserLabels::korean()).unwrap();
    for (name, signature) in [
        ("onChooserList", CallbackSignature::Action),
        ("onChooserNext", CallbackSignature::Action),
        ("onChooserDocument", CallbackSignature::Text),
        ("onChooserItem", CallbackSignature::Text),
        ("onOpenLiveProposal", CallbackSignature::Text),
    ] {
        assert_eq!(
            values[name],
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature,
            })
        );
    }
    let reference = component_reference();
    assert_eq!(reference.module.as_str(), "viewing/proposal-chooser.tsx");
    assert_eq!(reference.export, "ProposalChooser");
}

#[test]
fn alternate_labels_do_not_change_selection_or_callback_identity() {
    let values = props(
        ChooserView::default(),
        ChooserLabels {
            label: "保存した提案".into(),
            documents: "ドキュメント".into(),
            items: "項目".into(),
            list: "一覧を更新".into(),
            next: "次へ".into(),
            open: "開く".into(),
        },
    )
    .unwrap();
    assert_eq!(values["label"], Value::String("保存した提案".into()));
    assert_eq!(values["openLabel"], Value::String("開く".into()));
    assert_eq!(values["openDisabled"], Value::Bool(true));
    assert!(
        matches!(&values["onOpenLiveProposal"], Value::CallbackReference(reference)
        if reference.binding == "onOpenLiveProposal" && reference.signature == CallbackSignature::Text)
    );
}

#[test]
fn initial_read_root_has_an_empty_slot_without_new_callback_registration() {
    let values = super::super::read_props(&super::super::StartLabels {
        connection: "연결".into(),
        time: "00:00".into(),
    });
    assert_eq!(
        values["proposalChooser"],
        Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty)
    );
    assert!(!values.contains_key("onChooserList"));
    assert!(!values.contains_key("onOpenLiveProposal"));
}

#[test]
fn optional_chooser_provider_is_closed_without_changing_the_callback_free_root() {
    use super::super::{StartLabels, enable_read_panel, prepare_start_document};
    let labels = StartLabels {
        connection: "연결".into(),
        time: "00:00".into(),
    };
    let catalog =
        crate::presentation::web_ui_adapter::player_shell::RouteCatalog::bundled().unwrap();
    let mut document = prepare_start_document(&catalog, "view.start", labels)
        .ok()
        .unwrap();
    assert_eq!(document.inventory.len(), 4);
    assert!(
        !document
            .graph
            .modules
            .contains_key(&component_reference().module)
    );
    let labels = StartLabels {
        connection: "연결".into(),
        time: "00:00".into(),
    };
    enable_read_panel(&mut document, super::super::read_props(&labels))
        .ok()
        .unwrap();
    assert_eq!(document.inventory.len(), 7);
    assert!(
        document
            .graph
            .modules
            .contains_key(&component_reference().module)
    );
    document.graph.validate().unwrap();
    assert_eq!(
        document.props["proposalChooser"],
        Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty)
    );
}

#[test]
fn oversized_document_or_item_text_returns_original_budget_error() {
    for document in [true, false] {
        let row = ChoiceRow {
            key: "unchanged-opaque-key".into(),
            title: "x".repeat(1024 * 1024),
            detail: String::new(),
            disabled: false,
        };
        let mut view = ChooserView::default();
        if document {
            view.documents.push(row);
        } else {
            view.items.push(row);
        }
        assert_eq!(
            props(view, ChooserLabels::korean()).unwrap_err(),
            "prop exceeds total byte budget"
        );
    }
}

#[test]
fn individually_valid_rows_still_return_the_original_combined_array_budget_error() {
    let make_row = || ChoiceRow {
        key: "unchanged-key".into(),
        title: "x".repeat(64 * 1024),
        detail: String::new(),
        disabled: false,
    };
    rows(vec![make_row()]).unwrap();
    let view = ChooserView {
        documents: (0..17).map(|_| make_row()).collect(),
        ..ChooserView::default()
    };
    assert_eq!(
        props(view, ChooserLabels::korean()).unwrap_err(),
        "prop exceeds total byte budget"
    );
}

#[test]
fn unconfigured_root_keeps_empty_slot_without_issuing_or_registering_callbacks() {
    let mut root = BTreeMap::from([(
        "proposalChooser".into(),
        Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty),
    )]);
    let before = root.clone();
    compose_slot(&mut root, |_, _| {
        panic!("non-chooser must not issue a slot")
    })
    .unwrap();
    assert_eq!(root, before);
}

#[test]
fn staged_typed_display_is_consumed_by_the_original_slot_factory_once() {
    let values = props(ChooserView::default(), ChooserLabels::korean()).unwrap();
    let mut root = BTreeMap::new();
    stage_root(&mut root, values.clone()).unwrap();
    assert_eq!(staged_props(&root), Some(&values));
    let mut calls = 0;
    compose_slot(&mut root, |reference, component| {
        calls += 1;
        assert_eq!(reference, component_reference());
        assert_eq!(component, values);
        Ok(Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty))
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert!(staged_props(&root).is_none());
    assert_eq!(root.len(), 1);
}

#[test]
fn malformed_or_over_budget_projection_returns_error_without_slot_success() {
    let mut root = BTreeMap::from([(ROOT_PROJECTION.into(), Value::String("foreign".into()))]);
    assert!(
        compose_slot(&mut root, |_, _| panic!(
            "malformed input must not issue a slot"
        ))
        .is_err()
    );
    let mut values = props(ChooserView::default(), ChooserLabels::korean()).unwrap();
    values.insert("status".into(), Value::String("x".repeat(1024 * 1024)));
    let mut root = BTreeMap::new();
    assert_eq!(
        stage_root(&mut root, values).unwrap_err(),
        "prop exceeds total byte budget"
    );
    assert!(root.is_empty());
    stage_root(
        &mut root,
        props(ChooserView::default(), ChooserLabels::korean()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        compose_slot(&mut root, |_, _| Err(
            "original document factory error".into()
        ))
        .unwrap_err(),
        "original document factory error"
    );
    assert!(!root.contains_key("proposalChooser"));
}
