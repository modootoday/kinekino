use super::{CatalogConsumer, model::Intent};
use crate::{
    application::catalog::Category,
    assembly::meari::viewer_read_projection::native_playback::NativePlaybackProjection,
};
use meari::{
    application::{host_callbacks::HostCallbacks, plugins::*},
    domain::host_callbacks::{CallbackArguments, CallbackSignature},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn busy() -> PluginError {
    PluginError::new(PluginErrorCode::Busy, "Original catalog UI owner borrowed")
}
fn invalid() -> PluginError {
    PluginError::new(
        PluginErrorCode::InvalidPayload,
        "Invalid catalog callback input",
    )
}
fn input_error(error: crate::application::api::ApiError) -> PluginError {
    PluginError::new(
        PluginErrorCode::InvalidPayload,
        format!("UI selection: {}", error.message),
    )
}
pub(crate) fn bind(
    callbacks: &mut HostCallbacks,
    registration: RegistrationHandle,
    manifest: &PluginManifest,
    state: Rc<RefCell<CatalogConsumer>>,
    native: Rc<RefCell<NativePlaybackProjection>>,
    native_admission: Rc<Cell<bool>>,
    fault: Rc<Cell<bool>>,
) -> Result<(), PluginError> {
    super::super::episode_transition::callbacks::bind(
        callbacks,
        registration.clone(),
        manifest,
        state.clone(),
        native.clone(),
        native_admission,
        fault.clone(),
    )?;
    for (prop, category) in [
        ("onCatalogAiring", Category::Airing),
        ("onCatalogCompleted", Category::Completed),
        ("onCatalogMovies", Category::Movies),
    ] {
        let state = state.clone();
        let fault = fault.clone();
        let schema = manifest.capabilities["jobs.submit"].request_schema.clone();
        callbacks.bind(
            prop.into(),
            CallbackSignature::Action,
            registration.clone(),
            "jobs.submit".into(),
            move |arguments| {
                if fault.get() {
                    return Err(busy());
                }
                if !matches!(arguments, CallbackArguments::Action) {
                    return Err(invalid());
                }
                let mut owner = state.try_borrow_mut().map_err(|_| busy())?;
                owner.require_catalog_idle().map_err(input_error)?;
                if !owner.available || !owner.automatic_enabled {
                    return Err(PluginError::new(
                        PluginErrorCode::MissingCapability,
                        "라이브 카탈로그 지원 상태를 먼저 조회하세요",
                    ));
                }
                let input = owner
                    .model
                    .stage(Intent::Category { category, page: 1 })
                    .map_err(input_error)?;
                PluginPayload::encode(schema.clone(), &input)
            },
        )?;
    }
    for (prop, episode) in [("onCatalogSeries", false), ("onCatalogEpisode", true)] {
        let state = state.clone();
        let fault = fault.clone();
        let schema = manifest.capabilities["jobs.submit"].request_schema.clone();
        callbacks.bind(
            prop.into(),
            CallbackSignature::Text,
            registration.clone(),
            "jobs.submit".into(),
            move |arguments| {
                if fault.get() {
                    return Err(busy());
                }
                let CallbackArguments::Text(key) = arguments else {
                    return Err(invalid());
                };
                let mut owner = state.try_borrow_mut().map_err(|_| busy())?;
                owner.require_catalog_idle().map_err(input_error)?;
                if !owner.available || !owner.automatic_enabled {
                    return Err(PluginError::new(
                        PluginErrorCode::MissingCapability,
                        "라이브 카탈로그 연결이 없습니다",
                    ));
                }
                let intent = if episode {
                    owner.model.choose_episode(key)
                } else {
                    owner.model.choose_series(key)
                }
                .map_err(input_error)?;
                let input = owner.model.stage(intent).map_err(input_error)?;
                PluginPayload::encode(schema.clone(), &input)
            },
        )?;
    }
    macro_rules! action {
        ($prop:literal, $command:literal, $input:expr) => {{
            let state = state.clone();
            let fault = fault.clone();
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
                        return Err(invalid());
                    }
                    let owner = state.try_borrow().map_err(|_| busy())?;
                    owner.require_catalog_idle().map_err(input_error)?;
                    if owner.model.rpc.is_some() {
                        return Err(busy());
                    }
                    let input = ($input)(&owner).map_err(input_error)?;
                    PluginPayload::encode(schema.clone(), &input)
                },
            )?;
        }};
    }
    action!("onCatalogPoll", "jobs.get", |owner: &CatalogConsumer| owner
        .model
        .poll_input());
    action!(
        "onCatalogCancel",
        "jobs.cancel",
        |owner: &CatalogConsumer| owner.model.cancel_input()
    );
    action!(
        "onCatalogReadSeries",
        "catalog.search",
        |owner: &CatalogConsumer| owner.model.read_input("catalog.search")
    );
    action!(
        "onCatalogReadEpisodes",
        "episodes.list",
        |owner: &CatalogConsumer| owner.model.read_input("episodes.list")
    );
    action!(
        "onCatalogInspect",
        "episodes.inspect_resolution",
        |owner: &CatalogConsumer| owner.model.inspect_input()
    );
    let schema = manifest.capabilities["playback.open"]
        .request_schema
        .clone();
    callbacks.bind(
        "onCatalogPlay".into(),
        CallbackSignature::Action,
        registration,
        "playback.open".into(),
        move |arguments| {
            if fault.get() {
                return Err(busy());
            }
            if !matches!(arguments, CallbackArguments::Action) {
                return Err(invalid());
            }
            let owner = state.try_borrow().map_err(|_| busy())?;
            let native = native.try_borrow().map_err(|_| busy())?;
            if native.observation().is_some() {
                return Err(PluginError::new(
                    PluginErrorCode::MissingCapability,
                    "현재 재생 대상이 있습니다. 기존 이전/다음 전환을 사용하세요",
                ));
            }
            let volume = native.initial_volume()?;
            let input = owner
                .resolution()
                .map_err(input_error)?
                .open_input(volume, false);
            PluginPayload::encode(schema.clone(), &input)
        },
    )?;
    Ok(())
}
