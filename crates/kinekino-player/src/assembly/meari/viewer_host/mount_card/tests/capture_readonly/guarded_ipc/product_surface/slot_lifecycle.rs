//! Source-only candidate: actual product slots, original typed SDK/IPC/host fixture.
use super::*;
use crate::presentation::web_ui_adapter::player_shell::document::product_frame::WorkspaceRoute;
use meari::domain::{
    Value,
    host_callbacks::{CallbackReference, CallbackSignature},
};
use meari_native::infrastructure::accessibility;
use std::collections::BTreeMap;

const RAIL: &str = "studio/cards/selection-rail.tsx";

// Test-only projection input to the existing atomic adapter, not a new product toggle API.
fn reproject(
    host: &ViewerHost,
    visible: bool,
    edit: impl FnOnce(&mut BTreeMap<String, Value>),
) -> Result<(), String> {
    let handle = host.document.as_ref().unwrap();
    let owner = host.callbacks.borrow();
    let mut props = host.base_props.clone();
    props.extend(owner.projection.borrow().panel.props());
    props.extend(owner.local_props());
    props.insert("capturedCardsVisible".into(), Value::Bool(visible));
    edit(&mut props);
    let bindings = owner.bindings().map_err(|error| error.message)?;
    let frame = host.product_frame.as_ref().unwrap();
    handle
        .try_reproject(
            &handle.stamp().map_err(|error| format!("{error:?}"))?,
            |document| {
                let props = frame.workspace_props(
                    document,
                    WorkspaceRoute::Viewing,
                    host.viewing_generation,
                    props,
                )?;
                document.update_module_props_with_callbacks(props, bindings)
            },
        )
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
}

fn radio_dom_id(host: &ViewerHost) -> u64 {
    let handle = host.document.as_ref().unwrap();
    let path = captured_radio_path(handle);
    handle
        .try_with(|document| {
            document
                .renderer
                .document
                .get_element_by_id(document.path_id(&path).unwrap())
                .unwrap()
                .as_u64()
        })
        .unwrap()
}

fn rail_instance(host: &ViewerHost) -> meari::domain::component_state::InstanceHandle {
    host.document
        .as_ref()
        .unwrap()
        .try_with(|document| {
            let projection = document.module_projection().unwrap();
            let origin = projection
                .origins
                .values()
                .find(|origin| origin.module.as_str() == RAIL)
                .expect("actual Rail element origin");
            projection
                .instance(&origin.instance)
                .expect("actual Rail instance")
                .clone()
        })
        .unwrap()
}

fn assert_no_rail(host: &ViewerHost) {
    host.document
        .as_ref()
        .unwrap()
        .try_with(|document| {
            let projection = document.module_projection().unwrap();
            assert!(
                !projection
                    .calls
                    .values()
                    .any(|call| call.module.as_str() == RAIL)
            );
            assert!(
                !projection
                    .origins
                    .values()
                    .any(|origin| origin.module.as_str() == RAIL)
            );
            assert!(
                !projection
                    .callback_bindings()
                    .values()
                    .any(|callback| callback.reference.binding == CAPTURED_EDIT_BINDING)
            );
            assert!(!document.elements().iter().any(|element| {
                matches!(
                    element.attributes.get("role").map(String::as_str),
                    Some("radio" | "radiogroup")
                )
            }));
        })
        .unwrap();
}

fn pending_radio(host: &ViewerHost) -> PendingCallback {
    let handle = host.document.as_ref().unwrap();
    let path = captured_radio_path(handle);
    assert!(handle.dispatch_action(Action::Activate { path }).unwrap());
    let (mut pending, _) = handle
        .try_reproject(&handle.stamp().unwrap(), |document| {
            Ok::<_, String>(document.drain_callbacks())
        })
        .unwrap();
    assert_eq!(pending.len(), 1);
    pending.pop().unwrap()
}

