//! Explicit NoMedia preparation. The default launcher and the original cleanup protocol stay separate.
use super::{
    PreparedShellDocument, ShellHostConfig, ShellHostController, TypedPreparedViewerLaunch,
    TypedViewerCleanup, TypedViewerOwners, UnrunNative, ViewerCleanupLimits, ViewerHost,
    ViewerLaunchInput, finish_typed_viewer,
};
use crate::assembly::meari::{
    native_cleanup_driver::CleanupDriverOutcome,
    player_shell_document::video::ViewingVideoError,
    player_shell_host::ShellFrameConsumer,
    quarantine_surface::{
        QuarantineChild,
        frame_delivery::{FrameReceiver, FrameRequest, NoMediaFrameConsumer},
    },
};
use meari_native::infrastructure::native_host::NativeWindowHost;
use std::{
    cell::RefCell,
    num::{NonZeroU64, NonZeroUsize},
    rc::Rc,
    time::Instant,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoMediaFrameOptions {
    pub request_id: NonZeroU64,
    pub expires_at: Instant,
    pub frame_budget: NonZeroUsize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NoMediaSetupError {
    Video(ViewingVideoError),
    ReceiverOccupied,
    ConsumerRejected,
}

struct UnrunDocument<H> {
    document: PreparedShellDocument<H>,
    host: ShellHostConfig,
}
enum SetupOwners<H: NativeWindowHost> {
    Prepared(Box<UnrunDocument<H>>),
    Controller(Box<ShellHostController<H, QuarantineChild>>),
}
#[allow(
    dead_code,
    reason = "Rejected pair retains both original actors through cleanup without retrying them"
)]
struct FramePair {
    consumer: NoMediaFrameConsumer,
    receiver: FrameReceiver,
}
#[allow(
    dead_code,
    reason = "Opaque rejected actors remain owned through cleanup; no retry or replacement is performed"
)]
enum UninstalledFrames {
    None,
    Pair(Box<FramePair>),
    Consumer(Box<dyn ShellFrameConsumer<QuarantineChild>>),
}

#[must_use = "Retain the original setup owners and every rejected channel/consumer"]
struct SetupFailure<H: NativeWindowHost> {
    error: NoMediaSetupError,
    request: NoMediaFrameOptions,
    owners: SetupOwners<H>,
    uninstalled: UninstalledFrames,
}

trait ReceiverOwner {
    fn install_receiver(&mut self, receiver: FrameReceiver) -> Result<(), FrameReceiver>;
}
impl ReceiverOwner for ViewerHost {
    fn install_receiver(&mut self, receiver: FrameReceiver) -> Result<(), FrameReceiver> {
        self.install_frame_receiver(receiver)
    }
}

fn request_for<H>(
    document: &PreparedShellDocument<H>,
    request: NoMediaFrameOptions,
) -> Result<FrameRequest, ViewingVideoError> {
    let video = document.inspect_viewing_video()?;
    Ok(FrameRequest {
        id: request.request_id,
        origin: video.origin().clone(),
        video: video.id().as_str().into(),
        expires_at: request.expires_at,
        frame_budget: request.frame_budget,
    })
}

