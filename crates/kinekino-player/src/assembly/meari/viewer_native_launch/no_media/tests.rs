//! Actual CPU document/controller composition with a receiver port; no Viewer endpoint or Native loop.
mod end_to_end;
use super::*;
use crate::{
    application::api::{ApplicationClientHandle, PlayerSdk},
    assembly::meari::{
        player_shell_document::{DocumentOptions, mount_prepared_shell},
        player_shell_startup::{ShellInputs, prepare_shell},
        quarantine_surface::frame_delivery::{
            FrameOwnedResult, FrameResultState, reflection_channel_for_test,
        },
    },
    presentation::web_ui_adapter::player_shell::document::{StartLabels, video::ViewingVideoId},
};
use meari::{application::host_callbacks::PendingCallback, domain::build::BuildPath};
use meari_native::{
    application::{WindowOptions, window_lifecycle::LifecycleLimits},
    domain::Viewport,
    infrastructure::native_host::*,
};
use std::{collections::BTreeMap, rc::Rc, sync::Arc, time::Duration};

struct CpuHost {
    identity: Rc<()>,
    receiver: Option<FrameReceiver>,
}
impl ReceiverOwner for CpuHost {
    fn install_receiver(&mut self, receiver: FrameReceiver) -> Result<(), FrameReceiver> {
        if self.receiver.is_some() {
            return Err(receiver);
        }
        self.receiver = Some(receiver);
        Ok(())
    }
}
impl NativeWindowHost for CpuHost {
    type Error = std::io::Error;
    type CloseCheckpoint = ();
    fn admit_callback(
        &mut self,
        _: &HostStamp,
        _: &PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        IntentAdmission::Capacity
    }
    fn after_dispatch(&mut self, _: HostTurn) -> Result<HostEffects, Self::Error> {
        Ok(HostEffects::default())
    }
    fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), Self::Error> {
        CloseDecision::HostCleanupComplete { checkpoint: () }
    }
}
fn options() -> NoMediaFrameOptions {
    NoMediaFrameOptions {
        request_id: NonZeroU64::new(41).unwrap(),
        expires_at: Instant::now() + Duration::from_secs(30),
        frame_budget: NonZeroUsize::new(3).unwrap(),
    }
}
fn config() -> ShellHostConfig {
    ShellHostConfig {
        options: WindowOptions::default(),
        children: LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        schedule: HostScheduleLimits::new(
            Duration::from_millis(2),
            Duration::from_millis(250),
            100,
        )
        .unwrap(),
    }
}
fn document(markup: Option<&str>) -> PreparedShellDocument<CpuHost> {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let prepared = prepare_shell(
        ShellInputs {
            client,
            host_owner: CpuHost {
                identity: Rc::new(()),
                receiver: None,
            },
        },
        StartLabels {
            connection: "관측 없음".into(),
            time: "관측 없음".into(),
        },
    );
    let mut prepared = match prepared {
        Ok(prepared) => prepared,
        Err(_) => panic!("fixed preparation"),
    };
    if let Some(markup) = markup {
        let path: BuildPath = "viewing/no-media-fixture.tsx"
            .to_owned()
            .try_into()
            .unwrap();
        let inventory = BTreeMap::from([(
            path.clone(),
            format!("type Props={{}};export function App(props:Props){{return {markup};}}"),
        )]);
        prepared.request.document.graph = meari::parse_module_graph(&path, &inventory).unwrap();
        prepared.request.document.inventory = inventory;
        prepared.request.document.props = BTreeMap::new();
    }
    mount_prepared_shell(
        prepared,
        DocumentOptions {
            namespace: "no-media-v2b-cpu".into(),
            epoch: 1,
            source_generation: 1,
            viewport: Viewport {
                width: 320,
                height: 180,
                dark: true,
            },
        },
    )
    .unwrap_or_else(|failure| panic!("{}", failure.diagnostic))
}
fn completion(stamp: HostStamp) -> HostTurn {
    HostTurn {
        stamp,
        reason: HostTurnReason::Completion,
        accepted_callbacks: 0,
        rejected_callbacks: 0,
        remaining_callbacks: 0,
    }
}

