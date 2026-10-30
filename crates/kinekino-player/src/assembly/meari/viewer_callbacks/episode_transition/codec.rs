use super::{port::StartPort, state::HandleInput};
use crate::{
    application::{
        api::{ApiError, RequestContext},
        playback_session::transition::{
            BeginAdjacentResolution, EpisodeDirection, ReleasedTransition, ReplacedEpisode,
            SubmittedTransition, TransitionObservation,
        },
        sdk_execution::ExecutionAttachment,
    },
    assembly::meari::async_sdk_port::SdkPort,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{AsyncSdkBinding, async_binding};
use meari::{
    application::plugins::*,
    domain::build::{ContentHash, TargetKind},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    sync::Arc,
};
pub(crate) const COMMANDS: [&str; 5] = [
    "playback.previous",
    "playback.next",
    "playback.transition.observe",
    "playback.transition.commit",
    "playback.transition.release",
];
pub(crate) fn manifest(manifest: &mut PluginManifest) {
    for command in COMMANDS {
        let hash = |role: &[u8]| {
            ContentHash::of(
                &[
                    command.as_bytes(),
                    role,
                    include_bytes!("port.rs"),
                    include_bytes!("state.rs"),
                    include_bytes!("../../../../application/api/client/episode_transition/mod.rs"),
                    include_bytes!("../../../../application/api/episode_transition_schema.rs"),
                ]
                .concat(),
            )
        };
        manifest.capabilities.insert(
            command.into(),
            CapabilityDescriptor {
                targets: BTreeSet::from([TargetKind::Native]),
                request_schema: hash(b"request"),
                reply_schema: hash(b"reply"),
                error_schema: Some(ContentHash::of(include_bytes!(
                    "../../../../../../kinekino-media-contracts/src/errors.rs"
                ))),
                event_schema: None,
            },
        );
    }
    manifest.provider = ContentHash::of(
        &[
            manifest.provider.as_str().as_bytes(),
            include_bytes!("port.rs"),
            include_bytes!("state.rs"),
        ]
        .concat(),
    );
}
pub(crate) fn bindings(
    execution: Arc<ExecutionAttachment>,
    context: RequestContext,
    manifest: &PluginManifest,
) -> BTreeMap<String, Box<dyn AsyncSdkBinding>> {
    let context = Rc::new(
        move |ticket: &PluginTicket| -> Result<RequestContext, ApiError> {
            crate::assembly::meari::request_context::scoped_context(&context, ticket)
        },
    );
    let mut bindings = BTreeMap::<String, Box<dyn AsyncSdkBinding>>::new();
    for (command, direction) in [
        ("playback.previous", EpisodeDirection::Previous),
        ("playback.next", EpisodeDirection::Next),
    ] {
        let descriptor = manifest.capabilities[command].clone();
        bindings.insert(
            command.into(),
            async_binding::<BeginAdjacentResolution, SubmittedTransition, _>(
                descriptor.clone(),
                StartPort::new(
                    execution.clone(),
                    command,
                    direction,
                    descriptor,
                    context.clone(),
                ),
            ),
        );
    }
    macro_rules! bind {
        ($command:literal, $output:ty) => {{
            let descriptor = manifest.capabilities[$command].clone();
            bindings.insert(
                $command.into(),
                async_binding(
                    descriptor.clone(),
                    SdkPort::<HandleInput, $output, _>::new(
                        execution.clone(),
                        $command,
                        descriptor,
                        context.clone(),
                        256 * 1024,
                    ),
                ),
            );
        }};
    }
    bind!("playback.transition.observe", TransitionObservation);
    bind!("playback.transition.commit", ReplacedEpisode);
    bind!("playback.transition.release", ReleasedTransition);
    bindings
}
