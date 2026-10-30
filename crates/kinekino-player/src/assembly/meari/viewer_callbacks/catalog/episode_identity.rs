//! Display identity only. Selection and playback/transition authority remain in the SDK.
use super::CatalogConsumer;
use crate::application::playback_session::{EpisodePlaybackObservation, OpenedEpisodePlayback};
use meari::domain::Value;
use std::collections::BTreeMap;

impl CatalogConsumer {
    pub(crate) fn has_blocking_notice(&self) -> bool {
        self.model.diagnostic.is_some()
            || self.model.adapter_error.is_some()
            || self
                .model
                .active
                .as_ref()
                .is_some_and(|active| active.job.failure.is_some())
            || self.transition.diagnostic.is_some()
            || self.transition.commit_unconfirmed
            || self.open_unconfirmed
    }
    fn expected_episode_row(&self) -> Option<(String, &OpenedEpisodePlayback)> {
        let listing = self.model.episodes.as_ref()?;
        if let Some((replacement, original_catalog)) = &self.transition.replacement {
            if &listing.artifact.sha256 != original_catalog {
                return None;
            }
            let target = self.transition.retained_resolution_url()?;
            let index = listing
                .result
                .entries
                .iter()
                .position(|row| row.url == target)?;
            return Some((listing.key(index), &replacement.current));
        }
        let choice = self.opened.as_ref()?;
        if listing.artifact.sha256 != choice.selected.list.sha256 {
            return None;
        }
        let row = listing.select(&choice.selected.key).ok()?;
        if row.url != choice.selected.row.url || row.id != choice.selected.row.id {
            return None;
        }
        Some((choice.selected.key.clone(), &choice.opened))
    }
    pub(crate) fn episode_identity_props(
        &self,
        actual: Option<&EpisodePlaybackObservation>,
    ) -> BTreeMap<String, Value> {
        let expected = self.expected_episode_row();
        let current = expected.as_ref().is_some_and(|(_, expected)| {
            actual.is_some_and(|actual| {
                actual.playback.connected
                    && actual.engine_instance_id == expected.engine_instance_id
                    && actual.resolution_artifact_id == expected.resolution_artifact_id
            })
        });
        let current_key = expected
            .as_ref()
            .filter(|_| current)
            .map(|(key, _)| key.clone())
            .unwrap_or_default();
        let mut connecting_key = expected
            .filter(|_| !current)
            .map(|(key, _)| key)
            .unwrap_or_default();
        if !current
            && connecting_key.is_empty()
            && self.open_unconfirmed
            && let Some(choice) = &self.model.resolution
            && self.model.episodes.as_ref().is_some_and(|listing| {
                listing.artifact.sha256 == choice.selected.list.sha256
                    && listing.select(&choice.selected.key).is_ok()
            })
        {
            connecting_key = choice.selected.key.clone();
        }
        let label = if connecting_key.is_empty() {
            ""
        } else if self.open_unconfirmed {
            "시작 결과 미확인"
        } else {
            "엔진 연결 확인 중"
        };
        BTreeMap::from([
            ("catalogCurrentEpisode".into(), Value::String(current_key)),
            (
                "catalogConnectingEpisode".into(),
                Value::String(connecting_key),
            ),
            (
                "catalogEpisodeConnectionLabel".into(),
                Value::String(label.into()),
            ),
        ])
    }
}