#[test]
fn request_uses_current_original_document_after_reprojection_without_renewing_limits() {
    let document = document(None);
    let historical = document.inspect_viewing_video().unwrap();
    document
        .document
        .try_reproject(historical.origin(), |_| Ok::<_, std::io::Error>(()))
        .unwrap();
    let options = options();
    let request = request_for(&document, options).unwrap();
    assert_eq!(request.origin, document.document.stamp().unwrap());
    assert_ne!(
        request.origin.document_revision,
        historical.origin().document_revision
    );
    assert_eq!(request.origin.host_id, historical.origin().host_id);
    assert_eq!(request.video, ViewingVideoId::Primary.as_str());
    assert_eq!(request.id, options.request_id);
    assert_eq!(request.expires_at, options.expires_at);
    assert_eq!(request.frame_budget, options.frame_budget);
}

#[test]
fn same_document_installs_one_receiver_and_consumer_without_startup_or_new_host() {
    let document = document(None);
    let handle = document.document.clone();
    let identity = document.inputs.host_owner.identity.clone();
    let client = document.inputs.client.clone();
    let stamp = handle.stamp().unwrap();
    let controller = install_frames(document, config(), options())
        .unwrap_or_else(|failure| panic!("{:?}", failure.error));
    let host = controller.pending_owner().unwrap();
    assert!(Rc::ptr_eq(&identity, &host.identity));
    assert!(Arc::ptr_eq(&client, &controller.client));
    assert!(host.receiver.is_some());
    assert!(controller.runtime.is_none());
    assert!(controller.wake.is_none());
    assert!(controller.attach_failure.is_none());
    assert!(controller.frame_setup_error().is_none());
    assert_eq!(handle.stamp().unwrap(), stamp);
}

#[test]
fn failed_inspection_retains_original_prepared_bundle_without_minting_frame_parts() {
    let document = document(Some("<div/>"));
    let handle = document.document.clone();
    let client = document.inputs.client.clone();
    let identity = document.inputs.host_owner.identity.clone();
    let generation = document.ticket.generation();
    let styles = document.style_hashes.clone();
    let options = options();
    let failure = match install_frames(document, config(), options) {
        Err(failure) => failure,
        Ok(_) => panic!("missing video"),
    };
    assert_eq!(
        failure.error,
        NoMediaSetupError::Video(ViewingVideoError::Missing)
    );
    assert_eq!(failure.request, options);
    assert!(matches!(failure.uninstalled, UninstalledFrames::None));
    let SetupOwners::Prepared(prepared) = failure.owners else {
        panic!("original preparation")
    };
    assert_eq!(prepared.document.document.id(), handle.id());
    assert_eq!(
        prepared.document.document.stamp().unwrap(),
        handle.stamp().unwrap()
    );
    assert!(Arc::ptr_eq(&client, &prepared.document.inputs.client));
    assert!(Rc::ptr_eq(
        &identity,
        &prepared.document.inputs.host_owner.identity
    ));
    assert_eq!(prepared.document.ticket.generation(), generation);
    assert_eq!(prepared.document.style_hashes, styles);
    assert!(prepared.document.inputs.host_owner.receiver.is_none());
}

