//! Proposal DTOs and SDK policies remain in the root SDK, not the product plugin crate.
use crate::application::api::ProposalClient;
use crate::application::{
    api::{ApplicationClientHandle, RequestContext},
    proposals::{AcceptProposal, CreateProposal, EditProposal, InspectProposal},
};
use kinekino_media_contracts::errors::ApiError;
use kinekino_ui_plugins::infrastructure::typed_sdk_plugin::{TypedSdkPlugin, binding};
use meari::application::plugins::*;
use std::{collections::BTreeMap, rc::Rc};

pub const PLUGIN_ID: &str = "com.kinekino.player.proposals";

pub fn adapter<C>(
    client: ApplicationClientHandle,
    manifest: PluginManifest,
    context: C,
) -> Result<TypedSdkPlugin, PluginError>
where
    C: Fn(&PluginTicket) -> Result<RequestContext, ApiError> + 'static,
{
    if manifest.id != PLUGIN_ID {
        return Err(PluginError::new(
            PluginErrorCode::InvalidManifest,
            "Proposal plugin identity differs",
        ));
    }
    let descriptor = |name: &str| {
        manifest.capabilities.get(name).cloned().ok_or_else(|| {
            PluginError::new(
                PluginErrorCode::MissingCapability,
                "Required proposal capability missing",
            )
        })
    };
    let context = Rc::new(context);
    let create = {
        let (client, context) = (client.clone(), context.clone());
        binding(
            descriptor("proposals.create")?,
            move |ticket, request: CreateProposal| {
                client.create_proposal(context(ticket)?, request)
            },
        )
    };
    let inspect = {
        let (client, context) = (client.clone(), context.clone());
        binding(
            descriptor("proposals.inspect")?,
            move |ticket, request: InspectProposal| {
                client.inspect_proposal(context(ticket)?, request)
            },
        )
    };
    let edit = {
        let (client, context) = (client.clone(), context.clone());
        binding(
            descriptor("proposals.edit")?,
            move |ticket, request: EditProposal| client.edit_proposal(context(ticket)?, request),
        )
    };
    let accept = binding(
        descriptor("proposals.accept")?,
        move |ticket, request: AcceptProposal| client.accept_proposal(context(ticket)?, request),
    );
    TypedSdkPlugin::new(
        manifest,
        BTreeMap::from([
            ("proposals.create".into(), create),
            ("proposals.inspect".into(), inspect),
            ("proposals.edit".into(), edit),
            ("proposals.accept".into(), accept),
        ]),
    )
}
