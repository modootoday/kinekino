use super::*;
use crate::application::sdk_execution::ExecutionBudget;
use crate::{
    application::api::PlayerSdk,
    assembly::meari::{
        native_execution_startup::prepare_native_execution,
        viewer_cleanup::{ViewerCleanupStatus, ViewerNativeState},
    },
    infrastructure::local_host::TransportOptions,
};
use meari_native::{
    application::{WindowOptions, window_lifecycle::LifecycleLimits},
    domain::Viewport,
    infrastructure::native_host::HostScheduleLimits,
};
use std::sync::Arc;

#[test]
#[ignore = "Requires an explicit pipe and Native-loop authority slot; no OS view is run"]
fn document_failure_retains_original_owners_through_budget_and_explicit_cleanup_resume() {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let endpoint = LocalHost::start(client.clone(), TransportOptions::default()).unwrap();
    let schedule =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 100).unwrap();
    let execution = prepare_native_execution(
        client.clone(),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        schedule,
    )
    .unwrap();
    let input = ViewerLaunchInput {
        client: client.clone(),
        endpoint,
        execution,
        labels: StartLabels {
            connection: "관측 없음".into(),
            time: "관측 없음".into(),
        },
        document: DocumentOptions {
            namespace: "viewer-failure-test".into(),
            epoch: 1,
            source_generation: 0,
            viewport: Viewport {
                width: 320,
                height: 180,
                dark: true,
            },
        },
        host: ShellHostConfig {
            options: WindowOptions::default(),
            schedule,
            children: LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        },
    };
    let owners = match prepare_viewer_launch(input) {
        Err(owners) => *owners,
        Ok(_) => panic!("invalid document must retain input"),
    };
    let ViewerOwners::DocumentFailure { failure, .. } = &owners else {
        panic!("original typed failure")
    };
    assert!(Arc::ptr_eq(&client, &failure.inputs.client));
    assert!(Arc::ptr_eq(&client, failure.inputs.host_owner.client()));
    let outcome = finish_viewer(
        owners,
        ViewerCleanupLimits::new(Duration::from_secs(2), 1, Duration::from_millis(2)).unwrap(),
    );
    assert!(matches!(
        outcome.owners,
        ViewerOwners::DocumentFailure { .. }
    ));
    // Same carrier (including any PendingLocalHost) is resumed; no new host/worker/loop.
    let resumed = finish_viewer(
        outcome.owners,
        ViewerCleanupLimits::new(Duration::from_secs(2), 500, Duration::from_millis(2)).unwrap(),
    );
    assert_eq!(
        resumed.status,
        ViewerCleanupStatus::WorkersQuiescent(ViewerNativeState::NotRun)
    );
    assert!(resumed.evidence.error.is_none());
    assert!(matches!(
        resumed.owners,
        ViewerOwners::DocumentFailure { .. }
    ));
}
