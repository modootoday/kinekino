//! Canonical product proposal capability manifest and content identity inputs.
use meari::{
    application::plugins::{CapabilityDescriptor, PLUGIN_API_VERSION, PluginManifest},
    domain::build::{ContentHash, TargetKind},
};
use std::collections::{BTreeMap, BTreeSet};

pub const CAPABILITIES: [&str; 4] = [
    "proposals.create",
    "proposals.inspect",
    "proposals.edit",
    "proposals.accept",
];
pub const ASYNC_CAPABILITIES: [&str; 5] = [
    "proposals.create",
    "proposals.inspect",
    "proposals.edit",
    "proposals.accept",
    "proposals.import_historical",
];
pub const OUTPUT_REVIEW_CAPABILITIES: [&str; 3] = [
    "proposals.inspect_output_review_target",
    "proposals.record_output_decision",
    "proposals.output_decisions",
];
pub const RAW_REVIEW_CAPABILITIES: [&str; 4] = [
    "reviews.inspect",
    "reviews.read_text_asset",
    "reviews.record_decision",
    "reviews.decisions",
];
/// Explicit Native12 profile; raw model review remains separate from accepted output review.
pub fn raw_review_proposal_manifest() -> PluginManifest {
    let mut manifest = output_review_proposal_manifest();
    manifest.contract = ContentHash::of(b"player-proposals/native12-raw-reviews/1");
    for name in RAW_REVIEW_CAPABILITIES {
        manifest.capabilities.insert(
            name.into(),
            CapabilityDescriptor {
                targets: BTreeSet::from([TargetKind::Native]),
                request_schema: schema(
                    &format!("{name}:request"),
                    include_bytes!("../../application/model_review/schema.rs"),
                ),
                reply_schema: schema(
                    &format!("{name}:reply"),
                    include_bytes!("../../application/model_review/schema.rs"),
                ),
                error_schema: Some(ContentHash::of(include_bytes!(
                    "../../../../kinekino-media-contracts/src/errors.rs"
                ))),
                event_schema: None,
            },
        );
    }
    let mut source = manifest.provider.as_str().as_bytes().to_vec();
    source.extend_from_slice(include_bytes!("../../application/model_review/mod.rs"));
    source.extend_from_slice(include_bytes!("../../domain/model_review/decision.rs"));
    manifest.provider = ContentHash::of(&source);
    manifest
}
/// Explicit Native8 opt-in. Existing Immediate4 and async5 callers retain their contracts.
pub fn output_review_proposal_manifest() -> PluginManifest {
    let mut manifest = async_proposal_manifest();
    manifest.contract = ContentHash::of(
        kinekino_media_contracts::plugin_contracts::proposal_v1::OUTPUT_REVIEW_PROFILE_BYTES,
    );
    for name in OUTPUT_REVIEW_CAPABILITIES {
        manifest.capabilities.insert(
            name.into(),
            CapabilityDescriptor {
                targets: BTreeSet::from([TargetKind::Native]),
                request_schema: schema(
                    &format!("{name}:request"),
                    include_bytes!("../../application/proposals/output_review/schema.rs"),
                ),
                reply_schema: schema(
                    &format!("{name}:reply"),
                    include_bytes!("../../application/proposals/output_review/schema.rs"),
                ),
                error_schema: Some(ContentHash::of(include_bytes!(
                    "../../../../kinekino-media-contracts/src/errors.rs"
                ))),
                event_schema: None,
            },
        );
    }
    let mut source = manifest.provider.as_str().as_bytes().to_vec();
    for bytes in [
        include_bytes!("../../application/proposals/output_review/types.rs").as_slice(),
        include_bytes!("../../application/proposals/output_review/ports.rs").as_slice(),
        include_bytes!("../../application/proposals/output_review/usecases.rs").as_slice(),
        include_bytes!("../../application/proposals/output_review/schema.rs").as_slice(),
    ] {
        source.push(0);
        source.extend_from_slice(bytes);
    }
    manifest.provider = ContentHash::of(&source);
    manifest
}
/// Native owned execution profile. Legacy Immediate registration retains Native4.
pub fn async_proposal_manifest() -> PluginManifest {
    let mut manifest = proposal_manifest();
    manifest.capabilities.insert(
        "proposals.import_historical".into(),
        CapabilityDescriptor {
            targets: BTreeSet::from([TargetKind::Native]),
            request_schema: schema(
                "proposals.import_historical",
                include_bytes!("../../application/ai_provenance/import.rs"),
            ),
            reply_schema: schema(
                "proposals.import_historical",
                include_bytes!("../../domain/proposals/stored.rs"),
            ),
            error_schema: Some(ContentHash::of(include_bytes!(
                "../../../../kinekino-media-contracts/src/errors.rs"
            ))),
            event_schema: None,
        },
    );
    manifest.provider = async_provider_fingerprint(include_bytes!("async_sdk_port.rs"));
    manifest
}
fn async_provider_fingerprint(common_port: &[u8]) -> ContentHash {
    let mut source = Vec::new();
    for (name, bytes) in [
        (
            "async-provider",
            include_bytes!("async_proposals/mod.rs").as_slice(),
        ),
        (
            "async-port",
            include_bytes!("async_proposals/port.rs").as_slice(),
        ),
        ("async-sdk-port", common_port),
        (
            "execution-worker",
            include_bytes!("../../application/sdk_execution/worker.rs").as_slice(),
        ),
        (
            "execution-admission",
            include_bytes!("../../application/sdk_execution/admission.rs").as_slice(),
        ),
        (
            "execution-terminal",
            include_bytes!("../../application/sdk_execution/terminal.rs").as_slice(),
        ),
        (
            "execution-control",
            include_bytes!("../../application/sdk_execution/control.rs").as_slice(),
        ),
        (
            "execution-types",
            include_bytes!("../../application/sdk_execution/types.rs").as_slice(),
        ),
        (
            "execution-state",
            include_bytes!("../../application/sdk_execution/state.rs").as_slice(),
        ),
    ] {
        source.extend_from_slice(name.as_bytes());
        source.push(0);
        source.extend_from_slice(bytes);
        source.push(0);
    }
    ContentHash::of(&source)
}
fn schema(role: &str, definition: &[u8]) -> ContentHash {
    ContentHash::of(&[role.as_bytes(), b":", definition].concat())
}
pub fn proposal_manifest() -> PluginManifest {
    let capabilities = CAPABILITIES
        .into_iter()
        .map(|name| {
            (
                name.into(),
                CapabilityDescriptor {
                    targets: BTreeSet::from([TargetKind::Native]),
                    request_schema: schema(
                        name,
                        include_bytes!("../../application/proposals/mod.rs"),
                    ),
                    reply_schema: schema(name, include_bytes!("../../domain/proposals/mod.rs")),
                    error_schema: Some(ContentHash::of(include_bytes!(
                        "../../../../kinekino-media-contracts/src/errors.rs"
                    ))),
                    event_schema: None,
                },
            )
        })
        .collect();
    PluginManifest {
        api_version: PLUGIN_API_VERSION,
        id: super::proposals::PLUGIN_ID.into(),
        version: env!("CARGO_PKG_VERSION").into(),
        contract: ContentHash::of(kinekino_media_contracts::plugin_contracts::proposal_v1::BYTES),
        provider: ContentHash::of(include_bytes!("proposals.rs")),
        capabilities,
        assets: BTreeMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn async_provider_identity_changes_when_shared_port_source_changes() {
        let original = include_bytes!("async_sdk_port.rs");
        assert_eq!(
            async_proposal_manifest().provider,
            async_provider_fingerprint(original)
        );
        let mut changed = original.to_vec();
        changed.push(b'\n');
        assert_ne!(
            async_proposal_manifest().provider,
            async_provider_fingerprint(&changed)
        );
    }
    #[test]
    fn product_contract_has_only_native_proposal_capabilities_and_role_separated_pins() {
        let manifest = proposal_manifest();
        assert_eq!(manifest.id, "com.kinekino.player.proposals");
        assert_eq!(manifest.capabilities.len(), 4);
        let mut requests = BTreeSet::new();
        for name in CAPABILITIES {
            let descriptor = &manifest.capabilities[name];
            assert_eq!(descriptor.targets, BTreeSet::from([TargetKind::Native]));
            assert_ne!(descriptor.request_schema, descriptor.reply_schema);
            assert_ne!(
                descriptor.error_schema.as_ref(),
                Some(&descriptor.reply_schema)
            );
            assert!(requests.insert(descriptor.request_schema.as_str().to_owned()));
            assert!(descriptor.event_schema.is_none());
        }
        assert!(manifest.assets.is_empty());
        assert_ne!(manifest.contract, manifest.provider);
    }
}
