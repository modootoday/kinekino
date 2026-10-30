use crate::application::subtitle_conversion::SubtitleConversionAdapter;
use kinekino_media_contracts::errors::ApiError;
use kinekino_subtitles::application::convert::{ConversionResult, ConvertSubtitles};
use meari::{
    application::plugins::*,
    domain::build::{ContentHash, TargetKind},
};
use std::collections::{BTreeMap, BTreeSet};

pub const PLUGIN_ID: &str = "com.kinekino.player.ui-host";
pub const CONVERT_CAPABILITY: &str = "subtitle.convert";

/// Supplied by host assembly from the pinned contract/provider inventory.
pub struct ConversionPins {
    pub contract: ContentHash,
    pub provider: ContentHash,
    pub request_schema: ContentHash,
    pub reply_schema: ContentHash,
    pub error_schema: ContentHash,
}

pub struct MeariSubtitleConversion<F> {
    manifest: PluginManifest,
    invoke: F,
}

impl<F> MeariSubtitleConversion<F>
where
    F: Fn(&PluginTicket, ConvertSubtitles) -> Result<ConversionResult, ApiError>,
{
    pub fn new(
        pins: ConversionPins,
        targets: BTreeSet<TargetKind>,
        invoke: F,
    ) -> Result<Self, PluginError> {
        let manifest = PluginManifest {
            api_version: PLUGIN_API_VERSION,
            id: PLUGIN_ID.into(),
            version: env!("CARGO_PKG_VERSION").into(),
            contract: pins.contract,
            provider: pins.provider,
            capabilities: BTreeMap::from([(
                CONVERT_CAPABILITY.into(),
                CapabilityDescriptor {
                    targets,
                    request_schema: pins.request_schema,
                    reply_schema: pins.reply_schema,
                    error_schema: Some(pins.error_schema),
                    event_schema: None,
                },
            )]),
            assets: BTreeMap::new(),
        };
        manifest.validate()?;
        Ok(Self { manifest, invoke })
    }

    fn descriptor(&self, capability: &str) -> Result<&CapabilityDescriptor, PluginError> {
        self.manifest.capabilities.get(capability).ok_or_else(|| {
            PluginError::new(
                PluginErrorCode::MissingCapability,
                "Unknown product capability",
            )
        })
    }
}

impl<F> PluginAdapter for MeariSubtitleConversion<F>
where
    F: Fn(&PluginTicket, ConvertSubtitles) -> Result<ConversionResult, ApiError>,
{
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn validate_payload(
        &self,
        capability: &str,
        direction: PayloadDirection,
        payload: &PluginPayload,
    ) -> Result<(), PluginError> {
        let descriptor = self.descriptor(capability)?;
        match direction {
            PayloadDirection::Request => {
                payload.decode::<ConvertSubtitles>(&descriptor.request_schema)?;
            }
            PayloadDirection::Reply => {
                payload.decode::<ConversionResult>(&descriptor.reply_schema)?;
            }
            PayloadDirection::Error => {
                let schema = descriptor.error_schema.as_ref().ok_or_else(|| {
                    PluginError::new(PluginErrorCode::InvalidPayload, "No product error schema")
                })?;
                payload.decode::<ApiError>(schema)?;
            }
            PayloadDirection::Event => {
                return Err(PluginError::new(
                    PluginErrorCode::InvalidPayload,
                    "Synchronous subtitle conversion emits no progress events",
                ));
            }
        }
        Ok(())
    }

    fn dispatch(&mut self, invocation: &PluginInvocation) -> Result<PluginDispatch, PluginError> {
        let capability = invocation.ticket.capability();
        self.validate_payload(capability, PayloadDirection::Request, &invocation.payload)?;
        let descriptor = self.descriptor(capability)?;
        let codec =
            SubtitleConversionAdapter::new(|request| (self.invoke)(&invocation.ticket, request));
        let outcome = match codec.dispatch(invocation.payload.value.clone()) {
            Ok(value) => Ok(PluginPayload {
                schema: descriptor.reply_schema.clone(),
                value,
            }),
            Err(error) => {
                let schema = descriptor.error_schema.clone().ok_or_else(|| {
                    PluginError::new(PluginErrorCode::InvalidPayload, "No product error schema")
                })?;
                let mut failure = PluginError::new(PluginErrorCode::Adapter, error.message.clone());
                failure.details = Some(PluginPayload::encode(schema, &error)?);
                Err(failure)
            }
        };
        Ok(PluginDispatch::Immediate(outcome))
    }

    fn cancel(&mut self, _: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        // No Pending work is created; the registry owns completed ticket recognition.
        Ok(CancelReceipt::AlreadyTerminal)
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        // No external handles or tasks are owned by this synchronous adapter.
        Ok(())
    }
}
