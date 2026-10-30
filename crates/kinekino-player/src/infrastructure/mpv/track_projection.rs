//! Raw engine metadata is projected into bounded domain observations.
use crate::domain::playback_options::{
    NativeTrack, TrackCatalog, TrackCatalogError, TrackId, TrackKind,
};
use serde_json::Value;

pub(super) fn apply(catalog: &mut TrackCatalog, value: &Value) -> Result<bool, TrackCatalogError> {
    match project(value) {
        Ok(entries) => catalog.observe(entries),
        Err(error) => {
            catalog.invalidate()?;
            Err(error)
        }
    }
}
fn project(value: &Value) -> Result<Vec<NativeTrack>, TrackCatalogError> {
    let entries = value.as_array().ok_or(TrackCatalogError::InvalidMetadata)?;
    if entries.len() > crate::domain::playback_options::MAX_TRACKS {
        return Err(TrackCatalogError::TooManyTracks);
    }
    entries
        .iter()
        .map(|entry| {
            let id = TrackId::new(
                entry["id"]
                    .as_u64()
                    .ok_or(TrackCatalogError::InvalidTrackId)?,
            )?;
            let kind = match entry["type"].as_str() {
                Some("video") => TrackKind::Video,
                Some("audio") => TrackKind::Audio,
                Some("sub") => TrackKind::Subtitle,
                _ => return Err(TrackCatalogError::InvalidMetadata),
            };
            Ok(NativeTrack {
                id,
                kind,
                language: optional_text(entry.get("lang"))?,
                codec: optional_text(entry.get("codec"))?,
                selected: optional_flag(entry.get("selected"))?,
                external: optional_flag(entry.get("external"))?,
            })
        })
        .collect()
}
fn optional_text(value: Option<&Value>) -> Result<Option<String>, TrackCatalogError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if text.is_empty() => Ok(None),
        Some(Value::String(text)) if text.len() <= 128 => Ok(Some(text.clone())),
        _ => Err(TrackCatalogError::InvalidMetadata),
    }
}
fn optional_flag(value: Option<&Value>) -> Result<Option<bool>, TrackCatalogError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        _ => Err(TrackCatalogError::InvalidMetadata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn projection_keeps_missing_flags_unknown_and_discards_raw_locators() {
        let mut catalog = TrackCatalog::default();
        apply(&mut catalog, &json!([
            {"id":1,"type":"video","codec":"h264","selected":true,"external-filename":"https://private.invalid/opaque?token=synthetic"},
            {"id":1,"type":"audio","lang":"jpn","title":"unprojected metadata"}
        ])).unwrap();
        let view = catalog.view().unwrap();
        assert_eq!(view.entries[0].selected, Some(true));
        assert_eq!(view.entries[1].selected, None);
        assert_eq!(view.entries[1].external, None);
        let bytes = serde_json::to_string(&view).unwrap();
        assert!(!bytes.contains("private.invalid"));
        assert!(!bytes.contains("unprojected"));
    }
    #[test]
    fn malformed_events_invalidate_prior_catalog_instead_of_retaining_old_authority() {
        let mut catalog = TrackCatalog::default();
        apply(&mut catalog, &json!([{"id":1,"type":"audio"}])).unwrap();
        assert!(
            apply(
                &mut catalog,
                &json!([{"id":1,"type":"audio","selected":"yes"}])
            )
            .is_err()
        );
        assert!(catalog.view().is_none());
        assert!(
            apply(
                &mut catalog,
                &json!([{"id":1,"type":"audio","codec":"https://private.invalid"}])
            )
            .is_err()
        );
        apply(&mut catalog, &json!([])).unwrap();
        assert!(catalog.view().unwrap().entries.is_empty());
        assert!(apply(&mut catalog, &Value::Null).is_err());
        assert!(catalog.view().is_none());
    }
}
