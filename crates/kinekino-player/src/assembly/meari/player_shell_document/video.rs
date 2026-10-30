//! Historical declaration metadata from one actual document; no surface lease or execution authority.
use crate::presentation::web_ui_adapter::player_shell::document::video::ViewingVideoId;
use meari_native::infrastructure::native_host::{HostAccessError, HostStamp, NativeDocumentHandle};
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct MountedViewingVideo {
    id: ViewingVideoId,
    origin: HostStamp,
}

impl MountedViewingVideo {
    pub fn id(&self) -> ViewingVideoId {
        self.id
    }

    /// Historical data only. A later request must inspect the current original handle again.
    pub fn origin(&self) -> &HostStamp {
        &self.origin
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ViewingVideoError {
    Access(HostAccessError),
    Missing,
    WrongTag,
    MultipleVideos,
    ActiveDeclaration,
}

impl fmt::Display for ViewingVideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Access(error) => error.fmt(f),
            other => write!(f, "viewing video declaration: {other:?}"),
        }
    }
}

impl std::error::Error for ViewingVideoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Access(error) => Some(error),
            _ => None,
        }
    }
}

pub(super) fn inspect(
    document: &NativeDocumentHandle,
) -> Result<MountedViewingVideo, ViewingVideoError> {
    // The outer shared read remains live while stamp() takes another shared read. No mutation,
    // dispatch or external callback can change the inspected document between these observations.
    document
        .try_with(|mounted| {
            let id = ViewingVideoId::Primary;
            let element = mounted
                .elements()
                .iter()
                .find(|element| element.id == id.as_str())
                .ok_or(ViewingVideoError::Missing)?;
            if element.dom_tag != "video" {
                return Err(ViewingVideoError::WrongTag);
            }
            if mounted
                .elements()
                .iter()
                .filter(|element| element.dom_tag == "video")
                .count()
                != 1
            {
                return Err(ViewingVideoError::MultipleVideos);
            }
            if [
                "src",
                "srcObject",
                "srcobject",
                "poster",
                "autoplay",
                "controls",
            ]
            .iter()
            .any(|name| element.attributes.contains_key(*name))
                || mounted.elements().iter().any(|candidate| {
                    candidate.dom_tag == "source"
                        && candidate.path.starts_with(&format!("{}/", element.path))
                })
            {
                return Err(ViewingVideoError::ActiveDeclaration);
            }
            let projection = mounted
                .module_projection()
                .map_err(|message| ViewingVideoError::Access(HostAccessError::Invalid(message)))?;
            if projection
                .callback_bindings()
                .keys()
                .any(|path| element.owns_path(path))
                || projection.event_paths().any(|path| element.owns_path(path))
                || projection.input_paths().any(|path| element.owns_path(path))
            {
                return Err(ViewingVideoError::ActiveDeclaration);
            }
            let origin = document.stamp().map_err(ViewingVideoError::Access)?;
            Ok(MountedViewingVideo { id, origin })
        })
        .map_err(ViewingVideoError::Access)?
}
