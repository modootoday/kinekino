//! Product UI construction from one already-created SDK owner. No loop/client/store is created.
mod labels;
mod modules;
use super::{
    IntentPins,
    bootstrap::{BootstrapSelection, BootstrapTarget},
    document_factory::{
        ExplicitPanelSelection, PanelFactoryInput, PanelModuleInput, PanelSelectionMode,
    },
    initial_panel::InitialPanelStartup,
    panel_model::PanelBudget,
    window_startup::{ProposalWindowStartup, WindowStartupInput},
};
use kinekino_player::{
    application::{
        api::{ApiError, ApplicationClientHandle, ErrorCode},
        launch::native_proposal::NativeProposalLaunchRequest,
        proposals::InspectProposal,
        sdk_execution::ExecutionOwner,
    },
    assembly::meari::async_proposals::AsyncProposalRegistration,
    domain::product::PRODUCT,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncCodecBudget;
pub use labels::ProductWindowLabels;
use meari::{application::assets::VerifiedAssets, domain::build::ContentHash};
use meari_native::{
    application::WindowOptions,
    domain::Viewport,
    infrastructure::{native_host::HostScheduleLimits, window::NativeWindowObserver},
};
pub use modules::{
    modules as product_modules, modules_for_panel as product_modules_for_panel, product_css,
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
static NEXT_PRODUCT_WINDOW: AtomicU64 = AtomicU64::new(1);

pub const PRODUCT_TYPED_WIRE_BYTES: usize = 256 * 1024;
pub struct ProductWindowVisuals {
    pub css: String,
    pub assets: Option<VerifiedAssets>,
}
pub struct ProductWindowConfig {
    pub labels: ProductWindowLabels,
    pub visuals: ProductWindowVisuals,
    pub viewport: Viewport,
    pub limits: HostScheduleLimits,
    pub worker_exit_delay: Duration,
    pub observer: NativeWindowObserver,
}
pub struct ProductInputFailure {
    pub error: ApiError,
    pub owner: ExecutionOwner,
    pub request: NativeProposalLaunchRequest,
    pub config: ProductWindowConfig,
}
pub enum ProductWindowBuildFailure {
    Input(Box<ProductInputFailure>),
    Configuration(Box<WindowStartupInput>),
}
pub fn product_intent_pins() -> IntentPins {
    IntentPins {
        contract: ContentHash::of(include_bytes!("../intent_queue.rs")),
        provider: ContentHash::of(include_bytes!("../callback_adapter.rs")),
        request: ContentHash::of(b"com.kinekino.player.proposal-intent.request.v1"),
        reply: ContentHash::of(b"com.kinekino.player.proposal-intent.reply.v1"),
    }
}
pub fn product_codec_budget() -> AsyncCodecBudget {
    AsyncCodecBudget {
        max_pending: 8,
        max_terminal_wire_bytes: PRODUCT_TYPED_WIRE_BYTES,
        max_total_codec_wire_bytes: 8 * PRODUCT_TYPED_WIRE_BYTES * 4,
    }
}
pub fn product_panel_budget() -> PanelBudget {
    PanelBudget {
        entries: 8,
        text_bytes: 8192,
    }
}
pub fn product_close_policy() -> super::window_bridge::PanelClosePolicy {
    Box::new(|review, cause| {
        Ok(cause != meari_native::infrastructure::native_host::CloseCause::User || !review.dirty())
    })
}

pub fn prepare_product_window(
    client: ApplicationClientHandle,
    owner: ExecutionOwner,
    request: NativeProposalLaunchRequest,
    config: ProductWindowConfig,
) -> Result<ProposalWindowStartup, ProductWindowBuildFailure> {
    prepare_window(client, owner, request, config, None)
}
/// P1a local gate using the original frame; production application still requires P1b/P2.
pub fn prepare_product_frame_window(
    client: ApplicationClientHandle,
    owner: ExecutionOwner,
    request: NativeProposalLaunchRequest,
    config: ProductWindowConfig,
    frame: super::frame_composition::FrameComposition,
) -> Result<ProposalWindowStartup, ProductWindowBuildFailure> {
    prepare_window(client, owner, request, config, Some(frame))
}
fn prepare_window(
    client: ApplicationClientHandle,
    owner: ExecutionOwner,
    request: NativeProposalLaunchRequest,
    config: ProductWindowConfig,
    frame: Option<super::frame_composition::FrameComposition>,
) -> Result<ProposalWindowStartup, ProductWindowBuildFailure> {
    let selection = match BootstrapSelection::try_from(request.selection.clone()) {
        Ok(selection) => selection,
        Err(error) => {
            return Err(ProductWindowBuildFailure::Input(Box::new(
                ProductInputFailure {
                    error,
                    owner,
                    request,
                    config,
                },
            )));
        }
    };
    if config.worker_exit_delay < config.limits.min_delay()
        || config.worker_exit_delay > config.limits.max_delay()
    {
        return Err(ProductWindowBuildFailure::Input(Box::new(
            ProductInputFailure {
                error: ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Worker exit observation delay is outside Native schedule limits",
                ),
                owner,
                request,
                config,
            },
        )));
    }
    let sequence =
        match NEXT_PRODUCT_WINDOW.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        }) {
            Ok(sequence) => sequence,
            Err(_) => {
                return Err(ProductWindowBuildFailure::Input(Box::new(
                    ProductInputFailure {
                        error: ApiError::new(
                            ErrorCode::ResourceExhausted,
                            "Product window namespace exhausted",
                        ),
                        owner,
                        request,
                        config,
                    },
                )));
            }
        };
    let namespace = format!("product-native-proposal-{}-{sequence}", std::process::id());
    let (entry, inventory) = product_modules(&selection.target);
    let (mode, item_index) = match &selection.target {
        BootstrapTarget::Card { index }
        | BootstrapTarget::Highlight { index }
        | BootstrapTarget::CueCorrection { index, .. }
        | BootstrapTarget::DocumentCorrection { index, .. } => {
            (PanelSelectionMode::Item, Some(*index))
        }
        BootstrapTarget::DocumentRepair { .. } => (PanelSelectionMode::DocumentRepair, None),
        BootstrapTarget::ManualRepair { .. } => (PanelSelectionMode::ManualRepair, None),
    };
    let correction = matches!(
        selection.target,
        BootstrapTarget::CueCorrection { .. }
            | BootstrapTarget::DocumentCorrection { .. }
            | BootstrapTarget::DocumentRepair { .. }
    );
    let initial = match InitialPanelStartup::start(
        owner,
        request.context.clone(),
        selection,
        PRODUCT_TYPED_WIRE_BYTES,
    ) {
        Ok(initial) => initial,
        Err(failure) => {
            return Err(ProductWindowBuildFailure::Input(Box::new(
                ProductInputFailure {
                    error: failure.error,
                    owner: failure.owner,
                    request,
                    config,
                },
            )));
        }
    };
    let labels = config.labels;
    let initial_labels = labels.clone();
    let initial_correction_labels = labels.clone();
    let live_labels = labels.clone();
    let live_correction_labels = labels.clone();
    let factory_labels = labels.clone();
    let factory_correction_labels = labels;
    let input = WindowStartupInput {
        initial,
        initial_labels: Box::new(move || initial_labels.bootstrap()),
        correction_labels: Box::new(move || {
            correction.then(|| initial_correction_labels.correction())
        }),
        mount: Box::new(move |handoff| {
            handoff.mount_composed(
                PanelFactoryInput {
                    client,
                    context: request.context,
                    selection: ExplicitPanelSelection {
                        proposal: InspectProposal {
                            proposal_id: request.selection.proposal_id,
                        },
                        source_id: request.selection.source_id,
                        source_revision: request.selection.source_revision,
                        source_sha256: request.selection.source_sha256,
                        item_index,
                        mode,
                    },
                    modules: PanelModuleInput {
                        entry,
                        inventory,
                        initial_props: BTreeMap::new(),
                        instance_namespace: namespace.clone(),
                        epoch: 1,
                        source_generation: 1,
                        css: config.visuals.css,
                        viewport: config.viewport,
                        assets: config.visuals.assets,
                    },
                    labels: factory_labels.panel,
                    budget: product_panel_budget(),
                    pins: product_intent_pins(),
                    registry_pending: 8,
                    request_namespace: namespace,
                },
                correction.then(|| factory_correction_labels.correction()),
                frame,
                |execution, registry, scope, context| {
                    if mode == PanelSelectionMode::ManualRepair || mode == PanelSelectionMode::Item
                    {
                        AsyncProposalRegistration::register_raw_reviews(
                            registry,
                            scope,
                            context,
                            execution,
                            product_codec_budget(),
                        )
                    } else {
                        AsyncProposalRegistration::register_output_reviews(
                            registry,
                            scope,
                            context,
                            execution,
                            product_codec_budget(),
                        )
                    }
                },
            )
        }),
        labels: Box::new(move |_, _, _| live_labels.panel.clone()),
        live_correction_labels: correction.then(|| {
            Box::new(
                move |_: &super::ReviewSession, _: Option<usize>, _: Option<super::ViewError>| {
                    live_correction_labels.correction()
                },
            ) as super::window_bridge::CorrectionLabelProvider
        }),
        close_policy: product_close_policy(),
        pins: product_intent_pins(),
        options: WindowOptions {
            title: PRODUCT.display_name.into(),
        },
        observer: config.observer,
        limits: config.limits,
        worker_exit_delay: config.worker_exit_delay,
        commit: None,
    };
    ProposalWindowStartup::new(input).map_err(ProductWindowBuildFailure::Configuration)
}
