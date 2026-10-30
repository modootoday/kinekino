//! Adjacent episodes follow validated list order, never URL arithmetic.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::{CatalogResults, ListEpisodes},
        episode_queue::EpisodeQueue,
        episode_resolve::EpisodeResolutionResults,
        playback_session::EpisodePlaybackSource,
    },
    domain::{
        episodes::{Episode, EpisodePage},
        identifiers::valid_sha256,
    },
};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeDirection {
    Previous,
    Next,
}

/// A prepared target is not an engine replacement or proof of live playback.
/// No Deserialize: callers cannot substitute an arbitrary destination.
pub struct PreparedAdjacentEpisode {
    current_resolution_artifact_id: String,
    target_episode_url: String,
}

impl PreparedAdjacentEpisode {
    #[cfg(test)]
    pub(super) fn retirement_fixture(target_episode_url: String) -> Self {
        Self {
            current_resolution_artifact_id: "a".repeat(64),
            target_episode_url,
        }
    }
    pub fn current_resolution_artifact_id(&self) -> &str {
        &self.current_resolution_artifact_id
    }
    pub fn target_episode_url(&self) -> &str {
        &self.target_episode_url
    }
    /// The common result reader must verify all referenced raw evidence before returning.
    /// Resolve/identity failure leaves the owned current engine untouched.
    pub fn verify_resolution(
        &self,
        results: &dyn EpisodeResolutionResults,
        target_artifact_id: &str,
    ) -> Result<EpisodePlaybackSource, ApiError> {
        require_sha(target_artifact_id)?;
        let target = results.read(target_artifact_id)?;
        target.validate()?;
        if target.episode_url != self.target_episode_url {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Resolved episode does not match the prepared adjacent target",
            ));
        }
        EpisodePlaybackSource::from_result(target_artifact_id.into(), target)
    }
}

pub fn prepare(
    catalogs: &dyn CatalogResults,
    resolutions: &dyn EpisodeResolutionResults,
    catalog_result_sha256: &str,
    current_resolution_artifact_id: &str,
    direction: EpisodeDirection,
) -> Result<PreparedAdjacentEpisode, ApiError> {
    require_sha(catalog_result_sha256)?;
    require_sha(current_resolution_artifact_id)?;
    let (stored, reference) = catalogs.read(catalog_result_sha256)?;
    stored.validate()?;
    if stored.operation != "episodes.list" || reference.sha256 != catalog_result_sha256 {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Episode list identity mismatch",
        ));
    }
    let request: ListEpisodes = serde_json::from_value(stored.parameters).map_err(|_| {
        ApiError::new(
            ErrorCode::SourceExpired,
            "Invalid stored episode list request",
        )
    })?;
    let location = request.location()?;
    if stored
        .result
        .series
        .as_ref()
        .is_none_or(|series| series.url != location.as_str())
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Stored episode list series identity mismatch",
        ));
    }
    let current = resolutions.read(current_resolution_artifact_id)?;
    current.validate()?;
    // Use the existing shared list validation, duplicate handling and boundary rules.
    let page = EpisodePage {
        page: current.episode_url.clone(),
        series: location.as_str().into(),
        title: stored.result.title,
        entries: stored
            .result
            .entries
            .into_iter()
            .map(|entry| Episode {
                title: entry.title,
                url: entry.url,
            })
            .collect(),
    };
    let target_episode_url = select_adjacent(page, &current.episode_url, direction)?;
    Ok(PreparedAdjacentEpisode {
        current_resolution_artifact_id: current_resolution_artifact_id.into(),
        target_episode_url,
    })
}

fn select_adjacent(
    page: EpisodePage,
    current_url: &str,
    direction: EpisodeDirection,
) -> Result<String, ApiError> {
    let mut queue = EpisodeQueue::default();
    if !queue.observe(page, current_url) || queue.current.is_none() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Current episode is not in the verified list",
        ));
    }
    let delta = match direction {
        EpisodeDirection::Previous => -1,
        EpisodeDirection::Next => 1,
    };
    queue.adjacent(delta).ok_or_else(|| {
        ApiError::new(
            ErrorCode::NotFound,
            "No adjacent episode in the verified list",
        )
    })
}

fn require_sha(value: &str) -> Result<(), ApiError> {
    if valid_sha256(value) {
        Ok(())
    } else {
        Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid episode artifact identity",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn page(current: &str) -> EpisodePage {
        EpisodePage {
            page: format!("https://catalog.example.test/e/{current}"),
            series: "https://catalog.example.test/c/series".into(),
            title: "Series".into(),
            entries: ["z9", "a2", "z9", "special"]
                .into_iter()
                .map(|slug| Episode {
                    title: "Identical display title".into(),
                    url: format!("https://catalog.example.test/e/{slug}"),
                })
                .collect(),
        }
    }
    #[test]
    fn respects_observed_order_and_first_duplicate_identity_without_slug_arithmetic() {
        assert_eq!(
            select_adjacent(
                page("z9"),
                "https://catalog.example.test/e/z9",
                EpisodeDirection::Next
            )
            .unwrap(),
            "https://catalog.example.test/e/a2"
        );
        assert_eq!(
            select_adjacent(
                page("a2"),
                "https://catalog.example.test/e/a2",
                EpisodeDirection::Next
            )
            .unwrap(),
            "https://catalog.example.test/e/special"
        );
        assert_eq!(
            select_adjacent(
                page("special"),
                "https://catalog.example.test/e/special",
                EpisodeDirection::Previous
            )
            .unwrap(),
            "https://catalog.example.test/e/a2"
        );
    }
    #[test]
    fn rejects_missing_current_stale_page_and_invalid_destination() {
        assert_eq!(
            select_adjacent(
                page("missing"),
                "https://catalog.example.test/e/missing",
                EpisodeDirection::Next
            )
            .unwrap_err()
            .code,
            ErrorCode::SourceExpired
        );
        assert_eq!(
            select_adjacent(
                page("z9"),
                "https://catalog.example.test/e/a2",
                EpisodeDirection::Next
            )
            .unwrap_err()
            .code,
            ErrorCode::SourceExpired
        );
        let mut invalid = page("z9");
        invalid.entries[1].url = "https://evil.test/e/next".into();
        assert_eq!(
            select_adjacent(
                invalid,
                "https://catalog.example.test/e/z9",
                EpisodeDirection::Next
            )
            .unwrap_err()
            .code,
            ErrorCode::SourceExpired
        );
    }
    #[test]
    fn boundaries_do_not_wrap_to_an_unrelated_episode() {
        assert_eq!(
            select_adjacent(
                page("z9"),
                "https://catalog.example.test/e/z9",
                EpisodeDirection::Previous
            )
            .unwrap_err()
            .code,
            ErrorCode::NotFound
        );
        assert_eq!(
            select_adjacent(
                page("special"),
                "https://catalog.example.test/e/special",
                EpisodeDirection::Next
            )
            .unwrap_err()
            .code,
            ErrorCode::NotFound
        );
    }
}
