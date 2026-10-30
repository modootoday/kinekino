pub mod bootstrap;
mod callback_adapter;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod capture_card;
pub mod capture_choices;
pub mod capture_labels;
pub mod capture_mutation;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod card_selection_queue;
pub mod correction_panel;
pub mod document_controller;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod document_factory;
pub mod frame_composition;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod initial_panel;
#[path = "../studio/inspector_intents.rs"]
pub mod inspector_intents;
#[path = "../studio/inspector_state.rs"]
pub mod inspector_state;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod launch_selection;
pub mod output_review_panel;
pub mod output_review_state;
pub mod panel_model;
pub mod pending_controller;
pub mod pending_host;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod product_window;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod project_card_choices;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod project_task_choices;
pub mod raw_review_panel;
pub mod raw_review_state;
pub mod repair_panel;
pub mod review_feedback;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod scoped_panel;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod scoped_task_panel;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod window_bridge;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod window_cleanup;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod window_startup;
pub use callback_adapter::{
    IntentPins, QueueBinding, QueueCallbackAdapter, QueuedAck, bind_review_callbacks,
};
mod controller;
mod document_session;
pub use document_session::{
    ProposalDocumentSession, ReprojectionError, ReprojectionReceipt, ReviewBindings,
    SessionConstructionFailure,
};
mod fields;
mod intent_queue;
pub use controller::{ReviewController, ReviewHost};
pub use intent_queue::{DrainReport, IntentError, NodeLease, ReviewAction, ReviewIntents};
mod projection;
mod registry_host;
pub use fields::{ReviewItem, TextField, TextFieldKind, TextInput};
pub use registry_host::RegistryProposalHost;
mod session;
pub use projection::{
    OriginalText, ProposalView, SourceReference, historical_source_props, project,
};
pub use session::{
    AcceptIntent, EditIntent, ManualRepairEntry, ManualRepairSource, MutationReceipt, PendingKind,
    PendingToken, ResolvedMutation, ReviewDisplay, ReviewScope, ReviewSession, ViewError,
};
