//! Original native retirement tickets gate UI route changes; no lease cleanup policy here.
use crate::application::{
    api::{ApiError, ErrorCode},
    route_media::{RetireTicket, RouteMediaPort, RouteMediaProgress, ViewingTicket},
};
use meari::application::plugins::PluginScope;
use std::{rc::Rc, sync::Arc};

#[derive(Default)]
pub(super) struct NativeRouteGate {
    port: Option<Rc<dyn RouteMediaPort<PluginScope>>>,
    headless: bool,
    retirement: Option<RetireTicket<PluginScope>>,
    viewing: Option<ViewingTicket<PluginScope>>,
    retired: bool,
    viewing_complete: bool,
    first_error: Option<Arc<ApiError>>,
    owner: Option<PluginScope>,
}
fn invalid(message: &str) -> Arc<ApiError> {
    Arc::new(ApiError::new(ErrorCode::Unsupported, message))
}
impl NativeRouteGate {
    #[cfg(test)]
    pub(super) fn retirement_generation(&self) -> Option<u64> {
        self.retirement.as_ref().map(RetireTicket::generation)
    }
    pub(super) fn install(
        &mut self,
        port: Rc<dyn RouteMediaPort<PluginScope>>,
    ) -> Result<(), Arc<ApiError>> {
        if self.port.is_some() || self.headless || self.retirement.is_some() {
            return Err(invalid(
                "Original route media configuration is already fixed",
            ));
        }
        self.port = Some(port);
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn declare_headless(&mut self) -> Result<(), Arc<ApiError>> {
        if self.port.is_some() || self.retirement.is_some() {
            return Err(invalid("A native owner cannot become a headless fixture"));
        }
        self.headless = true;
        Ok(())
    }
    fn preserve<T>(&mut self, result: Result<T, Arc<ApiError>>) -> Result<T, Arc<ApiError>> {
        result.inspect_err(|error| {
            self.first_error.get_or_insert_with(|| error.clone());
        })
    }
    pub(super) fn retire(&mut self, scope: &PluginScope) -> Result<bool, Arc<ApiError>> {
        if self.owner.as_ref().is_some_and(|owner| owner != scope) {
            return Err(Arc::new(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original native route owner differs",
            )));
        }
        if let Some(error) = &self.first_error {
            return Err(error.clone());
        }
        if self.headless {
            return Ok(true);
        }
        if self.viewing.is_some() && !self.viewing_complete {
            return Err(Arc::new(ApiError::new(
                ErrorCode::NeedsInput,
                "Fresh Viewing registration is still pending",
            )));
        }
        let port = self
            .port
            .clone()
            .ok_or_else(|| invalid("Original native route media port is not configured"))?;
        if self.viewing_complete {
            // Do not discard the completed generation until the same port accepts
            // a new retirement request. An admission error retains the old tickets.
            let ticket = self.preserve(port.begin_retire(scope))?;
            self.retirement = Some(ticket);
            self.viewing = None;
            self.viewing_complete = false;
            self.retired = false;
        }
        if self.retired {
            return Ok(true);
        }
        if self.retirement.is_none() {
            let ticket = self.preserve(port.begin_retire(scope))?;
            self.retirement = Some(ticket);
            self.owner = Some(scope.clone());
        }
        let ticket = self
            .retirement
            .as_ref()
            .expect("Original ticket retained after begin");
        let result = port.poll_retire(ticket);
        match self.preserve(result)? {
            RouteMediaProgress::Pending => Ok(false),
            RouteMediaProgress::Complete => {
                self.retired = true;
                Ok(true)
            }
        }
    }
    pub(super) fn restore_viewing(&mut self, scope: &PluginScope) -> Result<bool, Arc<ApiError>> {
        if self.owner.as_ref().is_some_and(|owner| owner != scope) {
            return Err(Arc::new(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original native route owner differs",
            )));
        }
        if let Some(error) = &self.first_error {
            return Err(error.clone());
        }
        if self.headless || self.viewing_complete {
            return Ok(true);
        }
        let port = self
            .port
            .clone()
            .ok_or_else(|| invalid("Original native route media port is not configured"))?;
        if !self.retired {
            return Err(invalid("Original native retirement has not completed"));
        }
        if self.viewing.is_none() {
            let retirement = self
                .retirement
                .as_ref()
                .expect("Completed retirement retains its original ticket");
            let result = port.begin_viewing(scope, retirement);
            let ticket = self.preserve(result)?;
            self.viewing = Some(ticket);
        }
        let ticket = self
            .viewing
            .as_ref()
            .expect("Original viewing ticket retained after begin");
        let result = port.poll_viewing(ticket);
        match self.preserve(result)? {
            RouteMediaProgress::Pending => Ok(false),
            RouteMediaProgress::Complete => {
                self.viewing_complete = true;
                Ok(true)
            }
        }
    }
}

impl super::ViewerHost {
    pub(crate) fn install_route_media_port(
        &mut self,
        port: Rc<dyn RouteMediaPort<PluginScope>>,
    ) -> Result<(), Arc<ApiError>> {
        if self.shutdown_started {
            return Err(invalid("Original viewer is closing"));
        }
        self.native_route.install(port)
    }
    #[cfg(test)]
    pub(crate) fn declare_headless_route_media(&mut self) -> Result<(), Arc<ApiError>> {
        self.native_route.declare_headless()
    }
}

#[cfg(test)]
pub(super) mod tests;
