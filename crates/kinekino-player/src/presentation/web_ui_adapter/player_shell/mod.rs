//! UI intent/observation model only; creates no SDK, window, storage or media owner.
mod availability;
pub mod catalog_browser;
pub mod controller;
pub mod document;
pub mod episode_transition;
pub mod read_observation;
mod routes;
pub mod start_screen;
mod state;
pub(crate) mod window_caption;
pub use availability::{
    Availability, AvailabilityObservation, BackendObservation, CapabilityObservation,
    MountObservation, SourceObservation, availability,
};
pub use routes::{CatalogError, RouteCatalog, RouteDescriptor, RouteId, RouteKind};
pub use state::{
    NavigationError, NavigationGuards, RouteHostObservation, RouteTicket, ShellIntent, ShellPanel,
    ShellPhase, ShellState,
};
