//! Shared assembly source: product host owns the SDK client and request context.
use crate::application::api::{ProposalClient};
use kinekino_player::application::api::{ApplicationClientHandle, RequestContext};
use meari::{application::plugins::*, domain::build::TargetKind};
use kinekino_media_contracts::errors::ApiError;
use kinekino_ui_plugins::infrastructure::meari_conversion::{
    ConversionPins, MeariSubtitleConversion,
};
use std::collections::BTreeSet;

pub fn adapter<C>(
    client: ApplicationClientHandle,
    pins: ConversionPins,
    targets: BTreeSet<TargetKind>,
    context: C,
) -> Result<impl PluginAdapter, PluginError>
where
    C: Fn(&PluginTicket) -> Result<RequestContext, ApiError>,
{
    MeariSubtitleConversion::new(pins, targets, move |ticket, request| {
        client.convert_subtitles(context(ticket)?, request)
    })
}
