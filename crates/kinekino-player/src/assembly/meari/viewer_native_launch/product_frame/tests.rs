//! Actual product source preparation and CPU document mount, without endpoint or OS startup.
use super::*;
use crate::{
    application::{
        api::{ApplicationClientHandle, PlayerSdk, RequestContext},
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
        },
    },
    assembly::meari::{
        player_shell_document::{
            DocumentOptions, DocumentStage, mount_prepared_product_shell_with_callbacks,
        },
        player_shell_startup::{ShellInputs, prepare_shell},
        viewer_callbacks::{ViewerCallbacks, export_callback_bindings},
    },
    presentation::web_ui_adapter::player_shell::document::{
        StartLabels, enable_read_panel, read_props, video::ViewingVideoId,
    },
};
use meari_native::domain::Viewport;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), crate::application::api::ApiError> {
        Ok(())
    }
}
fn labels() -> StartLabels {
    StartLabels {
        connection: "관측 없음".into(),
        time: "관측 없음".into(),
    }
}
fn options() -> DocumentOptions {
    DocumentOptions {
        namespace: "product-frame-profile".into(),
        epoch: 1,
        source_generation: 1,
        viewport: Viewport {
            width: 960,
            height: 640,
            dark: true,
        },
    }
}

#[test]
fn original_product_viewing_mount_has_both_inventories_and_keeps_supplied_owners() {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let execution = ExecutionOwner::start(
        client.clone(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let owner = Rc::new(());
    let mut prepared = prepare_shell(
        ShellInputs {
            client: client.clone(),
            host_owner: owner.clone(),
        },
        labels(),
    )
    .unwrap_or_else(|_| panic!("Bundled source catalog must prepare"));
    enable_read_panel(&mut prepared.request.document, read_props(&labels()))
        .unwrap_or_else(|_| panic!("Actual viewing source must prepare"));
    let frame = prepare(&prepared.request.document)
        .unwrap_or_else(|_| panic!("Both actual inventories must compose"));
    for path in [
        "viewing/read-panel.tsx",
        "studio/frame.tsx",
        "studio/cards/proposal-panel.tsx",
    ] {
        let path: meari::domain::build::BuildPath = path.to_owned().try_into().unwrap();
        assert!(frame.graph.modules.contains_key(&path));
    }
    let callbacks = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let mounted = mount_prepared_product_shell_with_callbacks(
        prepared,
        options(),
        &frame,
        export_callback_bindings(callbacks.clone()),
    )
    .unwrap_or_else(|_| panic!("Actual product frame must mount through the original constructor"));
    assert!(Arc::ptr_eq(&client, &mounted.inputs.client));
    assert!(Rc::ptr_eq(&owner, &mounted.inputs.host_owner));
    assert_eq!(
        mounted.inspect_viewing_video().unwrap().id(),
        ViewingVideoId::Primary
    );
    assert!(
        mounted
            .style_hashes
            .contains_key("crates/kinekino-player/src/presentation/web_ui/styles/viewing.css")
    );
    assert!(
        mounted
            .style_hashes
            .contains_key("crates/kinekino-player/src/presentation/web_ui/styles/studio.css")
    );
    assert!(execution.handle().attach().is_err());
    let before = mounted.document.stamp().unwrap();
    let export = callbacks.borrow().bindings().unwrap();
    mounted.document.try_reproject(&before, |document| {
        let props = frame.workspace_props(document,
            crate::presentation::web_ui_adapter::player_shell::document::product_frame::WorkspaceRoute::Viewing,
            std::num::NonZeroU64::new(1).unwrap(), read_props(&labels()))?;
        document.update_module_props_with_callbacks(props, export)
    }).unwrap();
    let after = mounted.document.stamp().unwrap();
    assert_eq!(after.owner, before.owner);
    assert_eq!(after.document_revision, before.document_revision + 1);
    assert!(
        callbacks
            .borrow_mut()
            .prepare_route_release(&after.owner)
            .unwrap()
            .is_none()
    );
    let next = execution.handle().attach().unwrap();
    next.release().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let ExecutionCloseProgress::Closed(receipt) = execution.poll_close().unwrap() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn rejected_product_configuration_returns_original_client_and_host_without_export() {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let owner = Rc::new(());
    let mut prepared = prepare_shell(
        ShellInputs {
            client: client.clone(),
            host_owner: owner.clone(),
        },
        labels(),
    )
    .unwrap_or_else(|_| panic!("Bundled source catalog must prepare"));
    enable_read_panel(&mut prepared.request.document, read_props(&labels()))
        .unwrap_or_else(|_| panic!("Actual viewing source must prepare"));
    let frame = prepare(&prepared.request.document)
        .unwrap_or_else(|_| panic!("Both inventories must compose"));
    let mut invalid = options();
    invalid.viewport.width = 0;
    let failure = match mount_prepared_product_shell_with_callbacks(
        prepared,
        invalid,
        &frame,
        Box::new(|_| panic!("Rejected configuration must not consume callback owners")),
    ) {
        Err(failure) => failure,
        Ok(_) => panic!("Zero viewport must be rejected"),
    };
    assert_eq!(failure.stage, DocumentStage::Configuration);
    assert!(Arc::ptr_eq(&client, &failure.inputs.client));
    assert!(Rc::ptr_eq(&owner, &failure.inputs.host_owner));
    assert!(failure.state.observed_route().is_none());
}
