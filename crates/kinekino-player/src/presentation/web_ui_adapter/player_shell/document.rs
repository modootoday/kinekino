//! Fixed viewing source preparation, not a renderer or Native mount receipt.
#[cfg(all(windows, feature = "native-proposal-window"))]
#[path = "document/product_frame.rs"]
pub mod product_frame;
#[path = "proposal_chooser.rs"]
pub mod proposal_chooser;
pub mod video;
use super::{RouteCatalog, RouteId};
use meari::{
    application::source_modules::ModuleDiagnostic,
    domain::{Value, build::BuildPath, source_modules::SourceModuleGraph},
};
use std::collections::BTreeMap;

pub struct StartLabels {
    pub connection: String,
    pub time: String,
}

pub enum DocumentPreparationError {
    UnknownRoute,
    SourceMissing,
    SourceNotIntegrated,
    InvalidFixedPath,
    Source(ModuleDiagnostic),
}

pub struct PreparedStartDocument {
    route: RouteId,
    pub graph: SourceModuleGraph,
    pub inventory: BTreeMap<BuildPath, String>,
    pub props: BTreeMap<String, Value>,
}

impl PreparedStartDocument {
    pub fn route(&self) -> &RouteId {
        &self.route
    }
}

pub fn prepare_start_document(
    catalog: &RouteCatalog,
    id: &str,
    labels: StartLabels,
) -> Result<PreparedStartDocument, DocumentPreparationError> {
    let route = catalog
        .get(id)
        .ok_or(DocumentPreparationError::UnknownRoute)?;
    let source = route
        .source()
        .ok_or(DocumentPreparationError::SourceMissing)?;
    if id != "view.start" || source != "viewing/player.tsx" {
        return Err(DocumentPreparationError::SourceNotIntegrated);
    }
    let mut inventory = crate::presentation::web_ui_adapter::window_chrome_sources::inventory();
    for (path, text) in [
        (
            "components/ui/range.tsx",
            include_str!("../../web_ui/components/ui/range.tsx"),
        ),
        (
            "viewing/player.tsx",
            include_str!("../../web_ui/viewing/player.tsx"),
        ),
        (
            "viewing/start-screen.tsx",
            include_str!("../../web_ui/viewing/start-screen.tsx"),
        ),
        (
            "components/transport-bar.tsx",
            include_str!("../../web_ui/components/transport-bar.tsx"),
        ),
        (
            "viewing/player-frame.tsx",
            include_str!("../../web_ui/viewing/player-frame.tsx"),
        ),
        (
            "viewing/native-transport-bar.tsx",
            include_str!("../../web_ui/viewing/native-transport-bar.tsx"),
        ),
    ] {
        let path: BuildPath = path
            .to_owned()
            .try_into()
            .map_err(|_| DocumentPreparationError::InvalidFixedPath)?;
        inventory.insert(path, text.into());
    }
    let entry: BuildPath = source
        .to_owned()
        .try_into()
        .map_err(|_| DocumentPreparationError::InvalidFixedPath)?;
    let graph =
        meari::parse_module_graph(&entry, &inventory).map_err(DocumentPreparationError::Source)?;
    Ok(PreparedStartDocument {
        route: route.id().clone(),
        graph,
        inventory,
        props: base_props(&labels),
    })
}

