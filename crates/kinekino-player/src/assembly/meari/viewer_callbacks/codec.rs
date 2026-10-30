//! Read-only DTOs; opaque capability sections are never inferred UI authority.
use crate::{
    application::{
        api::{ApiError, RequestContext},
        playback_session::EpisodePlaybackObservation,
        playback_status::PlaybackStatus,
        proposals::{
            InspectProposal, StoredProposal,
            list::{ListProposals, ProposalListPage},
        },
        sdk_execution::ExecutionAttachment,
    },
    assembly::meari::async_sdk_port::SdkPort,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{
    AsyncCodecBudget, AsyncSdkPump, TypedAsyncSdkPlugin, async_binding,
};
use meari::{
    application::plugins::*,
    domain::build::{ContentHash, TargetKind},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    sync::Arc,
};

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadInput {}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeActionInput {
    engine_instance_id: crate::domain::api_values::DecimalU64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeSeekInput {
    engine_instance_id: crate::domain::api_values::DecimalU64,
    position_us: crate::domain::api_values::DecimalU64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeVolumeInput {
    engine_instance_id: crate::domain::api_values::DecimalU64,
    volume_percent: f64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandSupport {
    pub command: String,
    pub status: String,
    #[serde(flatten)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilitiesReply {
    pub commands: Vec<CommandSupport>,
    #[serde(flatten)]
    pub sections: BTreeMap<String, serde_json::Value>,
}
pub const READS: [(&str, &str); 3] = [
    ("onReadCapabilities", "capabilities"),
    ("onReadEngine", "playback.engine"),
    ("onReadStatus", "playback.status"),
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReadProfile {
    #[default]
    Default,
    ProposalChooser,
}
pub fn manifest_for(profile: ReadProfile) -> PluginManifest {
    let mut result = manifest();
    if profile == ReadProfile::Default {
        return result;
    }
    for command in ["proposals.list", "proposals.inspect"] {
        let hash = |role: &[u8]| {
            ContentHash::of(
                &[
                    command.as_bytes(),
                    role,
                    include_bytes!("chooser.rs").as_slice(),
                    include_bytes!("../../../application/proposals/list.rs"),
                    include_bytes!("../../../application/proposals/mod.rs"),
                    include_bytes!("../../../domain/proposals/mod.rs"),
                    include_bytes!("../../../domain/proposals/stored.rs"),
                    include_bytes!("../../../domain/proposals/document_source.rs"),
                ]
                .concat(),
            )
        };
        result.capabilities.insert(
            command.into(),
            CapabilityDescriptor {
                targets: BTreeSet::from([TargetKind::Native]),
                request_schema: hash(b"request"),
                reply_schema: hash(b"reply"),
                error_schema: Some(ContentHash::of(include_bytes!(
                    "../../../../../kinekino-media-contracts/src/errors.rs"
                ))),
                event_schema: None,
            },
        );
    }
    result.contract = ContentHash::of(b"viewer-read-chooser-v1:reads3,proposals.list,proposals.inspect;explicit-terminal-acceptance");
    result.provider = ContentHash::of(
        &[
            result.provider.as_str().as_bytes(),
            include_bytes!("chooser.rs").as_slice(),
            include_bytes!("../../../application/proposals/list.rs"),
            include_bytes!("../../../domain/proposals/stored.rs"),
        ]
        .concat(),
    );
    result
}
pub fn manifest() -> PluginManifest {
    let capabilities = READS
        .iter()
        .chain(super::native_playback::CONTROLS.iter())
        .map(|(_, name)| {
            let hash = |role: &str| {
                ContentHash::of(
                    &[
                        name.as_bytes(),
                        b"\0",
                        role.as_bytes(),
                        b"\0",
                        include_bytes!("codec.rs"),
                        include_bytes!("native_playback.rs"),
                        include_bytes!("../../../domain/playback_control.rs"),
                        include_bytes!("../../../application/playback_status/mod.rs"),
                        include_bytes!("../../../application/playback_session/types.rs"),
                    ]
                    .concat(),
                )
            };
            (
                (*name).into(),
                CapabilityDescriptor {
                    targets: BTreeSet::from([TargetKind::Native]),
                    request_schema: hash("empty-read-request"),
                    reply_schema: hash("typed-reply"),
                    error_schema: Some(ContentHash::of(include_bytes!(
                        "../../../../../kinekino-media-contracts/src/errors.rs"
                    ))),
                    event_schema: None,
                },
            )
        })
        .collect();
    let mut manifest = PluginManifest { api_version: PLUGIN_API_VERSION, id: "com.kinekino.player.viewer-read".into(), version: env!("CARGO_PKG_VERSION").into(),
        contract: ContentHash::of(b"viewer-native-v2:original-session-reads-and-instance-controls;explicit-terminal-acceptance"),
        provider: ContentHash::of(&[include_bytes!("mod.rs").as_slice(), include_bytes!("codec.rs"), include_bytes!("lifecycle.rs"), include_bytes!("../async_sdk_port.rs"),
            include_bytes!("../viewer_read_projection/mod.rs"),
            include_bytes!("../viewer_read_actions/mod.rs"),
            include_bytes!("../viewer_host/mod.rs"),
            include_bytes!("../viewer_native_launch.rs"),
            include_bytes!("../viewer_native_launch/no_media.rs"),
            include_bytes!("../viewer_cleanup.rs"),
            include_bytes!("../../../presentation/web_ui_adapter/player_shell/read_observation.rs"),
            include_bytes!("../../../presentation/web_ui_adapter/player_shell/document.rs"),
            include_bytes!("../../../presentation/web_ui_adapter/player_shell/document/video.rs"),
            include_bytes!("../../../presentation/web_ui/viewing/read-panel.tsx"),
            include_bytes!("../../../presentation/web_ui/viewing/player.tsx"),
            include_bytes!("../../../presentation/web_ui/viewing/source-graph.json"),
            include_bytes!("../../../presentation/web_ui/styles/viewing.css"),
        ].concat()),
        capabilities, assets: BTreeMap::new() };
    super::catalog::codec::manifest(&mut manifest);
    super::episode_transition::codec::manifest(&mut manifest);
    manifest
}
pub(crate) fn sdk_error(command: &str, error: &PluginError) -> Option<ApiError> {
    let manifest = manifest();
    let descriptor = manifest.capabilities.get(command)?;
    error
        .details
        .as_ref()?
        .decode(descriptor.error_schema.as_ref()?)
        .ok()
}
pub(super) fn adapter(
    attachment: Arc<ExecutionAttachment>,
    context: RequestContext,
) -> Result<(TypedAsyncSdkPlugin, AsyncSdkPump), PluginError> {
    adapter_for(attachment, context, ReadProfile::Default)
}
pub(super) fn adapter_for(
    attachment: Arc<ExecutionAttachment>,
    context: RequestContext,
    profile: ReadProfile,
) -> Result<(TypedAsyncSdkPlugin, AsyncSdkPump), PluginError> {
    let manifest = manifest_for(profile);
    let catalog_context = context.clone();
    let transition_context = context.clone();
    let context = Rc::new(
        move |ticket: &PluginTicket| -> Result<RequestContext, ApiError> {
            crate::assembly::meari::request_context::scoped_context(&context, ticket)
        },
    );
    let limit = 256 * 1024;
    macro_rules! bind {
        ($name:literal,$input:ty,$output:ty) => {{
            let descriptor = manifest.capabilities[$name].clone();
            async_binding(
                descriptor.clone(),
                SdkPort::<$input, $output, _>::new(
                    attachment.clone(),
                    $name,
                    descriptor,
                    context.clone(),
                    limit,
                ),
            )
        }};
    }
    let mut bindings = BTreeMap::from([
        (
            "capabilities".into(),
            bind!("capabilities", ReadInput, CapabilitiesReply),
        ),
        (
            "playback.engine".into(),
            bind!("playback.engine", ReadInput, EpisodePlaybackObservation),
        ),
        (
            "playback.status".into(),
            bind!("playback.status", ReadInput, PlaybackStatus),
        ),
    ]);
    bindings.insert(
        "playback.play".into(),
        bind!(
            "playback.play",
            NativeActionInput,
            crate::application::playback_control::ControlReceipt
        ),
    );
    bindings.insert(
        "playback.pause".into(),
        bind!(
            "playback.pause",
            NativeActionInput,
            crate::application::playback_control::ControlReceipt
        ),
    );
    bindings.insert(
        "playback.seek".into(),
        bind!(
            "playback.seek",
            NativeSeekInput,
            crate::application::playback_control::ControlReceipt
        ),
    );
    bindings.insert(
        "playback.set_volume".into(),
        bind!(
            "playback.set_volume",
            NativeVolumeInput,
            crate::application::playback_control::ControlReceipt
        ),
    );
    bindings.extend(super::catalog::codec::bindings(
        attachment.clone(),
        catalog_context,
        &manifest,
    ));
    bindings.extend(super::episode_transition::codec::bindings(
        attachment.clone(),
        transition_context,
        &manifest,
    ));
    if profile == ReadProfile::ProposalChooser {
        bindings.insert(
            "proposals.list".into(),
            bind!("proposals.list", ListProposals, ProposalListPage),
        );
        bindings.insert(
            "proposals.inspect".into(),
            bind!("proposals.inspect", InspectProposal, StoredProposal),
        );
    }
    TypedAsyncSdkPlugin::new(
        manifest,
        bindings,
        AsyncCodecBudget {
            max_pending: 8,
            max_terminal_wire_bytes: limit,
            max_total_codec_wire_bytes: 8 * limit * 4,
        },
    )
}
