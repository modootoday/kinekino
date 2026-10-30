//! Actual CPU document construction without OS attach, callbacks or SDK execution.
pub mod video;
use super::player_shell_startup::{PreparedShell, ShellInputs};
use crate::presentation::web_ui_adapter::player_shell::document::product_frame::ProductFrameBlueprint;
use crate::{
    infrastructure::meari::product_styles::{prepare_product_frame_styles, prepare_viewing_styles},
    presentation::web_ui_adapter::player_shell::{RouteTicket, ShellState},
};
use meari::{application::source_modules::ModuleProgram, domain::build::ContentHash};
use meari_native::{
    domain::Viewport,
    infrastructure::{document::InteractiveDocument, native_host::NativeDocumentHandle},
};
use std::collections::BTreeMap;

pub struct DocumentOptions {
    pub namespace: String,
    pub epoch: u64,
    pub source_generation: u64,
    pub viewport: Viewport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentStage {
    Configuration,
    Styles,
    ModuleMount,
    InteractiveDocument,
    HostHandle,
}

#[must_use = "Retain the original external owners; constructor inputs may be consumed"]
pub struct DocumentFailure<H> {
    pub stage: DocumentStage,
    pub diagnostic: String,
    pub inputs: ShellInputs<H>,
    pub state: ShellState,
    pub ticket: RouteTicket,
}

#[must_use = "A prepared document handle is not an OS window or observed route"]
pub struct PreparedShellDocument<H> {
    pub inputs: ShellInputs<H>,
    pub state: ShellState,
    pub ticket: RouteTicket,
    pub document: NativeDocumentHandle,
    pub style_hashes: BTreeMap<String, ContentHash>,
}

impl<H> PreparedShellDocument<H> {
    /// Reads this original document without consuming owners or reserving a native surface.
    pub fn inspect_viewing_video(
        &self,
    ) -> Result<video::MountedViewingVideo, video::ViewingVideoError> {
        video::inspect(&self.document)
    }
}

pub fn mount_prepared_shell<H>(
    prepared: PreparedShell<H>,
    options: DocumentOptions,
) -> Result<PreparedShellDocument<H>, Box<DocumentFailure<H>>> {
    mount_prepared_shell_impl(prepared, options, None, None)
}
pub type DocumentCallbackExport = Box<
    dyn FnOnce(
        meari::domain::plugins::PluginScope,
    ) -> Result<meari::application::host_callbacks::CallbackBindings, String>,
>;
pub fn mount_prepared_shell_with_callbacks<H>(
    prepared: PreparedShell<H>,
    options: DocumentOptions,
    export: DocumentCallbackExport,
) -> Result<PreparedShellDocument<H>, Box<DocumentFailure<H>>> {
    mount_prepared_shell_impl(prepared, options, Some(export), None)
}
pub fn mount_prepared_product_shell_with_callbacks<H>(
    prepared: PreparedShell<H>,
    options: DocumentOptions,
    frame: &ProductFrameBlueprint,
    export: DocumentCallbackExport,
) -> Result<PreparedShellDocument<H>, Box<DocumentFailure<H>>> {
    mount_prepared_shell_impl(prepared, options, Some(export), Some(frame))
}
fn mount_prepared_shell_impl<H>(
    prepared: PreparedShell<H>,
    options: DocumentOptions,
    export: Option<DocumentCallbackExport>,
    frame: Option<&ProductFrameBlueprint>,
) -> Result<PreparedShellDocument<H>, Box<DocumentFailure<H>>> {
    let PreparedShell {
        inputs,
        state,
        request,
    } = prepared;
    // External owners never enter a consuming constructor or callback closure.
    let result = (|| {
        if options.namespace.trim().is_empty()
            || options.epoch == 0
            || options.source_generation == 0
            || options.viewport.width == 0
            || options.viewport.height == 0
        {
            return Err((
                DocumentStage::Configuration,
                "Explicit namespace, positive epochs and viewport required".into(),
            ));
        }
        let styles = match frame {
            Some(_) => prepare_product_frame_styles(),
            None => prepare_viewing_styles(),
        }
        .map_err(|e| (DocumentStage::Styles, e))?;
        let program = if let Some(frame) = frame {
            frame
                .mount_viewing(
                    request.document.props,
                    options.namespace,
                    options.epoch,
                    std::num::NonZeroU64::new(1).expect("initial workspace generation is positive"),
                )
                .map_err(|error| (DocumentStage::ModuleMount, error))?
        } else {
            ModuleProgram::mount_with_node_slots(
                request.document.graph,
                options.namespace,
                options.epoch,
                |factory| {
                    let mut props = request.document.props;
                    crate::presentation::web_ui_adapter::player_shell::start_screen::compose_slot(
                        &mut props,
                        |reference, props| factory.component("viewing-start", reference, props),
                    )?;
                    Ok(props)
                },
            )
            .map_err(|e| (DocumentStage::ModuleMount, format!("{e:?}")))?
        };
        // These APIs consume sole preparation values on error. They do not receive inputs.
        let document = match export {
            Some(export) => InteractiveDocument::from_module_program_with_callbacks(
                program,
                &styles.css,
                options.viewport,
                Some(styles.assets),
                options.source_generation,
                export,
            ),
            None => InteractiveDocument::from_module_program_with_assets(
                program,
                &styles.css,
                options.viewport,
                styles.assets,
            ),
        }
        .map_err(|e| (DocumentStage::InteractiveDocument, e))?;
        let document = NativeDocumentHandle::new(document, options.source_generation)
            .map_err(|e| (DocumentStage::HostHandle, format!("{e:?}")))?;
        Ok((document, styles.source_hashes))
    })();
    match result {
        Ok((document, style_hashes)) => Ok(PreparedShellDocument {
            inputs,
            state,
            ticket: request.ticket,
            document,
            style_hashes,
        }),
        Err((stage, diagnostic)) => Err(Box::new(DocumentFailure {
            stage,
            diagnostic,
            inputs,
            state,
            ticket: request.ticket,
        })),
    }
}

#[cfg(test)]
#[path = "player_shell_document_tests.rs"]
mod tests;
