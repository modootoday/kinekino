use super::super::catalog::CatalogConsumer;
use crate::application::playback_session::transition::EpisodeDirection;
use crate::assembly::meari::viewer_read_projection::native_playback::NativePlaybackProjection;
use meari::{
    application::{host_callbacks::HostCallbacks, plugins::*},
    domain::host_callbacks::{CallbackArguments, CallbackSignature},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn busy() -> PluginError {
    PluginError::new(
        PluginErrorCode::Busy,
        "Original transition UI is borrowed or closed",
    )
}
fn error(error: crate::application::api::ApiError) -> PluginError {
    PluginError::new(PluginErrorCode::InvalidPayload, error.message)
}
pub(crate) fn bind(
    callbacks: &mut HostCallbacks,
    registration: RegistrationHandle,
    manifest: &PluginManifest,
    catalog: Rc<RefCell<CatalogConsumer>>,
    native: Rc<RefCell<NativePlaybackProjection>>,
    native_admission: Rc<Cell<bool>>,
    projection_fault: Rc<Cell<bool>>,
) -> Result<(), PluginError> {
    for (prop, command, direction) in [
        (
            "onPreparePrevious",
            "playback.previous",
            EpisodeDirection::Previous,
        ),
        ("onPrepareNext", "playback.next", EpisodeDirection::Next),
    ] {
        let catalog = catalog.clone();
        let native = native.clone();
        let admitted = native_admission.clone();
        let fault = projection_fault.clone();
        let schema = manifest.capabilities[command].request_schema.clone();
        callbacks.bind(
            prop.into(),
            CallbackSignature::Action,
            registration.clone(),
            command.into(),
            move |arguments| {
                if fault.get() || !admitted.get() {
                    return Err(busy());
                }
                if !matches!(arguments, CallbackArguments::Action) {
                    return Err(PluginError::new(
                        PluginErrorCode::InvalidPayload,
                        "Transition expects original Action",
                    ));
                }
                let native = native.try_borrow().map_err(|_| busy())?;
                let actual = native.settled_observation().ok_or_else(busy)?;
                let input = catalog
                    .try_borrow_mut()
                    .map_err(|_| busy())?
                    .stage_adjacent(actual, direction)
                    .map_err(error)?;
                PluginPayload::encode(schema.clone(), &input)
            },
        )?;
    }
    macro_rules! action {
        ($prop:literal, $command:literal, $prepare:expr) => {{
            let catalog = catalog.clone();
            let fault = projection_fault.clone();
            let schema = manifest.capabilities[$command].request_schema.clone();
            callbacks.bind(
                $prop.into(),
                CallbackSignature::Action,
                registration.clone(),
                $command.into(),
                move |arguments| {
                    if fault.get() {
                        return Err(busy());
                    }
                    if !matches!(arguments, CallbackArguments::Action) {
                        return Err(PluginError::new(
                            PluginErrorCode::InvalidPayload,
                            "Transition expects original Action",
                        ));
                    }
                    let mut catalog = catalog.try_borrow_mut().map_err(|_| busy())?;
                    let input = ($prepare)(&mut catalog).map_err(error)?;
                    PluginPayload::encode(schema.clone(), &input)
                },
            )?;
        }};
    }
    action!(
        "onObserveTransition",
        "playback.transition.observe",
        |owner: &mut CatalogConsumer| owner
            .stage_transition_handle(super::TransitionRequest::Observe)
    );
    action!(
        "onCommitTransition",
        "playback.transition.commit",
        |owner: &mut CatalogConsumer| owner
            .stage_transition_handle(super::TransitionRequest::Commit)
    );
    action!(
        "onReleaseTransition",
        "playback.transition.release",
        |owner: &mut CatalogConsumer| owner
            .stage_transition_handle(super::TransitionRequest::Release)
    );
    action!(
        "onCancelTransitionJob",
        "jobs.cancel",
        |owner: &mut CatalogConsumer| owner.stage_transition_cancel()
    );
    action!(
        "onReadTransitionJob",
        "jobs.get",
        |owner: &mut CatalogConsumer| owner.stage_transition_job()
    );
    Ok(())
}
