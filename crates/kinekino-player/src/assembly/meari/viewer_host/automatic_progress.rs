use super::{ViewerHost, automatic_wake::AutomaticWake};
use crate::assembly::meari::viewer_callbacks::ViewerCallbacks;
use meari::application::plugins::PluginError;
use meari_native::infrastructure::{
    native_host::{HostAccessError, HostTurn, NativeDocumentHandle},
    window::NativeWindowWake,
};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Instant};

#[derive(Default)]
pub(super) struct AutomaticProgress {
    source: Option<Rc<RefCell<Option<NativeWindowWake>>>>,
    document: Option<NativeDocumentHandle>,
    timer: Option<AutomaticWake>,
    timer_error: Option<Arc<HostAccessError>>,
    callback_error: Option<Arc<PluginError>>,
}
impl AutomaticProgress {
    fn prepare(&mut self, document: Option<NativeDocumentHandle>, allowed: bool) {
        self.document = document;
        if !allowed {
            self.stop();
            return;
        }
        if self.timer.is_some() || self.timer_error.is_some() || self.callback_error.is_some() {
            return;
        }
        let Some(document) = self.document.as_ref() else {
            return;
        };
        let Some(source) = self.source.as_ref() else {
            return;
        };
        let wake = match source.try_borrow() {
            Ok(source) => source.clone(),
            Err(_) => {
                self.timer_error = Some(Arc::new(HostAccessError::Busy));
                return;
            }
        };
        let Some(wake) = wake else {
            return;
        }; // Original attach has not happened yet.
        match AutomaticWake::new(&wake, document) {
            Ok(timer) => self.timer = Some(timer),
            Err(error) => self.timer_error = Some(Arc::new(error)),
        }
    }
    pub(super) fn stop(&mut self) {
        self.timer = None;
    }
    pub(super) fn retry_on_user_refresh(&mut self) {
        self.stop();
        self.timer_error = None;
        self.callback_error = None;
    }
    fn publish(&mut self, owner: &ViewerCallbacks) {
        if let Ok(mut projection) = owner.projection.try_borrow_mut() {
            let enabled =
                self.timer.is_some() && self.timer_error.is_none() && self.callback_error.is_none();
            match projection.catalog.try_borrow_mut() {
                Ok(mut catalog) => catalog.automatic_enabled = enabled,
                Err(_) => {
                    projection.catalog_projection_fault.set(true);
                }
            }
            let error = self
                .timer_error
                .as_ref()
                .map(|error| error.to_string())
                .or_else(|| self.callback_error.as_ref().map(|error| error.to_string()))
                .unwrap_or_default();
            if projection.automatic_enabled != enabled || projection.automatic_error != error {
                projection.automatic_enabled = enabled;
                projection.automatic_error = error;
                projection.dirty = true;
            }
        } else {
            self.callback_error.get_or_insert_with(|| {
                Arc::new(PluginError::new(
                    meari::application::plugins::PluginErrorCode::Busy,
                    "Original automatic projection borrowed",
                ))
            });
            self.stop();
        }
    }
    pub(super) fn advance(&mut self, owner: &mut ViewerCallbacks, turn: &HostTurn, allowed: bool) {
        self.prepare(self.document.clone(), allowed);
        if !allowed {
            self.publish(owner);
            return;
        }
        let Some(timer) = self.timer.as_mut() else {
            self.publish(owner);
            return;
        };
        let result = (|| {
            if let Some(expected) = if turn.reason
                == meari_native::infrastructure::native_host::HostTurnReason::Completion
            {
                timer
                    .take_due(Instant::now())
                    .map_err(AutomaticError::Timer)?
            } else {
                None
            } {
                let current = owner
                    .next_automatic_step(true)
                    .map_err(AutomaticError::Callback)?;
                if current.as_ref() == Some(&expected) {
                    owner
                        .dispatch_automatic_step(&turn.stamp.owner, &expected, true)
                        .map_err(AutomaticError::Callback)?;
                }
            }
            let next = owner
                .next_automatic_step(true)
                .map_err(AutomaticError::Callback)?;
            timer.reconcile(next).map_err(AutomaticError::Timer)
        })();
        match result {
            Ok(()) => (),
            Err(AutomaticError::Timer(error)) => {
                self.timer_error = Some(Arc::new(error));
                self.stop();
            }
            Err(AutomaticError::Callback(error)) => {
                self.callback_error = Some(Arc::new(error));
                self.stop();
            }
        }
        self.publish(owner);
    }
}
enum AutomaticError {
    Timer(HostAccessError),
    Callback(PluginError),
}
impl ViewerHost {
    pub(crate) fn bind_automatic_wake_source(
        &mut self,
        source: Rc<RefCell<Option<NativeWindowWake>>>,
    ) {
        self.automatic.source = Some(source);
    }
    pub(super) fn prepare_automatic_progress(&mut self) {
        let allowed = self.observed_viewing_for_chooser();
        self.automatic.prepare(self.document.clone(), allowed);
        if let Ok(owner) = self.callbacks.try_borrow() {
            self.automatic.publish(&owner);
        }
    }
}