pub fn read_props(labels: &StartLabels) -> BTreeMap<String, Value> {
    let mut props = base_props(labels);
    props.insert("diagnosticsExpanded".into(), Value::Bool(false));
    props.insert("jobsEntryDisabled".into(), Value::Bool(true));
    props.insert(
        "jobsEntryLabel".into(),
        Value::String("작업 대기열 열기".into()),
    );
    props.insert("jobsEntryStatus".into(), Value::String(String::new()));
    props.insert(
        "capturedCardChoices".into(),
        Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty),
    );
    props.insert(
        "proposalChooser".into(),
        Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty),
    );
    props.insert(
        "startScreen".into(),
        Value::NodeSlot(meari::domain::node_slots::NodeSlot::Empty),
    );
    props.extend(super::read_observation::ReadPanel::default().props());
    props.extend(super::read_observation::local_action_props());
    props.extend(super::read_observation::native_default_props());
    props.insert("localInputErrorPresent".into(), Value::Bool(false));
    props.insert("localInputError".into(), Value::String(String::new()));
    props.extend(super::episode_transition::TransitionControlsView::default().props());
    props.insert("transitionNavigationDisabled".into(), Value::Bool(true));
    props.insert("automaticAvailable".into(), Value::Bool(false));
    props.insert("automaticError".into(), Value::String(String::new()));
    props.insert(
        "transportClass".into(),
        Value::String("aw-transport".into()),
    );
    props.insert(
        "episodePanelClass".into(),
        Value::String("aw-episode-panel".into()),
    );
    props.insert("fullscreenDisabled".into(), Value::Bool(true));
    props.extend(super::window_caption::CaptionView::default().props());
    props.insert("viewingFullscreen".into(), Value::Bool(false));
    props.insert(
        "viewingFrameClass".into(),
        Value::String("aw-player-shell".into()),
    );
    props.insert(
        "viewingRootClass".into(),
        Value::String("aw-viewing-read-root".into()),
    );
    props.insert("episodePanelClosed".into(), Value::Bool(false));
    props.insert("fullscreenLabel".into(), Value::String("전체화면".into()));
    props.insert("windowError".into(), Value::String(String::new()));
    props.insert("nativeVolumeDraft".into(), Value::String("100".into()));
    props.insert("nativeAckRetryDisabled".into(), Value::Bool(true));
    props.extend(
        super::catalog_browser::props(super::catalog_browser::CatalogBrowserView {
            busy: true,
            ..Default::default()
        })
        .expect("fixed empty catalog props"),
    );
    props.insert("catalogAckRetryDisabled".into(), Value::Bool(true));
    for name in [
        "onOpenJobs",
        "onWindowMinimize",
        "onWindowToggleMaximize",
        "onWindowUserClose",
        "onPreparePrevious",
        "onPrepareNext",
        "onCancelTransitionJob",
        "onObserveTransition",
        "onReadCapabilities",
        "onReadEngine",
        "onReadStatus",
        "onRetryCapabilitiesAck",
        "onRetryEngineAck",
        "onRetryStatusAck",
        "onRefreshReadProjection",
        "onNativePlay",
        "onNativePause",
        "onToggleFullscreen",
        "onToggleEpisodes",
        "onToggleDiagnostics",
        "onRetryNativeAck",
        "onCatalogAiring",
        "onCatalogCompleted",
        "onCatalogMovies",
        "onCatalogPoll",
        "onCatalogCancel",
        "onCatalogReadSeries",
        "onCatalogReadEpisodes",
        "onCatalogInspect",
        "onCatalogPlay",
        "onReleaseCatalog",
        "onRetryCatalogAck",
    ] {
        props.insert(
            name.into(),
            Value::CallbackReference(meari::domain::host_callbacks::CallbackReference {
                binding: name.into(),
                signature: meari::domain::host_callbacks::CallbackSignature::Action,
            }),
        );
    }
    for name in [
        "onNativeSeek",
        "onNativeVolume",
        "onNativeSeekDraft",
        "onNativeVolumeDraft",
        "onNativeSeekRangeDraft",
        "onNativeVolumeRangeDraft",
        "onNativeSeekCommit",
        "onNativeVolumeCommit",
        "onCatalogSeries",
        "onCatalogEpisode",
    ] {
        props.insert(
            name.into(),
            Value::CallbackReference(meari::domain::host_callbacks::CallbackReference {
                binding: name.into(),
                signature: meari::domain::host_callbacks::CallbackSignature::Text,
            }),
        );
    }
    props
}
fn base_props(labels: &StartLabels) -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "videoSemanticId".into(),
            Value::String(video::ViewingVideoId::Primary.as_str().into()),
        ),
        (
            "productName".into(),
            Value::String(crate::domain::product::PRODUCT.display_name.into()),
        ),
        (
            "connectionLabel".into(),
            Value::String(labels.connection.clone()),
        ),
        ("timeLabel".into(), Value::String(labels.time.clone())),
    ])
}
/// Opt-in interactive root; the existing callback-free shell graph stays unchanged.
pub fn enable_read_panel(
    document: &mut PreparedStartDocument,
    props: BTreeMap<String, Value>,
) -> Result<(), DocumentPreparationError> {
    let path: BuildPath = "viewing/read-panel.tsx"
        .to_owned()
        .try_into()
        .map_err(|_| DocumentPreparationError::InvalidFixedPath)?;
    let mut inventory = document.inventory.clone();
    inventory.insert(
        "components/window-caption-controls.tsx"
            .to_owned()
            .try_into()
            .map_err(|_| DocumentPreparationError::InvalidFixedPath)?,
        include_str!("../../web_ui/components/window-caption-controls.tsx").into(),
    );
    inventory.insert(
        "viewing/native-status.tsx"
            .to_owned()
            .try_into()
            .map_err(|_| DocumentPreparationError::InvalidFixedPath)?,
        include_str!("../../web_ui/viewing/native-status.tsx").into(),
    );
    inventory.insert(
        "viewing/episode-navigation.tsx"
            .to_owned()
            .try_into()
            .map_err(|_| DocumentPreparationError::InvalidFixedPath)?,
        include_str!("../../web_ui/viewing/episode-navigation.tsx").into(),
    );
    inventory.insert(
        "viewing/catalog-browser.tsx"
            .to_owned()
            .try_into()
            .map_err(|_| DocumentPreparationError::InvalidFixedPath)?,
        include_str!("../../web_ui/viewing/catalog-browser.tsx").into(),
    );
    inventory.insert(
        path.clone(),
        include_str!("../../web_ui/viewing/read-panel.tsx").into(),
    );
    inventory.insert(
        "viewing/proposal-chooser.tsx"
            .to_owned()
            .try_into()
            .map_err(|_| DocumentPreparationError::InvalidFixedPath)?,
        include_str!("../../web_ui/viewing/proposal-chooser.tsx").into(),
    );
    inventory.insert(
        "studio/cards/selection-rail.tsx"
            .to_owned()
            .try_into()
            .map_err(|_| DocumentPreparationError::InvalidFixedPath)?,
        include_str!("../../web_ui/studio/cards/selection-rail.tsx").into(),
    );
    let mut graph =
        meari::parse_module_graph(&path, &inventory).map_err(DocumentPreparationError::Source)?;
    let start_screen = meari::parse_module_graph(
        &super::start_screen::component_reference().module,
        &inventory,
    )
    .map_err(DocumentPreparationError::Source)?;
    graph.modules.extend(start_screen.modules);
    let chooser =
        meari::parse_module_graph(&proposal_chooser::component_reference().module, &inventory)
            .map_err(DocumentPreparationError::Source)?;
    // Both providers use this exact fixed inventory; shared imports have identical modules.
    graph.modules.extend(chooser.modules);
    document.inventory = inventory;
    document.graph = graph;
    document.props = props;
    Ok(())
}
