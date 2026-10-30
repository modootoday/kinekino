//! Typed existing SDK operations on the original execution attachment; no browser implementation.
use crate::{
    application::{
        api::{ApiError, RequestContext},
        catalog::{CatalogResult, ReadCatalogResult},
        episode_resolve::{EpisodeResolutionInspection, InspectEpisodeResolution},
        jobs::{CancelJob, CancelledJob, GetJob, JobRecord, SubmitJob, SubmittedJob},
        playback_session::{OpenEpisodePlayback, OpenedEpisodePlayback},
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

pub(crate) const COMMANDS: [&str; 7] = [
    "jobs.submit",
    "jobs.get",
    "jobs.cancel",
    "catalog.search",
    "episodes.list",
    "episodes.inspect_resolution",
    "playback.open",
];
pub(crate) fn manifest(manifest: &mut PluginManifest) {
    for command in COMMANDS {
        let hash = |role: &[u8]| {
            ContentHash::of(
                &[
                    command.as_bytes(),
                    role,
                    include_bytes!("codec.rs"),
                    include_bytes!("contracts.rs"),
                    include_bytes!("model.rs"),
                    include_bytes!("progress.rs"),
                ]
                .concat(),
            )
        };
        manifest.capabilities.insert(
            command.into(),
            CapabilityDescriptor {
                targets: BTreeSet::from([TargetKind::Native]),
                request_schema: hash(b"typed-request"),
                reply_schema: hash(b"typed-reply"),
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
            include_bytes!("codec.rs"),
            include_bytes!("mod.rs"),
        ]
        .concat(),
    );
}
pub(crate) fn bindings(
    attachment: Arc<ExecutionAttachment>,
    context: RequestContext,
    manifest: &PluginManifest,
) -> BTreeMap<String, Box<dyn AsyncSdkBinding>> {
    let context = Rc::new(
        move |ticket: &PluginTicket| -> Result<RequestContext, ApiError> {
            let mut scoped =
                crate::assembly::meari::request_context::scoped_context(&context, ticket)?;
            if ticket.capability() == "jobs.submit" {
                scoped.idempotency_key = Some(scoped.request_id.clone());
            }
            Ok(scoped)
        },
    );
    macro_rules! bind {
        ($command:literal, $input:ty, $output:ty) => {{
            let descriptor = manifest.capabilities[$command].clone();
            async_binding(
                descriptor.clone(),
                SdkPort::<$input, $output, _>::new(
                    attachment.clone(),
                    $command,
                    descriptor,
                    context.clone(),
                    256 * 1024,
                ),
            )
        }};
    }
    BTreeMap::from([
        (
            "jobs.submit".into(),
            bind!("jobs.submit", SubmitJob, SubmittedJob),
        ),
        ("jobs.get".into(), bind!("jobs.get", GetJob, JobRecord)),
        (
            "jobs.cancel".into(),
            bind!("jobs.cancel", CancelJob, CancelledJob),
        ),
        (
            "catalog.search".into(),
            bind!("catalog.search", ReadCatalogResult, CatalogResult),
        ),
        (
            "episodes.list".into(),
            bind!("episodes.list", ReadCatalogResult, CatalogResult),
        ),
        (
            "episodes.inspect_resolution".into(),
            bind!(
                "episodes.inspect_resolution",
                InspectEpisodeResolution,
                EpisodeResolutionInspection
            ),
        ),
        (
            "playback.open".into(),
            bind!("playback.open", OpenEpisodePlayback, OpenedEpisodePlayback),
        ),
    ])
}
