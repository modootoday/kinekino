//! Related iframe observation barrier, on the owning WebView STA.
use super::*;
use crate::application::browser_network::NetworkEvent;

pub(super) struct TargetInitialization {
    session: ProtocolSession,
    tree_requested: bool,
    waiting: bool,
}

impl ProtocolObserver {
    pub fn observe_target_event(
        &self,
        session: &ProtocolSession,
        event: &NetworkEvent,
    ) -> Result<(), ApiError> {
        match event {
            NetworkEvent::TargetAttached {
                session_id,
                target_type,
                waiting_for_debugger,
                ..
            } if session == &ProtocolSession::Default && target_type == "iframe" => {
                let target = ProtocolSession::Target(session_id.clone());
                if self
                    .targets
                    .borrow()
                    .iter()
                    .any(|known| known.session == target)
                {
                    return Ok(());
                }
                self.enable_session(target.clone())?;
                self.targets.borrow_mut().push(TargetInitialization {
                    session: target,
                    tree_requested: false,
                    waiting: *waiting_for_debugger,
                });
            }
            NetworkEvent::TargetDetached { session_id, .. }
                if session == &ProtocolSession::Default =>
            {
                let target = ProtocolSession::Target(session_id.clone());
                self.targets
                    .borrow_mut()
                    .retain(|known| known.session != target);
                self.state
                    .enabled
                    .borrow_mut()
                    .retain(|(known, _)| known != &target);
            }
            NetworkEvent::FrameTree { .. } => {
                let resume = self
                    .targets
                    .borrow_mut()
                    .iter_mut()
                    .find(|target| &target.session == session && target.tree_requested)
                    .is_some_and(|target| std::mem::take(&mut target.waiting));
                if resume {
                    self.resume_target(session)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn drive_targets(&self) -> Result<(), ApiError> {
        let pending = self
            .targets
            .borrow()
            .iter()
            .filter(|target| !target.tree_requested && self.session_ready(&target.session))
            .map(|target| target.session.clone())
            .collect::<Vec<_>>();
        for session in pending {
            if let Some(target) = self
                .targets
                .borrow_mut()
                .iter_mut()
                .find(|target| target.session == session)
            {
                target.tree_requested = true;
            }
            let state = self.state.clone();
            let response_session = session.clone();
            let completed = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(
                move |result, json| {
                    if state.closed.get()
                        || !state
                            .enabled
                            .borrow()
                            .iter()
                            .any(|(known, _)| known == &response_session)
                    {
                        return Ok(());
                    }
                    if result.is_err() {
                        state.fail("Owned target frame tree unavailable");
                    } else if json.len() > 128 * 1024 {
                        state.fail("Owned target frame tree exceeds limit");
                    } else {
                        state.send(ProtocolMessage::Event {
                            received_unix_ms: now_ms(),
                            name: "Page.getFrameTree",
                            session: response_session,
                            json,
                        });
                    }
                    Ok(())
                },
            ));
            self.call_target(&session, "Page.getFrameTree", &completed)?;
        }
        Ok(())
    }

    fn resume_target(&self, session: &ProtocolSession) -> Result<(), ApiError> {
        let state = self.state.clone();
        let response_session = session.clone();
        let completed =
            CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result, _| {
                if result.is_err()
                    && !state.closed.get()
                    && state
                        .enabled
                        .borrow()
                        .iter()
                        .any(|(known, _)| known == &response_session)
                {
                    state.fail("Owned target observer resume failed");
                }
                Ok(())
            }));
        self.call_target(session, "Runtime.runIfWaitingForDebugger", &completed)
    }

    fn call_target(
        &self,
        session: &ProtocolSession,
        method: &str,
        completed: &ICoreWebView2CallDevToolsProtocolMethodCompletedHandler,
    ) -> Result<(), ApiError> {
        let ProtocolSession::Target(session) = session else {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Owned target session required",
            ));
        };
        // SAFETY: fixed observer commands with an actual attached session, on its creating STA.
        unsafe {
            self.core.cast::<ICoreWebView2_11>().and_then(|core| {
                core.CallDevToolsProtocolMethodForSession(
                    &HSTRING::from(session),
                    &HSTRING::from(method),
                    &HSTRING::from("{}"),
                    completed,
                )
            })
        }
        .map_err(|_| {
            error(
                ErrorCode::EngineFailed,
                "Owned target observer command could not start",
            )
        })
    }
}
