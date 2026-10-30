//! Preparation-only owner bundle. Does not create a second SDK/host or mount a window.
use crate::{
    application::api::ApplicationClientHandle,
    presentation::web_ui_adapter::player_shell::{
        CatalogError, NavigationGuards, RouteCatalog, ShellState,
        controller::{StartPreparationError, StartRequest, request_start},
        document::StartLabels,
    },
};

/// H remains on the caller/UI thread, with its actual cleanup authority intact.
pub struct ShellInputs<H> {
    pub client: ApplicationClientHandle,
    pub host_owner: H,
}

pub enum ShellStartupError {
    Catalog(CatalogError),
    Start(StartPreparationError),
}

#[must_use = "Preserve the supplied host owner and its cleanup authority"]
pub struct ShellPreparationFailure<H> {
    pub error: ShellStartupError,
    pub inputs: ShellInputs<H>,
    pub state: ShellState,
}

#[must_use = "Preparation is not mount; retain owners until explicit handoff"]
pub struct PreparedShell<H> {
    pub inputs: ShellInputs<H>,
    pub state: ShellState,
    pub request: StartRequest,
}

pub fn prepare_shell<H>(
    inputs: ShellInputs<H>,
    labels: StartLabels,
) -> Result<PreparedShell<H>, Box<ShellPreparationFailure<H>>> {
    let mut state = ShellState::new();
    let catalog = match RouteCatalog::bundled() {
        Ok(catalog) => catalog,
        Err(error) => {
            return Err(Box::new(ShellPreparationFailure {
                error: ShellStartupError::Catalog(error),
                inputs,
                state,
            }));
        }
    };
    let result = request_start(&mut state, &catalog, labels, NavigationGuards::default());
    retain_preparation(inputs, state, result)
}

fn retain_preparation<H>(
    inputs: ShellInputs<H>,
    state: ShellState,
    result: Result<StartRequest, StartPreparationError>,
) -> Result<PreparedShell<H>, Box<ShellPreparationFailure<H>>> {
    match result {
        Ok(request) => Ok(PreparedShell {
            inputs,
            state,
            request,
        }),
        Err(error) => Err(Box::new(ShellPreparationFailure {
            error: ShellStartupError::Start(error),
            inputs,
            state,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::api::PlayerSdk;
    use std::{rc::Rc, sync::Arc};

    #[test]
    fn preparation_failure_returns_original_client_and_host_owner() {
        use crate::presentation::web_ui_adapter::player_shell::document::DocumentPreparationError;
        let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
        let owner = Rc::new(());
        let result = retain_preparation(
            ShellInputs {
                client: client.clone(),
                host_owner: owner.clone(),
            },
            ShellState::new(),
            Err(StartPreparationError::Document(
                DocumentPreparationError::SourceNotIntegrated,
            )),
        );
        let failure = match result {
            Err(failure) => failure,
            Ok(_) => panic!("preparation error must be retained"),
        };
        assert!(Arc::ptr_eq(&client, &failure.inputs.client));
        assert!(Rc::ptr_eq(&owner, &failure.inputs.host_owner));
        assert!(failure.state.observed_route().is_none());
        assert_eq!(Rc::strong_count(&owner), 2);
    }

    #[test]
    fn supplied_client_and_ui_owner_are_retained_without_mount() {
        let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
        let owner = Rc::new(());
        let result = prepare_shell(
            ShellInputs {
                client: client.clone(),
                host_owner: owner.clone(),
            },
            StartLabels {
                connection: "관측 없음".into(),
                time: "관측 없음".into(),
            },
        );
        // Both outcomes retain the exact caller owners; parse support is tested separately.
        let inputs = match result {
            Ok(prepared) => {
                assert!(prepared.state.observed_route().is_none());
                prepared.inputs
            }
            Err(failure) => failure.inputs,
        };
        assert!(Arc::ptr_eq(&client, &inputs.client));
        assert!(Rc::ptr_eq(&owner, &inputs.host_owner));
        assert_eq!(Rc::strong_count(&owner), 2);
        drop(inputs);
        assert_eq!(Rc::strong_count(&owner), 1);
    }
}