#[cfg(test)]
fn install_frames<H: NativeWindowHost + ReceiverOwner>(
    document: PreparedShellDocument<H>,
    host: ShellHostConfig,
    request: NoMediaFrameOptions,
) -> Result<ShellHostController<H, QuarantineChild>, Box<SetupFailure<H>>> {
    install_frames_with_binding(document, host, request, |_, _| {})
}
fn install_frames_with_binding<H: NativeWindowHost + ReceiverOwner>(
    mut document: PreparedShellDocument<H>,
    host: ShellHostConfig,
    request: NoMediaFrameOptions,
    bind: impl FnOnce(
        &mut H,
        Rc<RefCell<crate::assembly::meari::player_shell_host::StartRouteObservation>>,
    ),
) -> Result<ShellHostController<H, QuarantineChild>, Box<SetupFailure<H>>> {
    let frame_request = match request_for(&document, request) {
        Ok(frame_request) => frame_request,
        Err(error) => {
            return Err(Box::new(SetupFailure {
                error: NoMediaSetupError::Video(error),
                request,
                owners: SetupOwners::Prepared(Box::new(UnrunDocument { document, host })),
                uninstalled: UninstalledFrames::None,
            }));
        }
    };
    let (consumer, receiver) = NoMediaFrameConsumer::new(frame_request);
    if let Err(receiver) = document.inputs.host_owner.install_receiver(receiver) {
        return Err(Box::new(SetupFailure {
            error: NoMediaSetupError::ReceiverOccupied,
            request,
            owners: SetupOwners::Prepared(Box::new(UnrunDocument { document, host })),
            uninstalled: UninstalledFrames::Pair(Box::new(FramePair { consumer, receiver })),
        }));
    }
    complete_installation(
        ShellHostController::new_with_route_binding(document, host, bind),
        Box::new(consumer),
        request,
    )
}

fn complete_installation<H: NativeWindowHost>(
    mut controller: ShellHostController<H, QuarantineChild>,
    consumer: Box<dyn ShellFrameConsumer<QuarantineChild>>,
    request: NoMediaFrameOptions,
) -> Result<ShellHostController<H, QuarantineChild>, Box<SetupFailure<H>>> {
    match controller.install_frame_consumer(consumer) {
        Ok(()) => Ok(controller),
        Err(consumer) => Err(Box::new(SetupFailure {
            error: NoMediaSetupError::ConsumerRejected,
            request,
            owners: SetupOwners::Controller(Box::new(controller)),
            uninstalled: UninstalledFrames::Consumer(consumer),
        })),
    }
}

#[must_use = "The original prepared document/controller and unrun Native owner remain retained"]
pub struct NoMediaSetupFailure {
    setup: SetupFailure<ViewerHost>,
    native: UnrunNative,
}
impl NoMediaSetupFailure {
    pub fn error(&self) -> &NoMediaSetupError {
        &self.setup.error
    }
    pub fn request(&self) -> NoMediaFrameOptions {
        self.setup.request
    }
}

#[must_use = "Preserve all supplied owners on preparation or NoMedia installation failure"]
pub enum NoMediaPrepareFailure {
    Preparation {
        owners: Box<TypedViewerOwners<QuarantineChild>>,
        request: NoMediaFrameOptions,
    },
    Setup(Box<NoMediaSetupFailure>),
}

pub fn prepare_no_media_viewer_launch(
    input: ViewerLaunchInput,
    request: NoMediaFrameOptions,
) -> Result<TypedPreparedViewerLaunch<QuarantineChild>, Box<NoMediaPrepareFailure>> {
    prepare_no_media_profile(
        input,
        request,
        super::ViewerDocumentMode::Standalone,
        |_, _| {},
    )
}
#[cfg(test)]
pub(crate) fn prepare_no_media_product_for_test(
    input: ViewerLaunchInput,
    request: NoMediaFrameOptions,
) -> Result<TypedPreparedViewerLaunch<QuarantineChild>, Box<NoMediaPrepareFailure>> {
    prepare_no_media_profile(
        input,
        request,
        super::ViewerDocumentMode::ProductFrame,
        |host, route| host.bind_route_observation(route),
    )
}
fn prepare_no_media_profile(
    input: ViewerLaunchInput,
    request: NoMediaFrameOptions,
    mode: super::ViewerDocumentMode,
    bind: impl FnOnce(
        &mut ViewerHost,
        Rc<RefCell<crate::assembly::meari::player_shell_host::StartRouteObservation>>,
    ),
) -> Result<TypedPreparedViewerLaunch<QuarantineChild>, Box<NoMediaPrepareFailure>> {
    prepare_no_media_choices(input, request, mode, None, bind)
}
fn prepare_no_media_choices(
    input: ViewerLaunchInput,
    request: NoMediaFrameOptions,
    mode: super::ViewerDocumentMode,
    choices: Option<super::ViewerChoices>,
    bind: impl FnOnce(
        &mut ViewerHost,
        Rc<RefCell<crate::assembly::meari::player_shell_host::StartRouteObservation>>,
    ),
) -> Result<TypedPreparedViewerLaunch<QuarantineChild>, Box<NoMediaPrepareFailure>> {
    let prepared = super::prepare_viewer_document_profile::<QuarantineChild>(input, mode, choices)
        .map_err(|owners| Box::new(NoMediaPrepareFailure::Preparation { owners, request }))?;
    let controller =
        match install_frames_with_binding(prepared.document, prepared.host, request, bind) {
            Ok(controller) => controller,
            Err(setup) => {
                return Err(Box::new(NoMediaPrepareFailure::Setup(Box::new(
                    NoMediaSetupFailure {
                        setup: *setup,
                        native: prepared.native,
                    },
                ))));
            }
        };
    Ok(TypedPreparedViewerLaunch {
        controller,
        native: prepared.native,
    })
}

