//! ManualRepair owns its declared projection; typed editor props are never inherited.
mod bindings;
mod feedback;
mod projection;
use super::{
    IntentPins, ReviewScope, ReviewSession,
    panel_model::{PanelBudget, PanelLabels, PanelPreparationError, PreparedPanel},
};
pub use feedback::apply_feedback;
pub use projection::initial_repair_props;

pub fn prepare_repair_panel(
    review: &ReviewSession,
    scope: &ReviewScope,
    labels: PanelLabels,
    budget: PanelBudget,
    pins: IntentPins,
) -> Result<PreparedPanel, PanelPreparationError> {
    if review.scope() != scope {
        return Err(PanelPreparationError::View(super::ViewError::Stale));
    }
    let mut props = projection::project(review, &labels).map_err(PanelPreparationError::View)?;
    let bindings = bindings::prepare(review, budget, pins, &mut props)?;
    let refresh = review
        .display()
        .map_err(|_| PanelPreparationError::View(super::ViewError::InvalidSnapshot))?
        .refresh_required;
    Ok(PreparedPanel { props, bindings }
        .with_transport_pending(review.pending().is_some() || refresh))
}

#[cfg(all(test, windows, feature = "native-proposal-window"))]
mod tests;
