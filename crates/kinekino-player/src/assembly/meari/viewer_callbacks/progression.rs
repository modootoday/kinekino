//! Presentation continuations only. Original typed callbacks still validate every SDK input.
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    ReadCapabilities,
    ReadEngine,
    PollCatalog,
    ReadSeries,
    ReadEpisodes,
    InspectEpisode,
    PlayEpisode,
    ObserveTransition,
    ReadTransitionJob,
    CommitTransition,
    ReleaseTransition,
}
impl Action {
    pub(crate) fn binding(self) -> &'static str {
        match self {
            Self::ReadCapabilities => "onReadCapabilities",
            Self::ReadEngine => "onReadEngine",
            Self::PollCatalog => "onCatalogPoll",
            Self::ReadSeries => "onCatalogReadSeries",
            Self::ReadEpisodes => "onCatalogReadEpisodes",
            Self::InspectEpisode => "onCatalogInspect",
            Self::PlayEpisode => "onCatalogPlay",
            Self::ObserveTransition => "onObserveTransition",
            Self::ReadTransitionJob => "onReadTransitionJob",
            Self::CommitTransition => "onCommitTransition",
            Self::ReleaseTransition => "onReleaseTransition",
        }
    }
    pub(crate) fn delay(self) -> Duration {
        match self {
            Self::PollCatalog
            | Self::ObserveTransition
            | Self::ReadTransitionJob
            | Self::ReadEngine => Duration::from_millis(250),
            _ => Duration::from_millis(1),
        }
    }
}

/// Correlates the retained operation, never an SDK authority or source-derived callback name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Continuation {
    pub action: Action,
    pub operation: String,
}