/// Rejected parts stay owned independently of the existing cleanup driver's returned owners.
#[must_use = "Keep rejected actors with the original request and failure throughout cleanup"]
pub struct NoMediaFailureRetention {
    error: Option<NoMediaSetupError>,
    request: NoMediaFrameOptions,
    #[allow(
        dead_code,
        reason = "Keep original uninstalled actors alive alongside every returned cleanup outcome"
    )]
    uninstalled: UninstalledFrames,
}
impl NoMediaFailureRetention {
    pub fn error(&self) -> Option<&NoMediaSetupError> {
        self.error.as_ref()
    }
    pub fn request(&self) -> NoMediaFrameOptions {
        self.request
    }
}
impl<H: NativeWindowHost> SetupFailure<H> {
    fn into_controller(
        self,
    ) -> (
        ShellHostController<H, QuarantineChild>,
        NoMediaFailureRetention,
    ) {
        let controller = match self.owners {
            SetupOwners::Prepared(prepared) => {
                let UnrunDocument { document, host } = *prepared;
                ShellHostController::new(document, host)
            }
            SetupOwners::Controller(controller) => *controller,
        };
        (
            controller,
            NoMediaFailureRetention {
                error: Some(self.error),
                request: self.request,
                uninstalled: self.uninstalled,
            },
        )
    }
}

