//! Closed Viewing/Studio-Card source declarations for the existing product startup.
use crate::presentation::web_ui_adapter::{
    player_shell::document::{
        DocumentPreparationError, PreparedStartDocument, product_frame::ProductFrameBlueprint,
    },
    proposals::{
        bootstrap::BootstrapPanel, frame_composition::FrameComposition,
        product_window::product_modules_for_panel,
    },
};
use meari::{
    application::source_modules::ModuleDiagnostic,
    domain::{Diagnostic, Span},
};

pub(super) fn prepare(
    viewing: &PreparedStartDocument,
) -> Result<ProductFrameBlueprint, DocumentPreparationError> {
    let (entry, mut inventory) = product_modules_for_panel(BootstrapPanel::Card);
    let card =
        meari::parse_module_graph(&entry, &inventory).map_err(DocumentPreparationError::Source)?;
    let (studio, _) = FrameComposition::new(
        crate::domain::product::PRODUCT.display_name.into(),
        String::new(),
    )
    .prepare(card, &mut inventory)
    .map_err(DocumentPreparationError::Source)?;
    ProductFrameBlueprint::prepare(
        viewing.graph.clone(),
        viewing.inventory.clone(),
        studio,
        inventory,
    )
    .map_err(|message| {
        DocumentPreparationError::Source(ModuleDiagnostic {
            file: "product/frame.tsx"
                .to_owned()
                .try_into()
                .expect("fixed product frame source path"),
            diagnostic: Diagnostic::unsupported(Span::default(), message),
        })
    })
}

#[cfg(test)]
#[path = "product_frame/tests.rs"]
mod tests;
