use crate::domain::api_values::DecimalU64;
use serde::{Deserialize, Deserializer, Serialize};
pub const MAX_TRACKS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackKind {
    Video,
    Audio,
    Subtitle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct TrackId(DecimalU64);
impl TrackId {
    pub fn new(value: u64) -> Result<Self, TrackCatalogError> {
        if value == 0 || value > u64::from(u32::MAX) {
            return Err(TrackCatalogError::InvalidTrackId);
        }
        Ok(Self(DecimalU64(value)))
    }
    pub fn get(self) -> u64 {
        self.0.0
    }
}
impl<'de> Deserialize<'de> for TrackId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(DecimalU64::deserialize(deserializer)?.0)
            .map_err(|_| serde::de::Error::custom("Invalid native track ID"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TrackSelection {
    Disabled,
    Id { track_id: TrackId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum ObservedTrackChoice {
    Disabled,
    Selected {
        track_id: TrackId,
    },
    /// The engine reports an option such as auto; effective selection is unresolved.
    Unresolved,
}

/// Public bounded projection. Raw external filenames, URLs and arbitrary metadata are excluded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTrack {
    pub id: TrackId,
    pub kind: TrackKind,
    pub language: Option<String>,
    pub codec: Option<String>,
    pub selected: Option<bool>,
    pub external: Option<bool>,
}
impl NativeTrack {
    fn valid(&self) -> bool {
        self.language.as_ref().is_none_or(|value| {
            !value.is_empty()
                && value.len() <= 64
                && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        }) && self.codec.as_ref().is_none_or(|value| {
            !value.is_empty()
                && value.len() <= 128
                && value
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '+' | '.'))
        })
    }
}

/// Inspection data only. Deserializing a view does not construct a live selection authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTracks {
    pub revision: DecimalU64,
    pub entries: Vec<NativeTrack>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackCatalogError {
    Unobserved,
    StaleRevision,
    UnknownTrack,
    InvalidTrackId,
    InvalidMetadata,
    TooManyTracks,
    DuplicateTrack,
    RevisionExhausted,
}
impl TrackCatalogError {
    pub const ALL: [Self; 8] = [
        Self::Unobserved,
        Self::StaleRevision,
        Self::UnknownTrack,
        Self::InvalidTrackId,
        Self::InvalidMetadata,
        Self::TooManyTracks,
        Self::DuplicateTrack,
        Self::RevisionExhausted,
    ];
}

#[derive(Default)]
pub struct TrackCatalog {
    revision: u64,
    entries: Option<Vec<NativeTrack>>,
}
impl TrackCatalog {
    pub fn observe(&mut self, entries: Vec<NativeTrack>) -> Result<bool, TrackCatalogError> {
        if let Err(error) = validate(&entries) {
            self.replace(None)?;
            return Err(error);
        }
        self.replace(Some(entries))
    }
    pub fn invalidate(&mut self) -> Result<bool, TrackCatalogError> {
        self.replace(None)
    }
    pub fn view(&self) -> Option<ObservedTracks> {
        Some(ObservedTracks {
            revision: DecimalU64(self.revision),
            entries: self.entries.clone()?,
        })
    }
    /// Matches the latest observed catalog. Queue acceptance is not an engine transaction.
    pub fn authorize(
        &self,
        expected_revision: DecimalU64,
        kind: TrackKind,
        selection: TrackSelection,
    ) -> Result<(), TrackCatalogError> {
        let entries = self.entries.as_ref().ok_or(TrackCatalogError::Unobserved)?;
        if expected_revision.0 == 0 || expected_revision.0 != self.revision {
            return Err(TrackCatalogError::StaleRevision);
        }
        if let TrackSelection::Id { track_id } = selection
            && !entries
                .iter()
                .any(|track| track.kind == kind && track.id == track_id)
        {
            return Err(TrackCatalogError::UnknownTrack);
        }
        Ok(())
    }
    fn replace(&mut self, entries: Option<Vec<NativeTrack>>) -> Result<bool, TrackCatalogError> {
        if self.entries == entries {
            return Ok(false);
        }
        let Some(next) = self.revision.checked_add(1) else {
            self.entries = None;
            return Err(TrackCatalogError::RevisionExhausted);
        };
        self.revision = next;
        self.entries = entries;
        Ok(true)
    }
}
fn validate(entries: &[NativeTrack]) -> Result<(), TrackCatalogError> {
    if entries.len() > MAX_TRACKS {
        return Err(TrackCatalogError::TooManyTracks);
    }
    let mut seen = std::collections::HashSet::new();
    for track in entries {
        if !track.valid() {
            return Err(TrackCatalogError::InvalidMetadata);
        }
        if !seen.insert((track.kind, track.id.get())) {
            return Err(TrackCatalogError::DuplicateTrack);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn track(kind: TrackKind, id: u64) -> NativeTrack {
        NativeTrack {
            id: TrackId::new(id).unwrap(),
            kind,
            language: Some("jpn".into()),
            codec: None,
            selected: None,
            external: None,
        }
    }
    #[test]
    fn kind_and_revision_are_required_and_missing_observation_is_not_empty() {
        let mut catalog = TrackCatalog::default();
        let selection = TrackSelection::Id {
            track_id: TrackId::new(1).unwrap(),
        };
        assert_eq!(
            catalog.authorize(DecimalU64(0), TrackKind::Audio, selection),
            Err(TrackCatalogError::Unobserved)
        );
        assert!(
            catalog
                .observe(vec![track(TrackKind::Video, 1), track(TrackKind::Audio, 1)])
                .unwrap()
        );
        assert!(
            !catalog
                .observe(vec![track(TrackKind::Video, 1), track(TrackKind::Audio, 1)])
                .unwrap()
        );
        let revision = catalog.view().unwrap().revision;
        assert!(
            catalog
                .authorize(revision, TrackKind::Audio, selection)
                .is_ok()
        );
        assert_eq!(
            catalog.authorize(revision, TrackKind::Subtitle, selection),
            Err(TrackCatalogError::UnknownTrack)
        );
        catalog.observe(vec![]).unwrap();
        assert_eq!(
            catalog.authorize(revision, TrackKind::Audio, selection),
            Err(TrackCatalogError::StaleRevision)
        );
        let empty = catalog.view().unwrap();
        assert!(empty.entries.is_empty());
        assert!(
            catalog
                .authorize(empty.revision, TrackKind::Audio, TrackSelection::Disabled)
                .is_ok()
        );
        catalog.invalidate().unwrap();
        assert!(catalog.view().is_none());
    }
    #[test]
    fn malformed_replacement_invalidates_old_authority_and_enforces_limits() {
        let mut catalog = TrackCatalog::default();
        let audio = track(TrackKind::Audio, 1);
        catalog.observe(vec![audio.clone()]).unwrap();
        assert_eq!(
            catalog.observe(vec![audio.clone(), audio]),
            Err(TrackCatalogError::DuplicateTrack)
        );
        assert!(catalog.view().is_none());
        let mut unsafe_metadata = track(TrackKind::Audio, 2);
        unsafe_metadata.codec = Some("https://private.invalid/opaque".into());
        assert_eq!(
            catalog.observe(vec![unsafe_metadata]),
            Err(TrackCatalogError::InvalidMetadata)
        );
        assert_eq!(
            catalog.observe((1..=65).map(|id| track(TrackKind::Audio, id)).collect()),
            Err(TrackCatalogError::TooManyTracks)
        );
        assert!(TrackId::new(0).is_err());
        assert!(TrackId::new(u64::from(u32::MAX) + 1).is_err());
    }
    #[test]
    fn exhausted_revision_never_reauthorizes_old_tokens() {
        let mut catalog = TrackCatalog {
            revision: u64::MAX,
            entries: Some(vec![track(TrackKind::Audio, 1)]),
        };
        assert_eq!(
            catalog.observe(vec![track(TrackKind::Audio, 2)]),
            Err(TrackCatalogError::RevisionExhausted)
        );
        assert!(catalog.view().is_none());
    }
}