#[must_use = "Move the same owners into cleanup while retaining the rejected actors"]
pub struct NoMediaCleanupParts {
    pub owners: TypedViewerOwners<QuarantineChild>,
    pub retention: NoMediaFailureRetention,
}
impl NoMediaPrepareFailure {
    /// Explicit ownership transition for cleanup, never a preparation retry or a new frame request.
    pub fn into_cleanup_parts(self) -> NoMediaCleanupParts {
        match self {
            Self::Preparation { owners, request } => NoMediaCleanupParts {
                owners: *owners,
                retention: NoMediaFailureRetention {
                    error: None,
                    request,
                    uninstalled: UninstalledFrames::None,
                },
            },
            Self::Setup(failure) => {
                let NoMediaSetupFailure { setup, native } = *failure;
                let (controller, retention) = setup.into_controller();
                NoMediaCleanupParts {
                    owners: TypedViewerOwners::Unrun {
                        controller: Box::new(controller),
                        native: Box::new(native),
                    },
                    retention,
                }
            }
        }
    }
}
#[must_use = "Keep the original cleanup outcome and rejected actors together"]
pub struct NoMediaFailureCleanup {
    pub cleanup: CleanupDriverOutcome<TypedViewerCleanup<QuarantineChild>>,
    pub retention: NoMediaFailureRetention,
}
impl NoMediaCleanupParts {
    pub fn finish(self, limits: ViewerCleanupLimits) -> NoMediaFailureCleanup {
        NoMediaFailureCleanup {
            cleanup: finish_typed_viewer(self.owners, limits),
            retention: self.retention,
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) fn prepare_no_media_live_for_test(
    input: ViewerLaunchInput,
    request: NoMediaFrameOptions,
    choices: crate::assembly::meari::viewer_host::TrustedLiveChoices,
) -> Result<TypedPreparedViewerLaunch<QuarantineChild>, Box<NoMediaPrepareFailure>> {
    prepare_no_media_choices(
        input,
        request,
        super::ViewerDocumentMode::ProductFrame,
        Some(super::ViewerChoices::Live(choices)),
        |host, route| {
            host.bind_route_observation(route);
        },
    )
}

/// Static test source only, once before the original closed product graph and first mount.
#[cfg(test)]
pub(super) fn prepare_live_fixture_source(
    document: &mut crate::presentation::web_ui_adapter::player_shell::document::PreparedStartDocument,
) -> Result<(), crate::presentation::web_ui_adapter::player_shell::document::DocumentPreparationError>
{
    use crate::presentation::web_ui_adapter::player_shell::document::DocumentPreparationError;
    if !crate::assembly::meari::viewer_native_probe::current()
        .is_some_and(|probe| probe.uses_explicit_live_fixture_source())
    {
        return Ok(());
    }
    let path: meari::domain::build::BuildPath = "viewing/read-panel.tsx"
        .to_owned()
        .try_into()
        .map_err(|_| DocumentPreparationError::InvalidFixedPath)?;
    let mut inventory = document.inventory.clone();
    let source = inventory
        .get_mut(&path)
        .ok_or(DocumentPreparationError::InvalidFixedPath)?;
    let props_anchor = "type Props = {";
    let button_anchor = "    <div>{props.capturedCardChoices}</div>";
    if source.matches(props_anchor).count() != 1 || source.matches(button_anchor).count() != 1 {
        return Err(DocumentPreparationError::InvalidFixedPath);
    }
    let replacement = source.replace(props_anchor, "type Props = { fixtureLiveKey: string; onOpenLiveProposal: (key: string) => void;")
        .replace(button_anchor, "    <button className=\"aw-button\" onClick={() => props.onOpenLiveProposal(props.fixtureLiveKey)}>Explicit fixture proposal</button><div>{props.capturedCardChoices}</div>");
    *source = replacement;
    let graph =
        meari::parse_module_graph(&path, &inventory).map_err(DocumentPreparationError::Source)?;
    graph
        .validate()
        .map_err(|_| DocumentPreparationError::InvalidFixedPath)?;
    let mut props = document.props.clone();
    props.insert(
        "fixtureLiveKey".into(),
        meari::domain::Value::String("live-proposal-7-0".into()),
    );
    props.insert(
        "onOpenLiveProposal".into(),
        meari::domain::Value::CallbackReference(meari::domain::host_callbacks::CallbackReference {
            binding: "onOpenLiveProposal".into(),
            signature: meari::domain::host_callbacks::CallbackSignature::Text,
        }),
    );
    document.graph = graph;
    document.inventory = inventory;
    document.props = props;
    Ok(())
}

#[cfg(test)]
fn live_source_contract_document()
-> crate::presentation::web_ui_adapter::player_shell::document::PreparedStartDocument {
    use crate::presentation::web_ui_adapter::player_shell::{
        RouteCatalog,
        document::{StartLabels, enable_read_panel, prepare_start_document, read_props},
    };
    let catalog = RouteCatalog::bundled().unwrap();
    let labels = StartLabels {
        connection: "fixture".into(),
        time: "unobserved".into(),
    };
    let props = read_props(&labels);
    let mut document = prepare_start_document(&catalog, "view.start", labels)
        .unwrap_or_else(|_| panic!("fixed Viewing source must prepare"));
    enable_read_panel(&mut document, props)
        .unwrap_or_else(|_| panic!("fixed read source must prepare"));
    document
}
#[cfg(test)]
struct LiveSourceDiagnostic(
    crate::presentation::web_ui_adapter::player_shell::document::DocumentPreparationError,
);
#[cfg(test)]
impl std::fmt::Debug for LiveSourceDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::presentation::web_ui_adapter::player_shell::document::DocumentPreparationError;
        match &self.0 {
            DocumentPreparationError::Source(error) => f
                .debug_tuple("OriginalSourcePreparationError")
                .field(error)
                .finish(),
            DocumentPreparationError::UnknownRoute => {
                f.write_str("OriginalPreparationError::UnknownRoute")
            }
            DocumentPreparationError::SourceMissing => {
                f.write_str("OriginalPreparationError::SourceMissing")
            }
            DocumentPreparationError::SourceNotIntegrated => {
                f.write_str("OriginalPreparationError::SourceNotIntegrated")
            }
            DocumentPreparationError::InvalidFixedPath => {
                f.write_str("OriginalPreparationError::InvalidFixedPath")
            }
        }
    }
}
#[cfg(test)]
#[test]
fn live_static_source_opt_in_preserves_default_graph_and_closed_inventory()
-> Result<(), LiveSourceDiagnostic> {
    use crate::assembly::meari::viewer_native_probe::{
        Probe, Scope,
        live_retained::{Case, Mode},
    };
    let mut document = live_source_contract_document();
    let before = document.graph.fingerprint().unwrap();
    let inventory = document.inventory.clone();
    let original_props = document.props.clone();
    prepare_live_fixture_source(&mut document).map_err(LiveSourceDiagnostic)?;
    assert_eq!(document.graph.fingerprint().unwrap(), before);
    assert_eq!(document.inventory, inventory);
    assert_eq!(document.props, original_props);
    {
        let _scope = Scope::enter(Probe::for_live(Case::Card, Mode::DefaultChooser));
        prepare_live_fixture_source(&mut document).map_err(LiveSourceDiagnostic)?;
        assert_eq!(document.graph.fingerprint().unwrap(), before);
        assert_eq!(document.inventory, inventory);
        assert_eq!(document.props, original_props);
    }
    let _scope = Scope::enter(Probe::for_live(Case::Card, Mode::Positive));
    prepare_live_fixture_source(&mut document).map_err(LiveSourceDiagnostic)?;
    assert_ne!(document.graph.fingerprint().unwrap(), before);
    assert_eq!(document.inventory.len(), inventory.len());
    assert_eq!(document.props.len(), original_props.len() + 2);
    assert_eq!(
        document.props.get("fixtureLiveKey"),
        Some(&meari::domain::Value::String("live-proposal-7-0".into()))
    );
    assert!(
        matches!(document.props.get("onOpenLiveProposal"), Some(meari::domain::Value::CallbackReference(reference)) if reference.binding == "onOpenLiveProposal" && reference.signature == meari::domain::host_callbacks::CallbackSignature::Text)
    );
    for (key, value) in original_props {
        assert_eq!(document.props.get(&key), Some(&value));
    }
    for (path, source) in inventory {
        if path.as_str() == "viewing/read-panel.tsx" {
            assert!(document.inventory[&path].contains("onOpenLiveProposal"));
            assert!(
                document.inventory[&path]
                    .contains("props.onOpenLiveProposal(props.fixtureLiveKey)")
            );
        } else {
            assert_eq!(document.inventory.get(&path), Some(&source));
        }
    }
    Ok(())
}
#[cfg(test)]
#[test]
fn live_static_source_rejects_foreign_anchor_without_partial_graph_replacement() {
    use crate::assembly::meari::viewer_native_probe::{
        Probe, Scope,
        live_retained::{Case, Mode},
    };
    let _scope = Scope::enter(Probe::for_live(Case::Highlight, Mode::Positive));
    let mut document = live_source_contract_document();
    let path: meari::domain::build::BuildPath =
        "viewing/read-panel.tsx".to_owned().try_into().unwrap();
    document
        .inventory
        .insert(path, "export function Foreign(){return <main/>;}".into());
    let before = document.graph.fingerprint().unwrap();
    let inventory = document.inventory.clone();
    assert!(matches!(prepare_live_fixture_source(&mut document), Err(crate::presentation::web_ui_adapter::player_shell::document::DocumentPreparationError::InvalidFixedPath)));
    assert_eq!(document.graph.fingerprint().unwrap(), before);
    assert_eq!(document.inventory, inventory);
}
