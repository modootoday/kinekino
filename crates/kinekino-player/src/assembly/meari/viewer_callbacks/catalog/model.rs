use super::contracts::{SelectedEpisode, SelectedSeries};
use crate::application::{
    api::{ApiError, ErrorCode},
    catalog::{CatalogResult, Category, Entry, ListEpisodes, ResultArtifactRef, SearchCatalog},
    episode_resolve::ResolveEpisode,
    jobs::{GetJob, JobRecord, SubmitJob},
};
use meari::application::plugins::{PluginError, PluginTicket};

#[derive(Clone)]
pub(super) enum Intent {
    Category { category: Category, page: u32 },
    Series(SelectedSeries),
    Episode(SelectedEpisode),
}
impl Intent {
    pub(super) fn submission(&self) -> Result<SubmitJob, ApiError> {
        let (operation, parameters, timeout) = match self {
            Self::Category { category, page } => (
                "catalog.search",
                serde_json::to_value(SearchCatalog {
                    category: *category,
                    page: *page,
                }),
                30_000,
            ),
            Self::Series(selected) => {
                let input = ListEpisodes {
                    series_url: selected.row.url.clone(),
                };
                let normalized = input.location()?.as_str().to_owned();
                (
                    "episodes.list",
                    serde_json::to_value(ListEpisodes {
                        series_url: normalized,
                    }),
                    30_000,
                )
            }
            Self::Episode(selected) => {
                let input = ResolveEpisode {
                    episode_url: selected.row.url.clone(),
                };
                let normalized = input.location()?.as_str().to_owned();
                (
                    "episodes.resolve",
                    serde_json::to_value(ResolveEpisode {
                        episode_url: normalized,
                    }),
                    60_000,
                )
            }
        };
        Ok(SubmitJob {
            operation: operation.into(),
            parameters: parameters
                .map_err(|_| mismatch("Typed browser request encoding failed"))?,
            expected_source_sha256: None,
            execution_timeout_ms: crate::domain::api_values::DecimalU64(timeout),
        })
    }
}
pub(super) struct Listing {
    pub artifact: ResultArtifactRef,
    pub result: CatalogResult,
}
impl Listing {
    pub(super) fn key(&self, index: usize) -> String {
        format!("{}:{index}", self.artifact.sha256)
    }
    pub(super) fn select(&self, key: &str) -> Result<Entry, ApiError> {
        self.result
            .entries
            .iter()
            .enumerate()
            .find(|(index, _)| self.key(*index) == key)
            .map(|(_, row)| row.clone())
            .ok_or_else(|| mismatch("Catalog row belongs to a stale or different result"))
    }
}
pub(super) struct Staged {
    pub intent: Intent,
    pub input: SubmitJob,
    pub ticket: Option<PluginTicket>,
}
pub(super) struct Active {
    pub intent: Intent,
    pub job: JobRecord,
}
#[derive(Default)]
pub(super) struct CatalogModel {
    pub series: Option<Listing>,
    pub episodes: Option<Listing>,
    pub selected_series: Option<SelectedSeries>,
    pub staged: Option<Staged>,
    pub active: Option<Active>,
    pub rpc: Option<PluginTicket>,
    pub resolution: Option<super::contracts::ResolutionChoice>,
    pub diagnostic: Option<ApiError>,
    pub cancel_requested: bool,
    pub adapter_error: Option<PluginError>,
    pub previous: Option<Active>,
}
impl CatalogModel {
    pub(super) fn stage(&mut self, intent: Intent) -> Result<SubmitJob, ApiError> {
        if self.active.is_some()
            || self.rpc.is_some()
            || self.staged.as_ref().is_some_and(|s| s.ticket.is_some())
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Original browser submission still requires observation or cancellation",
            ));
        }
        let input = intent.submission()?;
        self.resolution = None;
        self.staged = Some(Staged {
            intent,
            input: input.clone(),
            ticket: None,
        });
        Ok(input)
    }
    pub(super) fn choose_series(&self, key: &str) -> Result<Intent, ApiError> {
        let listing = self
            .series
            .as_ref()
            .ok_or_else(|| mismatch("Original catalog list missing"))?;
        Ok(Intent::Series(SelectedSeries {
            key: key.into(),
            list: listing.artifact.clone(),
            row: listing.select(key)?,
        }))
    }
    pub(super) fn choose_episode(&self, key: &str) -> Result<Intent, ApiError> {
        let listing = self
            .episodes
            .as_ref()
            .ok_or_else(|| mismatch("Original episode list missing"))?;
        let series = self
            .selected_series
            .clone()
            .ok_or_else(|| mismatch("Original selected series missing"))?;
        if self
            .series
            .as_ref()
            .is_none_or(|current| current.artifact.sha256 != series.list.sha256)
        {
            return Err(mismatch(
                "Selected series belongs to another original catalog result",
            ));
        }
        Ok(Intent::Episode(SelectedEpisode {
            key: key.into(),
            series,
            list: listing.artifact.clone(),
            row: listing.select(key)?,
        }))
    }
    pub(super) fn poll_input(&self) -> Result<GetJob, ApiError> {
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original browser job missing"))?;
        Ok(GetJob {
            job_id: active.job.job_id.clone(),
        })
    }
    pub(super) fn release_terminal(&mut self) -> Result<(), ApiError> {
        if self.rpc.is_some()
            || !self
                .active
                .as_ref()
                .is_some_and(|active| active.job.state.terminal())
        {
            return Err(mismatch(
                "Original browser job is not terminal and accepted",
            ));
        }
        self.previous = self.active.take();
        self.diagnostic = None;
        self.adapter_error = None;
        Ok(())
    }
}
pub(super) fn mismatch(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}