#[test]
fn occupied_receiver_keeps_original_reader_and_returns_both_new_uninstalled_actors() {
    let mut document = document(None);
    let stamp = document.document.stamp().unwrap();
    let id = NonZeroU64::new(7).unwrap();
    let options = options();
    let (receiver, publish) = reflection_channel_for_test(id, &stamp, options.expires_at);
    document.inputs.host_owner.receiver = Some(receiver);
    let failure = match install_frames(document, config(), options) {
        Err(failure) => failure,
        Ok(_) => panic!("occupied reader"),
    };
    assert_eq!(failure.error, NoMediaSetupError::ReceiverOccupied);
    assert_eq!(failure.request, options);
    let SetupOwners::Prepared(prepared) = failure.owners else {
        panic!("prepared host retained")
    };
    let UninstalledFrames::Pair(pair) = failure.uninstalled else {
        panic!("same rejected pair")
    };
    assert!(pair.consumer.first_error().is_none());
    assert!(pair.consumer.registration_failure().is_none());
    assert!(
        pair.receiver
            .receive(&completion(stamp.clone()))
            .unwrap()
            .result
            .is_none()
    );
    publish(
        FrameOwnedResult {
            request: id,
            observed: stamp.clone(),
            window: None,
            state: FrameResultState::Pending,
        },
        true,
    );
    let update = prepared
        .document
        .inputs
        .host_owner
        .receiver
        .as_ref()
        .unwrap()
        .receive(&completion(stamp))
        .unwrap();
    assert_eq!(update.result.unwrap().request, id);
    assert!(update.redraw);
}

#[test]
fn occupied_consumer_returns_the_same_box_and_original_controller_with_installed_receiver() {
    let document = document(None);
    let request = options();
    let frame_request = request_for(&document, request).unwrap();
    let identity = document.inputs.host_owner.identity.clone();
    let controller = install_frames(document, config(), request)
        .unwrap_or_else(|failure| panic!("{:?}", failure.error));
    let (consumer, _spare_receiver) = NoMediaFrameConsumer::new(frame_request);
    let consumer: Box<dyn ShellFrameConsumer<QuarantineChild>> = Box::new(consumer);
    let original = std::ptr::from_ref(consumer.as_ref());
    let failure = match complete_installation(controller, consumer, request) {
        Err(failure) => failure,
        Ok(_) => panic!("occupied consumer"),
    };
    assert_eq!(failure.error, NoMediaSetupError::ConsumerRejected);
    assert_eq!(failure.request, request);
    let UninstalledFrames::Consumer(consumer) = failure.uninstalled else {
        panic!("original rejected consumer")
    };
    assert!(std::ptr::eq(
        original,
        std::ptr::from_ref(consumer.as_ref())
    ));
    let SetupOwners::Controller(controller) = failure.owners else {
        panic!("same controller")
    };
    assert!(Rc::ptr_eq(
        &identity,
        &controller.pending_owner().unwrap().identity
    ));
    assert!(controller.pending_owner().unwrap().receiver.is_some());
    assert!(controller.runtime.is_none());
}

#[test]
fn explicit_cleanup_conversion_keeps_same_pending_host_and_rejected_actor_retention() {
    let mut document = document(None);
    let identity = document.inputs.host_owner.identity.clone();
    let options = options();
    let stamp = document.document.stamp().unwrap();
    let (receiver, _publish) =
        reflection_channel_for_test(NonZeroU64::new(7).unwrap(), &stamp, options.expires_at);
    document.inputs.host_owner.receiver = Some(receiver);
    let failure = match install_frames(document, config(), options) {
        Err(failure) => failure,
        Ok(_) => panic!("occupied reader"),
    };
    let (controller, retention) = (*failure).into_controller();
    assert!(Rc::ptr_eq(
        &identity,
        &controller.pending_owner().unwrap().identity
    ));
    assert_eq!(retention.request(), options);
    assert_eq!(
        retention.error(),
        Some(&NoMediaSetupError::ReceiverOccupied)
    );
    let UninstalledFrames::Pair(pair) = retention.uninstalled else {
        panic!("keep rejected pair across cleanup transition")
    };
    assert!(pair.consumer.registration_failure().is_none());
    assert!(
        pair.receiver
            .receive(&completion(stamp))
            .unwrap()
            .result
            .is_none()
    );
    assert!(controller.pending_owner().unwrap().receiver.is_some());
    assert!(controller.runtime.is_none());
    assert!(controller.wake.is_none());
}
