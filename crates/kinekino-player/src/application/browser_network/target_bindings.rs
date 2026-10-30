//! Related-target identity proof. Target IDs are never interpreted as frame IDs.
use super::{NetworkEvent, ProtocolSession};
use crate::application::api::{ApiError, ErrorCode};

#[derive(Clone)]
struct Target {
    session: ProtocolSession,
    frame: Option<String>,
}

#[derive(Clone, Default)]
pub(super) struct TargetBindings {
    attachments: Vec<String>,
    targets: Vec<Target>,
}
impl TargetBindings {
    pub fn clear(&mut self) {
        self.attachments.clear();
        self.targets.clear();
    }

    pub fn observe_default(&mut self, event: &NetworkEvent, root: &str) -> Result<(), ApiError> {
        match event {
            NetworkEvent::FrameAttached {
                frame_id,
                parent_frame_id,
            } if parent_frame_id == root => {
                if !self.attachments.contains(frame_id) {
                    if self.attachments.len() >= 8 {
                        return Err(limit());
                    }
                    self.attachments.push(frame_id.clone());
                }
            }
            NetworkEvent::FrameDetached { frame_id, reason } if reason != "swap" => {
                self.attachments.retain(|frame| frame != frame_id);
                // A late tree response from a removed frame cannot bind to a reused frame ID.
                self.targets
                    .retain(|target| target.frame.as_ref().is_some_and(|frame| frame != frame_id));
            }
            NetworkEvent::TargetAttached {
                session_id,
                target_type,
                ..
            } if target_type == "iframe" => {
                let session = ProtocolSession::Target(session_id.clone());
                if self.targets.iter().any(|target| target.session == session) {
                    return Ok(());
                }
                if self.targets.len() >= 8 {
                    return Err(limit());
                }
                self.targets.push(Target {
                    session,
                    frame: None,
                });
            }
            NetworkEvent::TargetDetached { session_id, .. } => {
                self.targets
                    .retain(|target| target.session != ProtocolSession::Target(session_id.clone()));
            }
            _ => {}
        }
        Ok(())
    }

    pub fn observe_tree(&mut self, session: &ProtocolSession, event: &NetworkEvent, root: &str) {
        let NetworkEvent::FrameTree {
            frame_id,
            parent_frame_id,
            ..
        } = event
        else {
            return;
        };
        let Some(target) = self
            .targets
            .iter_mut()
            .find(|target| &target.session == session)
        else {
            return;
        };
        if parent_frame_id
            .as_deref()
            .is_some_and(|parent| parent != root)
            || (parent_frame_id.is_none() && !self.attachments.contains(frame_id))
        {
            return;
        }
        if target.frame.as_ref().is_none_or(|frame| frame == frame_id) {
            target.frame = Some(frame_id.clone());
        }
    }

    pub fn bound(&self, session: &ProtocolSession, frame: &str) -> bool {
        self.targets
            .iter()
            .any(|target| &target.session == session && target.frame.as_deref() == Some(frame))
    }
    pub fn retire_session(&mut self, session: &ProtocolSession) {
        self.targets.retain(|target| &target.session != session);
    }
}

fn limit() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Owned protocol target binding limit exceeded",
    )
}
