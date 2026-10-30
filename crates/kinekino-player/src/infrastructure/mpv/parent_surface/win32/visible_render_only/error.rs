use super::*;
#[derive(Debug)]
pub enum VisibleError {
    Input(SurfaceInputError),
    Parent(NativeParentClientError),
    Child(ProbeError),
    Geometry,
    CreationStyleMissing,
    OriginalFrameChanged,
    Identity,
    FocusChanged,
    Phase,
    StyleWrite(u32),
    WindowLongRead(u32),
    InputObserved(u32),
    ObserverUnavailable,
    Os(windows::core::Error),
}
impl std::fmt::Display for VisibleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "visible render-only child: {self:?}")
    }
}
impl std::error::Error for VisibleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(e) => Some(e),
            Self::Parent(e) => Some(e),
            Self::Child(e) => Some(e),
            Self::Os(e) => Some(e),
            _ => None,
        }
    }
}
pub(super) type Result<T> = std::result::Result<T, VisibleError>;
pub(super) type SharedResult<T> = std::result::Result<T, Arc<VisibleError>>;
