use super::routes::{RouteCatalog, RouteId};
use std::rc::Rc;
#[derive(Clone, Copy, Debug, Default)]
pub struct NavigationGuards {
    pub pending: bool,
    pub dirty: bool,
    pub recovery_blocked: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationError {
    UnknownRoute,
    Busy,
    UnsavedEdits,
    RecoveryBlocked,
    GenerationExhausted,
    StaleObservation,
    ForeignObservation,
    RouteMismatch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellPhase {
    NoSelection,
    AwaitingHost,
    Observed,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellIntent {
    EpisodeListRequested,
    SettingsRequested,
    CloseRequested,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellPanel {
    Episodes,
    Settings,
}
#[derive(Clone, Debug)]
pub struct RouteTicket {
    owner: Rc<()>,
    generation: u64,
}
impl RouteTicket {
    pub fn generation(&self) -> u64 {
        self.generation
    }
}
pub enum RouteHostObservation {
    Reported { route: RouteId },
    Failed { diagnostic: String },
}
struct PendingRoute {
    target: RouteId,
    ticket: RouteTicket,
}
pub struct ShellState {
    identity: Rc<()>,
    generation: u64,
    pending: Option<PendingRoute>,
    observed: Option<RouteId>,
    phase: ShellPhase,
    diagnostic: Option<String>,
    intent: Option<ShellIntent>,
}
impl Default for ShellState {
    fn default() -> Self {
        Self::new()
    }
}
impl ShellState {
    pub fn new() -> Self {
        Self {
            identity: Rc::new(()),
            generation: 0,
            pending: None,
            observed: None,
            phase: ShellPhase::NoSelection,
            diagnostic: None,
            intent: None,
        }
    }
    pub fn phase(&self) -> ShellPhase {
        self.phase
    }
    pub fn desired_route(&self) -> Option<&RouteId> {
        self.pending.as_ref().map(|pending| &pending.target)
    }
    /// Last host-reported route, never synthesized from a requested selection.
    pub fn observed_route(&self) -> Option<&RouteId> {
        self.observed.as_ref()
    }
    pub fn diagnostic(&self) -> Option<&str> {
        self.diagnostic.as_deref()
    }
    pub fn intent(&self) -> Option<ShellIntent> {
        self.intent
    }
    fn guard(&self, guards: NavigationGuards) -> Result<(), NavigationError> {
        if guards.recovery_blocked {
            return Err(NavigationError::RecoveryBlocked);
        }
        if guards.pending
            || self.pending.is_some()
            || self.intent == Some(ShellIntent::CloseRequested)
        {
            return Err(NavigationError::Busy);
        }
        if guards.dirty {
            return Err(NavigationError::UnsavedEdits);
        }
        Ok(())
    }
    pub fn request_route(
        &mut self,
        catalog: &RouteCatalog,
        id: &str,
        guards: NavigationGuards,
    ) -> Result<RouteTicket, NavigationError> {
        self.guard(guards)?;
        let route = catalog.get(id).ok_or(NavigationError::UnknownRoute)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(NavigationError::GenerationExhausted)?;
        let ticket = RouteTicket {
            owner: self.identity.clone(),
            generation,
        };
        self.generation = generation;
        self.pending = Some(PendingRoute {
            target: route.id().clone(),
            ticket: ticket.clone(),
        });
        self.phase = ShellPhase::AwaitingHost;
        self.diagnostic = None;
        Ok(ticket)
    }
    fn validate(&self, ticket: &RouteTicket) -> Result<&PendingRoute, NavigationError> {
        if !Rc::ptr_eq(&self.identity, &ticket.owner) {
            return Err(NavigationError::ForeignObservation);
        }
        self.pending
            .as_ref()
            .filter(|pending| pending.ticket.generation == ticket.generation)
            .ok_or(NavigationError::StaleObservation)
    }
    pub fn cancel_route(&mut self, ticket: &RouteTicket) -> Result<(), NavigationError> {
        self.validate(ticket)?;
        self.pending = None;
        self.phase = if self.observed.is_some() {
            ShellPhase::Observed
        } else {
            ShellPhase::NoSelection
        };
        self.diagnostic = None;
        Ok(())
    }
    /// Checked intent metadata only; this neither reports a renderer nor commits the route.
    pub fn route_target(&self, ticket: &RouteTicket) -> Result<&RouteId, NavigationError> {
        Ok(&self.validate(ticket)?.target)
    }
    /// Trusted host input only; model acceptance is not a renderer/OS/cleanup receipt.
    pub fn observe_route(
        &mut self,
        ticket: &RouteTicket,
        observation: RouteHostObservation,
    ) -> Result<(), NavigationError> {
        let pending = self.validate(ticket)?;
        match observation {
            RouteHostObservation::Reported { route } => {
                if route != pending.target {
                    return Err(NavigationError::RouteMismatch);
                }
                self.observed = Some(route);
                self.phase = ShellPhase::Observed;
                self.diagnostic = None;
            }
            RouteHostObservation::Failed { diagnostic } => {
                self.phase = ShellPhase::Failed;
                self.diagnostic = Some(diagnostic);
            }
        }
        self.pending = None;
        Ok(())
    }
    /// An intent only; no native owner is released and no close completion is generated.
    pub fn request_close(&mut self, guards: NavigationGuards) -> Result<(), NavigationError> {
        self.guard(guards)?;
        self.intent = Some(ShellIntent::CloseRequested);
        Ok(())
    }
    pub fn request_panel(&mut self, panel: ShellPanel) -> Result<(), NavigationError> {
        if self.pending.is_some() || self.intent == Some(ShellIntent::CloseRequested) {
            return Err(NavigationError::Busy);
        }
        self.intent = Some(match panel {
            ShellPanel::Episodes => ShellIntent::EpisodeListRequested,
            ShellPanel::Settings => ShellIntent::SettingsRequested,
        });
        Ok(())
    }
    /// Clears a rejected UI request only; does not reopen or release any native owner.
    pub fn dismiss_close_request(&mut self) {
        if self.intent == Some(ShellIntent::CloseRequested) {
            self.intent = None;
        }
    }
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
