//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait WorkspaceReadPort: Send + Sync {
    fn snapshot(&self, workspace: WorkspaceId) -> WorkspaceSnapshot;
}
