//! Only the caller-selected watching locator changes, never media/capture authority.
use super::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ReconnectError {
    InvalidProject(&'static str),
    InvalidSelection(&'static str),
    NoWatchingSelection,
    SelectionChanged,
    KindOrIdChanged,
    RevisionExhausted,
}
pub fn watching_locator(
    previous: &ProjectRevision,
    expected: &WatchingSource,
    replacement: WatchingSelection,
) -> Result<ProjectRevision, ReconnectError> {
    previous
        .validate()
        .map_err(ReconnectError::InvalidProject)?;
    let watching = previous
        .watching
        .as_ref()
        .ok_or(ReconnectError::NoWatchingSelection)?;
    WatchingSelection {
        label: watching.label.clone(),
        source: expected.clone(),
    }
    .validate()
    .map_err(ReconnectError::InvalidSelection)?;
    replacement
        .validate()
        .map_err(ReconnectError::InvalidSelection)?;
    if watching.source != *expected {
        return Err(ReconnectError::SelectionChanged);
    }
    let same_identity = match (&watching.source, &replacement.source) {
        (
            WatchingSource::LocalVideo { source_id: a, .. },
            WatchingSource::LocalVideo { source_id: b, .. },
        )
        | (
            WatchingSource::EpisodePage { source_id: a, .. },
            WatchingSource::EpisodePage { source_id: b, .. },
        ) => a == b,
        _ => false,
    };
    if !same_identity {
        return Err(ReconnectError::KindOrIdChanged);
    }
    let mut next = previous.clone();
    next.revision = DecimalU64(
        previous
            .revision
            .0
            .checked_add(1)
            .ok_or(ReconnectError::RevisionExhausted)?,
    );
    next.parent_revision = Some(previous.revision);
    next.watching = Some(replacement);
    next.validate().map_err(ReconnectError::InvalidSelection)?;
    Ok(next)
}