#[test]
#[ignore = "Assigned original LocalHost slot required; actual product document, no OS window"]
fn empty_owned_empty_removes_actual_radio_focus_accessibility_and_callback_targets()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        bind_product(host, rig, project.clone());
        let original = host.document.as_ref().unwrap().stamp().unwrap();
        let calls = requests(&rig.requests).len();
        reproject(host, false, |_| {}).unwrap();
        assert_no_rail(host);
        reproject(host, true, |_| {}).unwrap();
        let handle = host.document.as_ref().unwrap().clone();
        let path = captured_radio_path(&handle);
        let old_id = radio_dom_id(host);
        handle
            .try_reproject(&handle.stamp().unwrap(), |document| {
                document.focus_path(&path)?;
                let focused = document.renderer.document.get_focussed_node_id().unwrap();
                assert_eq!(focused.as_u64(), old_id);
                let tree = accessibility::tree(document);
                assert!(tree.nodes.iter().any(|(id, _)| id.0 == old_id));
                assert_eq!(tree.focus.0, old_id);
                Ok::<_, String>(())
            })
            .unwrap();
        reproject(host, false, |_| {}).unwrap();
        assert_no_rail(host);
        handle
            .try_with(|document| {
                assert_ne!(
                    document
                        .renderer
                        .document
                        .get_focussed_node_id()
                        .map(|id| id.as_u64()),
                    Some(old_id)
                );
                assert_ne!(document.session.focus.as_deref(), Some(path.as_str()));
                let tree = accessibility::tree(document);
                assert!(!tree.nodes.iter().any(|(id, _)| id.0 == old_id));
                assert_ne!(tree.focus.0, old_id);
            })
            .unwrap();
        let after = handle.stamp().unwrap();
        assert_eq!(after.host_id, original.host_id);
        assert_eq!(after.owner, original.owner);
        assert!(after.document_revision > original.document_revision);
        assert_eq!(requests(&rig.requests).len(), calls);
        assert_immutable(rig, &project);
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot required; actual product document, no OS window"]
fn repeated_visible_projection_preserves_actual_rail_instance_and_radio_dom_identity()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        bind_product(host, rig, project.clone());
        let original = host.document.as_ref().unwrap().stamp().unwrap();
        let instance = rail_instance(host);
        let radio = radio_dom_id(host);
        let calls = requests(&rig.requests).len();
        for _ in 0..3 {
            reproject(host, true, |_| {}).unwrap();
            assert_eq!(rail_instance(host), instance);
            assert_eq!(radio_dom_id(host), radio);
            let after = host.document.as_ref().unwrap().stamp().unwrap();
            assert_eq!(after.host_id, original.host_id);
            assert_eq!(after.owner, original.owner);
        }
        assert_eq!(requests(&rig.requests).len(), calls);
        assert_immutable(rig, &project);
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot required; actual product callbacks and SDK, no OS window"]
fn drained_rail_callback_stays_retired_after_removal_and_remount_while_new_item_can_edit()
-> Result<(), Box<GuardedFailure>> {
    let mut acknowledged = Vec::new();
    let result = with_rig(|host, rig| {
        let project = seed(&rig.client);
        bind_product(host, rig, project.clone());
        let stale = pending_radio(host);
        let calls = requests(&rig.requests).len();
        reproject(host, false, |_| {}).unwrap();
        assert_no_rail(host);
        let handle = host.document.as_ref().unwrap().clone();
        assert!(matches!(
            host.admit_callback(&handle.stamp().unwrap(), &stale),
            IntentAdmission::Rejected(_)
        ));
        assert!(host.card_selection_queue().unwrap().borrow().is_empty());
        assert_eq!(requests(&rig.requests).len(), calls);
        reproject(host, true, |_| {}).unwrap();
        assert!(matches!(
            host.admit_callback(&handle.stamp().unwrap(), &stale),
            IntentAdmission::Rejected(_)
        ));
        assert!(host.card_selection_queue().unwrap().borrow().is_empty());
        assert_eq!(requests(&rig.requests).len(), calls);
        open(host);
        let initial_revision = host
            .capture_adapter_mut()
            .unwrap()
            .display()
            .state
            .document
            .draft_revision
            .0;
        text(host, "Fresh callback after remount");
        click(host, "onSave");
        ready(host);
        click(host, "onCaptureConsume");
        acknowledged.extend(host.take_captured_surface_acknowledgements().unwrap());
        assert_eq!(acknowledged.len(), 1);
        assert_eq!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .state
                .document
                .draft_revision
                .0,
            initial_revision
                .checked_add(1)
                .expect("draft revision increment")
        );
        let saved = host
            .capture_adapter_mut()
            .unwrap()
            .display()
            .state
            .document
            .draft
            .clone()
            .unwrap();
        let ProposalPayload::Cards { cards } = saved else {
            panic!("saved Card payload")
        };
        assert_eq!(cards[0].front, "Fresh callback after remount");
        assert_immutable(rig, &project);
    });
    if result.is_ok() {
        assert!(acknowledged[0].terminal.as_ref().unwrap().response.ok);
    }
    result
}

