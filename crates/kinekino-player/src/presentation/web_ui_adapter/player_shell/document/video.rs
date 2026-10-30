//! Product semantic identifiers, independent of display naming and native surface authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewingVideoId {
    Primary,
}

impl ViewingVideoId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "viewing-primary-video",
        }
    }
}
