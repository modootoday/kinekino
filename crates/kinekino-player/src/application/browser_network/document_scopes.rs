use super::{NetworkEvent, ProtocolSession, target_bindings::TargetBindings};
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{navigation, player_document::PlayerDocumentLocation},
};
use kinekino_core::source::document::{PageLocation, ResourceKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolDocumentScope {
    #[serde(default)]
    pub session: ProtocolSession,
    pub root_revision: u64,
    pub root_frame_id: String,
    pub root_loader_id: String,
    pub frame_id: String,
    pub loader_id: String,
    pub url: String,
}
#[derive(Clone)]
struct Root {
    frame: String,
    loader: String,
    url: String,
}

/// Scope is derived within one owned protocol session, never by native/CDP ID equality.
#[derive(Clone)]
pub struct DocumentScopes {
    initial_url: String,
    initial_observed: bool,
    revision: u64,
    root: Option<Root>,
    children: Vec<ProtocolDocumentScope>,
    targets: TargetBindings,
}
impl DocumentScopes {
    pub fn new(initial_url: &str) -> Result<Self, ApiError> {
        let location = PageLocation::parse(initial_url).map_err(|_| invalid())?;
        if location.kind() != ResourceKind::Episode {
            return Err(invalid());
        }
        Ok(Self {
            initial_url: location.as_str().into(),
            initial_observed: false,
            revision: 0,
            root: None,
            children: Vec::new(),
            targets: TargetBindings::default(),
        })
    }
    pub fn observe(&mut self, event: &NetworkEvent) -> Result<(), ApiError> {
        self.observe_session(&ProtocolSession::Default, event)
    }
    pub fn observe_session(
        &mut self,
        session: &ProtocolSession,
        event: &NetworkEvent,
    ) -> Result<(), ApiError> {
        if session != &ProtocolSession::Default {
            return self.observe_target(session, event);
        }
        if let Some(root) = &self.root
            && navigation::player_page(&root.url)
        {
            self.targets.observe_default(event, &root.frame)?;
        }
        match event {
            NetworkEvent::FrameNavigated {
                frame_id,
                parent_frame_id,
                loader_id,
                url,
            } => {
                if parent_frame_id.is_none() {
                    if !self.initial_observed && url != &self.initial_url {
                        return Ok(());
                    }
                    if !navigation::navigation_allowed(url) {
                        return Err(ApiError::new(
                            ErrorCode::PermissionDenied,
                            "Owned protocol root left its admitted scope",
                        ));
                    }
                    if self.root.as_ref().is_some_and(|root| {
                        root.frame == *frame_id && root.loader == *loader_id && root.url == *url
                    }) {
                        return Ok(());
                    }
                    self.revision = self.revision.checked_add(1).ok_or_else(invalid)?;
                    if self.revision > 32 {
                        return Err(limit());
                    }
                    self.initial_observed = true;
                    self.root = Some(Root {
                        frame: frame_id.clone(),
                        loader: loader_id.clone(),
                        url: url.clone(),
                    });
                    self.children.clear();
                    self.targets.clear();
                } else {
                    // A new navigation first retires this child's previous loader, even outside scope.
                    self.children.retain(|child| child.frame_id != *frame_id);
                    let Some(root) = &self.root else {
                        return Ok(());
                    };
                    if parent_frame_id.as_deref() != Some(root.frame.as_str())
                        || !navigation::player_page(&root.url)
                        || PlayerDocumentLocation::parse(url).is_err()
                    {
                        return Ok(());
                    }
                    if self.children.len() >= 8 {
                        return Err(limit());
                    }
                    self.children.push(ProtocolDocumentScope {
                        session: ProtocolSession::Default,
                        root_revision: self.revision,
                        root_frame_id: root.frame.clone(),
                        root_loader_id: root.loader.clone(),
                        frame_id: frame_id.clone(),
                        loader_id: loader_id.clone(),
                        url: url.clone(),
                    });
                }
            }
            NetworkEvent::FrameDetached { frame_id, reason } => {
                self.children.retain(|child| {
                    child.frame_id != *frame_id
                        || reason == "swap" && child.session != ProtocolSession::Default
                });
                if self
                    .root
                    .as_ref()
                    .is_some_and(|root| root.frame == *frame_id)
                {
                    self.root = None;
                    self.children.clear();
                    self.targets.clear();
                }
            }
            NetworkEvent::TargetDetached { session_id, .. } => {
                self.children
                    .retain(|child| child.session != ProtocolSession::Target(session_id.clone()));
            }
            _ => {}
        }
        Ok(())
    }
    fn observe_target(
        &mut self,
        session: &ProtocolSession,
        event: &NetworkEvent,
    ) -> Result<(), ApiError> {
        let Some(root) = &self.root else {
            return Ok(());
        };
        if !navigation::player_page(&root.url) {
            return Ok(());
        }
        self.targets.observe_tree(session, event, &root.frame);
        let (frame, parent, loader, url) = match event {
            NetworkEvent::FrameTree {
                frame_id,
                parent_frame_id,
                loader_id,
                url,
            }
            | NetworkEvent::FrameNavigated {
                frame_id,
                parent_frame_id,
                loader_id,
                url,
            } => (frame_id, parent_frame_id, loader_id, url),
            NetworkEvent::FrameDetached { frame_id, .. } => {
                self.children
                    .retain(|child| &child.session != session || child.frame_id != *frame_id);
                self.targets.retire_session(session);
                return Ok(());
            }
            _ => return Ok(()),
        };
        if !self.targets.bound(session, frame) {
            return Ok(());
        }
        self.children
            .retain(|child| &child.session != session || child.frame_id != *frame);
        if parent.as_deref().is_some_and(|parent| parent != root.frame)
            || loader.is_empty()
            || PlayerDocumentLocation::parse(url).is_err()
        {
            return Ok(());
        }
        if self.children.len() >= 8 {
            return Err(limit());
        }
        self.children.push(ProtocolDocumentScope {
            session: session.clone(),
            root_revision: self.revision,
            root_frame_id: root.frame.clone(),
            root_loader_id: root.loader.clone(),
            frame_id: frame.clone(),
            loader_id: loader.clone(),
            url: url.clone(),
        });
        Ok(())
    }
    pub fn current(&self, frame: &str, loader: &str) -> Option<ProtocolDocumentScope> {
        self.current_in_session(&ProtocolSession::Default, frame, loader)
    }
    pub fn current_in_session(
        &self,
        session: &ProtocolSession,
        frame: &str,
        loader: &str,
    ) -> Option<ProtocolDocumentScope> {
        self.children
            .iter()
            .find(|scope| {
                &scope.session == session && scope.frame_id == frame && scope.loader_id == loader
            })
            .cloned()
    }
    pub fn is_current(&self, scope: &ProtocolDocumentScope) -> bool {
        self.current_in_session(&scope.session, &scope.frame_id, &scope.loader_id)
            .as_ref()
            == Some(scope)
    }
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid owned protocol document identity",
    )
}
fn limit() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Owned protocol document scope limit exceeded",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn target() -> ProtocolSession {
        ProtocolSession::Target("session-different-from-frame".into())
    }
    fn attach(scopes: &mut DocumentScopes) {
        scopes
            .observe(&NetworkEvent::FrameAttached {
                frame_id: "protocol-child".into(),
                parent_frame_id: "protocol-root".into(),
            })
            .unwrap();
        scopes
            .observe(&NetworkEvent::TargetAttached {
                session_id: "session-different-from-frame".into(),
                target_id: "opaque-target-not-frame".into(),
                target_type: "iframe".into(),
                waiting_for_debugger: true,
            })
            .unwrap();
    }
    fn tree(parent: Option<&str>) -> NetworkEvent {
        NetworkEvent::FrameTree {
            frame_id: "protocol-child".into(),
            parent_frame_id: parent.map(Into::into),
            loader_id: "child-loader".into(),
            url: "https://media.example.test/video/0123456789abcdef0123456789abcdef".into(),
        }
    }
    #[test]
    fn named_target_requires_tree_proof_and_keeps_session_namespace() {
        let mut scopes = scopes();
        attach(&mut scopes);
        scopes
            .observe_session(
                &target(),
                &navigated(
                    "protocol-child",
                    None,
                    "child-loader",
                    "https://media.example.test/video/0123456789abcdef0123456789abcdef",
                ),
            )
            .unwrap();
        assert!(
            scopes
                .current_in_session(&target(), "protocol-child", "child-loader")
                .is_none()
        );
        scopes.observe_session(&target(), &tree(None)).unwrap();
        let scope = scopes
            .current_in_session(&target(), "protocol-child", "child-loader")
            .unwrap();
        assert_eq!(scope.session, target());
        assert!(scopes.current("protocol-child", "child-loader").is_none());
        scopes
            .observe(&NetworkEvent::FrameDetached {
                frame_id: "protocol-child".into(),
                reason: "swap".into(),
            })
            .unwrap();
        assert!(scopes.is_current(&scope));
        scopes
            .observe(&NetworkEvent::TargetDetached {
                session_id: "session-different-from-frame".into(),
                target_id: None,
            })
            .unwrap();
        assert!(!scopes.is_current(&scope));
        scopes.observe_session(&target(), &tree(None)).unwrap();
        assert!(!scopes.is_current(&scope));
    }
    #[test]
    fn foreign_tree_parent_and_root_reload_reject_late_binding() {
        let mut scopes = scopes();
        attach(&mut scopes);
        scopes
            .observe_session(&target(), &tree(Some("foreign-parent")))
            .unwrap();
        assert!(
            scopes
                .current_in_session(&target(), "protocol-child", "child-loader")
                .is_none()
        );
        scopes
            .observe(&navigated(
                "protocol-root",
                None,
                "wrapper-reload",
                "https://frame.example.test/tv-show.php?v=fixture",
            ))
            .unwrap();
        scopes
            .observe_session(&target(), &tree(Some("protocol-root")))
            .unwrap();
        assert!(
            scopes
                .current_in_session(&target(), "protocol-child", "child-loader")
                .is_none()
        );
    }
    #[test]
    fn named_frame_removal_cannot_be_revived_by_late_tree() {
        let mut scopes = scopes();
        attach(&mut scopes);
        scopes.observe_session(&target(), &tree(None)).unwrap();
        let old = scopes
            .current_in_session(&target(), "protocol-child", "child-loader")
            .unwrap();
        scopes
            .observe_session(
                &target(),
                &NetworkEvent::FrameDetached {
                    frame_id: "protocol-child".into(),
                    reason: "remove".into(),
                },
            )
            .unwrap();
        scopes.observe_session(&target(), &tree(None)).unwrap();
        assert!(!scopes.is_current(&old));
    }
    fn navigated(frame: &str, parent: Option<&str>, loader: &str, url: &str) -> NetworkEvent {
        NetworkEvent::FrameNavigated {
            frame_id: frame.into(),
            parent_frame_id: parent.map(Into::into),
            loader_id: loader.into(),
            url: url.into(),
        }
    }
    fn scopes() -> DocumentScopes {
        let mut scopes = DocumentScopes::new("https://catalog.example.test/e/fixture").unwrap();
        scopes
            .observe(&navigated(
                "protocol-root",
                None,
                "episode-loader",
                "https://catalog.example.test/e/fixture",
            ))
            .unwrap();
        scopes
            .observe(&navigated(
                "protocol-root",
                None,
                "wrapper-loader",
                "https://frame.example.test/tv-show.php?v=fixture",
            ))
            .unwrap();
        scopes
    }
    #[test]
    fn root_and_child_loader_changes_invalidate_previous_scopes_even_at_same_url() {
        let mut scopes = scopes();
        let url = "https://media.example.test/video/0123456789abcdef0123456789abcdef";
        scopes
            .observe(&navigated(
                "protocol-child",
                Some("protocol-root"),
                "child-loader",
                url,
            ))
            .unwrap();
        let old = scopes.current("protocol-child", "child-loader").unwrap();
        scopes
            .observe(&navigated(
                "protocol-child",
                Some("protocol-root"),
                "child-reload",
                url,
            ))
            .unwrap();
        assert!(!scopes.is_current(&old));
        let current = scopes.current("protocol-child", "child-reload").unwrap();
        scopes
            .observe(&navigated(
                "protocol-root",
                None,
                "wrapper-reload",
                "https://frame.example.test/tv-show.php?v=fixture",
            ))
            .unwrap();
        assert!(!scopes.is_current(&current));
    }
    #[test]
    fn foreign_parent_and_out_of_scope_navigation_never_inherit_a_previous_document() {
        let mut scopes = scopes();
        let url = "https://media.example.test/video/0123456789abcdef0123456789abcdef";
        scopes
            .observe(&navigated(
                "protocol-child",
                Some("other-parent"),
                "loader",
                url,
            ))
            .unwrap();
        assert!(scopes.current("protocol-child", "loader").is_none());
        scopes
            .observe(&navigated(
                "protocol-child",
                Some("protocol-root"),
                "loader",
                url,
            ))
            .unwrap();
        let previous = scopes.current("protocol-child", "loader").unwrap();
        scopes
            .observe(&navigated(
                "protocol-child",
                Some("protocol-root"),
                "other-loader",
                "https://ads.example/",
            ))
            .unwrap();
        assert!(!scopes.is_current(&previous));
    }
}
