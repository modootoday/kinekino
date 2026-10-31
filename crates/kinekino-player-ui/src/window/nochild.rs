//! The player window's (empty) native child set.

use super::*;

/// The player window registers no child windows.
pub(super) struct NoChild;
impl NativeChild for NoChild {
    type Error = std::io::Error;
    fn parent_lost(&mut self) {}
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        _: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        Err(std::io::Error::other("player window has no child windows"))
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        Err(std::io::Error::other("player window has no child windows"))
    }
}
