//! Host observations are supplied by the owner; preparation never commits a route.
use super::document::{
    DocumentPreparationError, PreparedStartDocument, StartLabels, prepare_start_document,
};
use super::{
    NavigationError, NavigationGuards, RouteCatalog, RouteHostObservation, RouteTicket, ShellState,
};

pub enum StartPreparationError {
    Document(DocumentPreparationError),
    Navigation(NavigationError),
}

pub struct StartRequest {
    pub ticket: RouteTicket,
    pub document: PreparedStartDocument,
}

pub fn request_start(
    state: &mut ShellState,
    catalog: &RouteCatalog,
    labels: StartLabels,
    guards: NavigationGuards,
) -> Result<StartRequest, StartPreparationError> {
    let document = prepare_start_document(catalog, "view.start", labels)
        .map_err(StartPreparationError::Document)?;
    let ticket = state
        .request_route(catalog, "view.start", guards)
        .map_err(StartPreparationError::Navigation)?;
    Ok(StartRequest { ticket, document })
}

/// Only an adapter that actually observed the host should supply this input.
/// Acceptance is a UI route observation, never media selection or OS readiness.
pub fn observe_start(
    state: &mut ShellState,
    ticket: &RouteTicket,
    observation: RouteHostObservation,
) -> Result<(), NavigationError> {
    state.observe_route(ticket, observation)
}

#[cfg(test)]
#[path = "preparation_tests.rs"]
mod tests;