#[test]
#[ignore = "Assigned original LocalHost slot required; actual atomic preflight and SDK attachment"]
fn failed_slot_and_callback_preflight_keep_last_good_frame_export_and_sdk_owner()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        bind_product(host, rig, project.clone());
        let handle = host.document.as_ref().unwrap().clone();
        let before = handle.stamp().unwrap();
        let instance = rail_instance(host);
        let radio = radio_dom_id(host);
        let path = callback_path(&handle, "onReadStatus");
        handle.dispatch_action(Action::Activate { path }).unwrap();
        let (mut pending, _) = handle
            .try_reproject(&handle.stamp().unwrap(), |document| {
                Ok::<_, String>(document.drain_callbacks())
            })
            .unwrap();
        assert_eq!(pending.len(), 1);
        let old_read = pending.pop().unwrap();
        let last_good = handle.stamp().unwrap();
        let component = handle
            .try_with(|document| document.module_projection().unwrap().component.clone())
            .unwrap();
        let calls = requests(&rig.requests).len();
        let missing = reproject(host, true, |props| {
            props.remove("capturedCardsLabel");
        })
        .unwrap_err();
        assert!(missing.contains("Missing captured choice prop"));
        let wrong_type = reproject(host, true, |props| {
            props.insert("capturedCardsLabel".into(), Value::Bool(false));
        })
        .unwrap_err();
        assert!(wrong_type.contains("Invalid captured choice prop"));
        let unexported = reproject(host, true, |props| {
            props.insert(
                "onCapturedCardEdit".into(),
                Value::CallbackReference(CallbackReference {
                    binding: "unexportedCaptureChoice".into(),
                    signature: CallbackSignature::Text,
                }),
            );
        })
        .unwrap_err();
        assert!(unexported.contains("callback reference is not exported"));
        // The callback drain itself is an accepted host transaction; failed transactions follow it.
        let after = handle.stamp().unwrap();
        assert_eq!(after, last_good);
        assert_eq!(after.host_id, before.host_id);
        assert_eq!(rail_instance(host), instance);
        assert_eq!(radio_dom_id(host), radio);
        assert!(
            handle
                .try_with(|document| document
                    .module_projection()
                    .unwrap()
                    .component
                    .semantic_eq(&component))
                .unwrap()
        );
        assert!(
            host.callbacks
                .borrow()
                .original_execution()
                .attach()
                .is_err()
        );
        assert_eq!(requests(&rig.requests).len(), calls);
        assert!(matches!(
            host.admit_callback(&after, &old_read),
            IntentAdmission::Accepted
        ));
        host.after_dispatch(turn(host, HostTurnReason::Input))
            .unwrap();
        assert_immutable(rig, &project);
    })
}
