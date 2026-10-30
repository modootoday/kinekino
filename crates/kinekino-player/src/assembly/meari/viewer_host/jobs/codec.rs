//! Existing jobs.list typed port; no job execution or storage-root configuration.
use crate::{
    application::{
        api::{ApiError, RequestContext},
        jobs::{JobPage, ListJobs},
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
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    sync::Arc,
};

pub(super) const COMMAND: &str = "jobs.list";
pub(super) fn manifest() -> PluginManifest {
    let schema = |role: &str| {
        ContentHash::of(
            &[
                role.as_bytes(),
                include_bytes!("codec.rs"),
                include_bytes!("../../../../../../kinekino-jobs/src/application/api/mod.rs"),
                include_bytes!("../../../../../../kinekino-jobs/src/application/ports.rs"),
                include_bytes!("../../../../../../kinekino-jobs/src/domain/state.rs"),
            ]
            .concat(),
        )
    };
    PluginManifest {
        api_version: PLUGIN_API_VERSION,
        id: "com.kinekino.player.jobs-list-read".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        contract: ContentHash::of(b"jobs-list-read-v1:readonly;explicit-first-page;no-watch"),
        provider: ContentHash::of(
            &[
                include_bytes!("codec.rs").as_slice(),
                include_bytes!("consumer.rs"),
                include_bytes!("mod.rs"),
                include_bytes!("../../async_sdk_port.rs"),
                include_bytes!("../../../../presentation/web_ui_adapter/studio/jobs/list.rs"),
                include_bytes!("../../../../presentation/web_ui_adapter/studio/jobs/selection.rs"),
                include_bytes!("../../../../presentation/web_ui_adapter/studio/jobs/labels.rs"),
                include_bytes!("../../../../presentation/web_ui/studio/jobs/read-workspace.tsx"),
                include_bytes!("../../../../presentation/web_ui/studio/jobs/read-row.tsx"),
            ]
            .concat(),
        ),
        capabilities: BTreeMap::from([(
            COMMAND.into(),
            CapabilityDescriptor {
                targets: BTreeSet::from([TargetKind::Native]),
                request_schema: schema("ListJobs"),
                reply_schema: schema("JobPage"),
                error_schema: Some(ContentHash::of(include_bytes!(
                    "../../../../../../kinekino-media-contracts/src/errors.rs"
                ))),
                event_schema: None,
            },
        )]),
        assets: BTreeMap::new(),
    }
}
pub(super) fn adapter(
    attachment: Arc<ExecutionAttachment>,
    context: RequestContext,
) -> Result<(TypedAsyncSdkPlugin, AsyncSdkPump), PluginError> {
    let manifest = manifest();
    let descriptor = manifest.capabilities[COMMAND].clone();
    let context = Rc::new(
        move |ticket: &PluginTicket| -> Result<RequestContext, ApiError> {
            crate::assembly::meari::request_context::scoped_context(&context, ticket)
        },
    );
    let limit = 256 * 1024;
    let bindings = BTreeMap::from([(
        COMMAND.into(),
        async_binding(
            descriptor.clone(),
            SdkPort::<ListJobs, JobPage, _>::new(attachment, COMMAND, descriptor, context, limit),
        ),
    )]);
    TypedAsyncSdkPlugin::new(
        manifest,
        bindings,
        AsyncCodecBudget {
            max_pending: 1,
            max_terminal_wire_bytes: limit,
            max_total_codec_wire_bytes: 4 * limit,
        },
    )
}
