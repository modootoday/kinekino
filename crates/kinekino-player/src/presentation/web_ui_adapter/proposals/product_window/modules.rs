//! Closed product source inventory. Launch JSON cannot select modules or verification entries.
use super::super::bootstrap::{BootstrapPanel, BootstrapTarget};
use meari::domain::build::BuildPath;
use std::collections::BTreeMap;

pub fn modules(target: &BootstrapTarget) -> (BuildPath, BTreeMap<BuildPath, String>) {
    let panel = match target {
        BootstrapTarget::Card { .. } => BootstrapPanel::Card,
        BootstrapTarget::Highlight { .. } => BootstrapPanel::Highlight,
        BootstrapTarget::CueCorrection { .. } | BootstrapTarget::DocumentCorrection { .. } => {
            BootstrapPanel::Correction
        }
        BootstrapTarget::DocumentRepair { .. } => BootstrapPanel::DocumentRepair,
        BootstrapTarget::ManualRepair { .. } => BootstrapPanel::ManualRepair,
    };
    modules_for_panel(panel)
}
/// Source inventory selection only. This does not select a proposal or default item index.
pub fn modules_for_panel(panel: BootstrapPanel) -> (BuildPath, BTreeMap<BuildPath, String>) {
    let (entry, source) = match panel {
        BootstrapPanel::ManualRepair => (
            "studio/manual-repair.tsx",
            include_str!("../../../web_ui/studio/manual-repair.tsx"),
        ),
        BootstrapPanel::Card => (
            "studio/cards/proposal-panel.tsx",
            include_str!("../../../web_ui/studio/cards/proposal-panel.tsx"),
        ),
        BootstrapPanel::Highlight => (
            "studio/clips/highlight-proposal-panel.tsx",
            include_str!("../../../web_ui/studio/clips/highlight-proposal-panel.tsx"),
        ),
        BootstrapPanel::Correction | BootstrapPanel::DocumentRepair => (
            "studio/subtitles/correction-panel.tsx",
            include_str!("../../../web_ui/studio/subtitles/correction-panel.tsx"),
        ),
    };
    let path =
        |value: &str| BuildPath::try_from(value.to_owned()).expect("fixed product module path");
    let mut inventory = BTreeMap::from([
        (path(entry), source.into()),
        (
            path("studio/cards/captured-proposal-panel.tsx"),
            include_str!("../../../web_ui/studio/cards/captured-proposal-panel.tsx").into(),
        ),
        (
            path("components/proposal-output-review.tsx"),
            include_str!("../../../web_ui/components/proposal-output-review.tsx").into(),
        ),
        (
            path("components/proposal-source.tsx"),
            include_str!("../../../web_ui/components/proposal-source.tsx").into(),
        ),
        (
            path("components/proposal-actions.tsx"),
            include_str!("../../../web_ui/components/proposal-actions.tsx").into(),
        ),
        (
            path("components/capture-actions.tsx"),
            include_str!("../../../web_ui/components/capture-actions.tsx").into(),
        ),
        (
            path("components/card-copy-fields.tsx"),
            include_str!("../../../web_ui/components/card-copy-fields.tsx").into(),
        ),
        (
            path("components/text-comparison.tsx"),
            include_str!("../../../web_ui/components/text-comparison.tsx").into(),
        ),
    ]);
    if !matches!(panel, BootstrapPanel::DocumentRepair) {
        inventory.insert(
            path("studio/raw-review-panel.tsx"),
            include_str!("../../../web_ui/studio/raw-review-panel.tsx").into(),
        );
    }
    (path(entry), inventory)
}

pub fn product_css() -> String {
    [
        include_str!("../../../web_ui/styles/tokens.css"),
        include_str!("../../../web_ui/styles/components.css"),
        include_str!("../../../web_ui/styles/studio.css"),
        include_str!("../../../web_ui/styles/subtitles.css"),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_player::assembly::meari::proposal_manifest::{
        ASYNC_CAPABILITIES, OUTPUT_REVIEW_CAPABILITIES, RAW_REVIEW_CAPABILITIES,
        raw_review_proposal_manifest,
    };
    use meari::domain::{
        PropType,
        build::{ContentHash, TargetKind},
        host_callbacks::CallbackSignature,
    };
    use std::collections::BTreeSet;

    #[test]
    fn manual_repair_product_graph_loads_exact_callbacks_and_native12_metadata() {
        let (entry, inventory) = modules_for_panel(BootstrapPanel::ManualRepair);
        assert_eq!(entry.as_str(), "studio/manual-repair.tsx");
        let graph = meari::parse_module_graph(&entry, &inventory).unwrap();
        graph.validate().unwrap();
        assert_eq!(graph.entry.module, entry);
        assert_eq!(graph.entry.export, "ManualRepair");
        for (path, module) in &graph.modules {
            let source = &inventory[path];
            assert_eq!(module.content, ContentHash::of(source.as_bytes()));
            assert_eq!(module.source_bytes, source.len() as u64);
        }
        let definition = graph.definition(&graph.entry).unwrap();
        let actual: BTreeMap<_, _> = definition
            .component
            .props
            .iter()
            .filter_map(|prop| match &prop.kind {
                PropType::Callback(signature) => Some((prop.name.as_str(), *signature)),
                _ => None,
            })
            .collect();
        let mut expected = BTreeMap::new();
        for name in [
            "onSourceChoice",
            "onFrontChange",
            "onBackChange",
            "onTitleChange",
            "onDraftChange",
            "outputReviewOnNoteChange",
            "outputReviewOnReviewerLabelChange",
            "outputReviewOnReviewerIdChange",
            "outputReviewOnClaimedTimeChange",
            "rawReviewOnPackageChange",
            "rawReviewOnItemChange",
            "rawReviewOnNoteChange",
        ] {
            expected.insert(name, CallbackSignature::Text);
        }
        for name in [
            "onStartManual",
            "onSave",
            "onAccept",
            "onRefresh",
            "outputReviewOnInspect",
            "outputReviewOnHistory",
            "outputReviewOnSelect",
            "outputReviewOnUsable",
            "outputReviewOnNeedsChanges",
            "outputReviewOnDeferred",
            "outputReviewOnRecord",
            "outputReviewOnRetry",
            "rawReviewOnOpenPackage",
            "rawReviewOnOpenItem",
            "rawReviewOnReadSource",
            "rawReviewOnReadCandidate",
            "rawReviewOnHistory",
            "rawReviewOnRecord",
            "rawReviewOnUsable",
            "rawReviewOnNeedsChanges",
            "rawReviewOnDeferred",
            "rawReviewOnInspectRetained",
            "rawReviewOnHistoryRetained",
            "rawReviewOnReleaseRetained",
        ] {
            expected.insert(name, CallbackSignature::Action);
        }
        assert_eq!(actual, expected);
        let manifest = raw_review_proposal_manifest();
        let names: BTreeSet<_> = ASYNC_CAPABILITIES
            .into_iter()
            .chain(OUTPUT_REVIEW_CAPABILITIES)
            .chain(RAW_REVIEW_CAPABILITIES)
            .collect();
        assert_eq!(names.len(), 12);
        assert_eq!(
            manifest
                .capabilities
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            names
        );
        assert_eq!(
            manifest.contract,
            ContentHash::of(b"player-proposals/native12-raw-reviews/1")
        );
        for name in RAW_REVIEW_CAPABILITIES {
            let descriptor = &manifest.capabilities[name];
            assert_eq!(descriptor.targets, BTreeSet::from([TargetKind::Native]));
            assert!(descriptor.error_schema.is_some());
            assert!(descriptor.event_schema.is_none());
        }
    }
}
