use super::*;

#[test]
fn released_read_owner_reexports_on_original_scope_and_rejects_previous_callback() {
    let mut fixture = route_fixture();
    let (_old_export, old_read) = local_input(&fixture.owner.borrow(), "onReadCapabilities");
    assert!(
        fixture
            .owner
            .borrow_mut()
            .resume_released_route(&fixture.scope)
            .is_err()
    );
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    let mut foreign = fixture.scope.clone();
    foreign.source_generation += 1;
    assert_eq!(
        fixture
            .owner
            .borrow_mut()
            .resume_released_route(&foreign)
            .err()
            .unwrap()
            .code,
        PluginErrorCode::Stale
    );
    assert!(fixture.owner.borrow().is_released());
    let bindings = fixture
        .owner
        .borrow_mut()
        .resume_released_route(&fixture.scope)
        .unwrap();
    // Export reattached the same worker but the document is not committed until here.
    assert!(fixture.execution.handle().attach().is_err());
    assert!(!fixture.owner.borrow().is_released());
    let props = BTreeMap::from([(
        "action".into(),
        Value::CallbackReference(CallbackReference {
            binding: "onReadCapabilities".into(),
            signature: CallbackSignature::Action,
        }),
    )]);
    fixture
        ._document
        .update_module_props_with_callbacks(props, bindings)
        .unwrap();
    assert_eq!(fixture._document.callback_scope(), Some(&fixture.scope));
    assert!(
        fixture
            .owner
            .borrow()
            .dispatch(&fixture.scope, &old_read)
            .is_err()
    );
    let (_new_export, new_read) = local_input(&fixture.owner.borrow(), "onReadCapabilities");
    let submitted = fixture
        .owner
        .borrow()
        .dispatch(&fixture.scope, &new_read)
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let accepted = fixture
            .owner
            .borrow_mut()
            .accept_terminal(&submitted.ticket)
            .unwrap();
        if let Some(terminal) = accepted {
            terminal.owner_ack.unwrap();
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    let attachment = fixture.execution.handle().attach().unwrap();
    attachment.release().unwrap();
    close(&fixture.execution);
}

#[test]
fn failed_read_reattach_retains_cleanup_owner_and_can_retry_after_card_detaches() {
    let fixture = route_fixture();
    fixture
        .owner
        .borrow_mut()
        .prepare_route_release(&fixture.scope)
        .unwrap();
    let card_attachment = fixture.execution.handle().attach().unwrap();
    assert!(
        fixture
            .owner
            .borrow_mut()
            .resume_released_route(&fixture.scope)
            .is_err()
    );
    assert_eq!(fixture.owner.borrow().scope(), Some(&fixture.scope));
    assert!(!fixture.owner.borrow().is_released());
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    // Releasing the failed read setup must not release the Card's attachment.
    assert!(fixture.execution.handle().attach().is_err());
    card_attachment.release().unwrap();
    fixture
        .owner
        .borrow_mut()
        .resume_released_route(&fixture.scope)
        .unwrap();
    assert!(fixture.execution.handle().attach().is_err());
    fixture
        .owner
        .borrow_mut()
        .prepare_route_release(&fixture.scope)
        .unwrap();
    close(&fixture.execution);
}
