//! Package/item display state. The existing Pending host owns execution, ACK and cleanup.
use super::pending_host::{PendingOutput, PendingRequest};
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::*,
    },
    domain::model_review::{RecordReviewDecision, ReviewItem, ReviewVerdict},
};
use std::{collections::BTreeMap, sync::Arc};

pub enum RawReviewAction {
    InspectRetained,
    HistoryRetained,
    ReleaseRetained,
    PackageInput(String),
    ItemInput(String),
    OpenPackage,
    OpenItem,
    ReadSource,
    ReadCandidate,
    Inspect(String),
    Select(String),
    Read(String),
    Verdict(ReviewVerdict),
    Note(String),
    Record,
    History,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RawReviewButton {
    InspectRetained,
    HistoryRetained,
    ReleaseRetained,
    OpenPackage,
    OpenItem,
    ReadSource,
    ReadCandidate,
    Record,
    History,
    Verdict(ReviewVerdict),
}
impl RawReviewButton {
    pub fn action(self) -> RawReviewAction {
        match self {
            Self::InspectRetained => RawReviewAction::InspectRetained,
            Self::HistoryRetained => RawReviewAction::HistoryRetained,
            Self::ReleaseRetained => RawReviewAction::ReleaseRetained,
            Self::OpenPackage => RawReviewAction::OpenPackage,
            Self::OpenItem => RawReviewAction::OpenItem,
            Self::ReadSource => RawReviewAction::ReadSource,
            Self::ReadCandidate => RawReviewAction::ReadCandidate,
            Self::Record => RawReviewAction::Record,
            Self::History => RawReviewAction::History,
            Self::Verdict(value) => RawReviewAction::Verdict(value),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RawReviewField {
    Package,
    Item,
    Note,
}
impl RawReviewField {
    pub fn action(self, value: String) -> RawReviewAction {
        match self {
            Self::Package => RawReviewAction::PackageInput(value),
            Self::Item => RawReviewAction::ItemInput(value),
            Self::Note => RawReviewAction::Note(value),
        }
    }
}
enum Operation {
    Inspect,
    Read(String),
    Record(String),
    History,
}
pub struct RawReviewToken {
    owner: Arc<()>,
    generation: u64,
    package_sha256: String,
    item_id: Option<String>,
    operation: Operation,
}
impl RawReviewToken {
    pub(super) fn original_selection(&self) -> (&str, Option<&str>) {
        (&self.package_sha256, self.item_id.as_deref())
    }
    pub(super) fn package_only_read(&self) -> bool {
        self.item_id.is_none() && matches!(&self.operation, Operation::Inspect | Operation::Read(_))
    }
    pub fn identity_label(&self) -> String {
        let operation = match &self.operation {
            Operation::Inspect => "자료 조회".into(),
            Operation::Read(sha) => format!("원 자산 읽기 {sha}"),
            Operation::Record(id) => format!("판정 기록 {id}"),
            Operation::History => "판정 이력 조회".into(),
        };
        format!(
            "자료 {}\n후보 {}\n선택 버전 {}\n{}",
            self.package_sha256,
            self.item_id.as_deref().unwrap_or("선택 전"),
            self.generation,
            operation
        )
    }
}
#[must_use = "Retain the original token and typed terminal until explicit recovery or owner cleanup"]
pub struct RawReviewRejected {
    pub token: RawReviewToken,
    pub outcome: Result<PendingOutput, ApiError>,
}
pub struct RawReviewState {
    owner: Arc<()>,
    generation: u64,
    package_sha256: String,
    pub package: Option<Box<ReviewPackageView>>,
    pub selected_item: Option<String>,
    pub package_input: String,
    pub item_input: String,
    pub assets: BTreeMap<String, ReviewTextAsset>,
    pub history: Option<Vec<StoredDecision>>,
    pub saved: Option<StoredDecision>,
    pub attempted_record: Option<RecordReviewDecision>,
    pub verdict: Option<ReviewVerdict>,
    pub note: String,
    pub error: Option<ApiError>,
    pub record_result_unknown: bool,
    next_id: super::pending_controller::DecisionIdSource,
}
fn input(message: &str) -> ApiError {
    ApiError::new(ErrorCode::NeedsInput, message)
}
impl RawReviewState {
    pub(super) fn clear_after_recovery(&mut self) {
        self.clear_selection();
        self.package = None;
        self.selected_item = None;
        self.package_sha256.clear();
        self.package_input.clear();
        self.item_input.clear();
        self.attempted_record = None;
    }
    pub fn new(next_id: super::pending_controller::DecisionIdSource) -> Self {
        Self {
            owner: Arc::new(()),
            generation: 0,
            package_sha256: String::new(),
            package: None,
            selected_item: None,
            package_input: String::new(),
            item_input: String::new(),
            assets: BTreeMap::new(),
            history: None,
            saved: None,
            attempted_record: None,
            verdict: None,
            note: String::new(),
            error: None,
            record_result_unknown: false,
            next_id,
        }
    }
    pub fn invalidate(&mut self) {
        self.owner = Arc::new(());
    }
    pub fn item(&self) -> Option<&ReviewItem> {
        self.package
            .as_ref()?
            .package
            .items
            .iter()
            .find(|item| Some(&item.item_id) == self.selected_item.as_ref())
    }
    pub fn selection_matches_input(&self) -> bool {
        self.package_input == self.package_sha256
            && self.selected_item.as_ref() == Some(&self.item_input)
    }
    fn advance(&mut self) -> Result<(), ApiError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| input("Raw review selection generation exhausted"))?;
        Ok(())
    }
    fn clear_selection(&mut self) {
        self.assets.clear();
        self.history = None;
        self.saved = None;
        self.verdict = None;
        self.note.clear();
        self.error = None;
        self.record_result_unknown = false;
    }
    pub fn prepare(
        &mut self,
        action: RawReviewAction,
    ) -> Result<Option<(RawReviewToken, PendingRequest)>, ApiError> {
        if matches!(
            &action,
            RawReviewAction::ReadSource
                | RawReviewAction::ReadCandidate
                | RawReviewAction::Read(_)
                | RawReviewAction::Verdict(_)
                | RawReviewAction::Record
                | RawReviewAction::History
        ) && !self.selection_matches_input()
        {
            return Err(input(
                "Open the edited package and item selection before reviewing it",
            ));
        }
        let (operation, request) = match action {
            RawReviewAction::InspectRetained
            | RawReviewAction::HistoryRetained
            | RawReviewAction::ReleaseRetained => {
                return Err(input(
                    "Retained recovery requires the original pending controller",
                ));
            }
            RawReviewAction::PackageInput(value) => {
                self.package_input = value;
                return Ok(None);
            }
            RawReviewAction::ItemInput(value) => {
                self.item_input = value;
                return Ok(None);
            }
            RawReviewAction::OpenPackage => {
                return self.prepare(RawReviewAction::Inspect(self.package_input.clone()));
            }
            RawReviewAction::OpenItem => {
                return self.prepare(RawReviewAction::Select(self.item_input.clone()));
            }
            RawReviewAction::ReadSource => {
                let sha = self
                    .item()
                    .ok_or_else(|| input("Choose a raw model result"))?
                    .source
                    .sha256
                    .clone();
                return self.prepare(RawReviewAction::Read(sha));
            }
            RawReviewAction::ReadCandidate => {
                let item = self
                    .item()
                    .ok_or_else(|| input("Choose a raw model result"))?;
                let [asset] = item.candidate.assets.as_slice() else {
                    return Err(input(
                        "This entry requires one explicit candidate text asset",
                    ));
                };
                return self.prepare(RawReviewAction::Read(asset.sha256.clone()));
            }
            RawReviewAction::Inspect(package_sha256) => {
                self.advance()?;
                self.clear_selection();
                self.package = None;
                self.selected_item = None;
                self.package_sha256 = package_sha256.clone();
                self.package_input = package_sha256.clone();
                self.item_input.clear();
                (
                    Operation::Inspect,
                    PendingRequest::InspectRawReview(InspectReviewPackage { package_sha256 }),
                )
            }
            RawReviewAction::Select(item_id) => {
                if self
                    .package
                    .as_ref()
                    .is_none_or(|p| !p.package.items.iter().any(|item| item.item_id == item_id))
                {
                    return Err(input("Choose an item from the inspected package"));
                }
                self.advance()?;
                self.clear_selection();
                self.item_input = item_id.clone();
                self.selected_item = Some(item_id);
                return Ok(None);
            }
            RawReviewAction::Verdict(verdict) => {
                self.item()
                    .ok_or_else(|| input("Choose a raw model result"))?;
                self.verdict = Some(verdict);
                return Ok(None);
            }
            RawReviewAction::Note(note) => {
                self.item()
                    .ok_or_else(|| input("Choose a raw model result"))?;
                self.note = note;
                return Ok(None);
            }
            RawReviewAction::Read(asset_sha256) => {
                let item = self
                    .item()
                    .ok_or_else(|| input("Choose a raw model result"))?;
                if item.source.sha256 != asset_sha256
                    && !item
                        .candidate
                        .assets
                        .iter()
                        .any(|asset| asset.sha256 == asset_sha256)
                {
                    return Err(input("Choose source or candidate evidence from this item"));
                }
                (
                    Operation::Read(asset_sha256.clone()),
                    PendingRequest::ReadRawReviewAsset(ReadReviewTextAsset {
                        package_sha256: self.package_sha256.clone(),
                        asset_sha256,
                    }),
                )
            }
            RawReviewAction::History => {
                let item_id = self
                    .item()
                    .ok_or_else(|| input("Choose a raw model result"))?
                    .item_id
                    .clone();
                (
                    Operation::History,
                    PendingRequest::RawReviewDecisions(ListReviewDecisions {
                        package_sha256: self.package_sha256.clone(),
                        item_id,
                    }),
                )
            }
            RawReviewAction::Record => {
                if self.record_result_unknown {
                    return Err(input(
                        "Read the original decision history before another explicit record",
                    ));
                }
                let item_id = self
                    .item()
                    .ok_or_else(|| input("Choose a raw model result"))?
                    .item_id
                    .clone();
                let verdict = self
                    .verdict
                    .ok_or_else(|| input("Choose an explicit verdict"))?;
                let decision_id = (self.next_id)()?;
                let request = RecordReviewDecision {
                    package_sha256: self.package_sha256.clone(),
                    item_id,
                    decision_id,
                    verdict,
                    note: self.note.clone(),
                    issue_ranges: Vec::new(),
                };
                self.attempted_record = Some(request.clone());
                (
                    Operation::Record(request.decision_id.clone()),
                    PendingRequest::RecordRawReview(request),
                )
            }
        };
        self.error = None;
        Ok(Some((
            RawReviewToken {
                owner: self.owner.clone(),
                generation: self.generation,
                package_sha256: self.package_sha256.clone(),
                item_id: self.selected_item.clone(),
                operation,
            },
            request,
        )))
    }
    /// A rejected display handoff returns the exact token and typed terminal without serialization.
    pub fn complete(
        &mut self,
        token: RawReviewToken,
        outcome: Result<PendingOutput, ApiError>,
    ) -> Result<(), Box<RawReviewRejected>> {
        if !Arc::ptr_eq(&self.owner, &token.owner)
            || self.generation != token.generation
            || self.package_sha256 != token.package_sha256
            || self.selected_item != token.item_id
        {
            return Err(Box::new(RawReviewRejected { token, outcome }));
        }
        if matches!(&token.operation, Operation::Record(_)) {
            self.record_result_unknown = true;
        }
        let matches = match (&token.operation, &outcome) {
            (_, Err(_)) => true,
            (Operation::Inspect, Ok(PendingOutput::RawReviewInspected(view))) => {
                view.package_sha256 == token.package_sha256
            }
            (Operation::Read(sha), Ok(PendingOutput::RawReviewAsset(asset))) => {
                asset.package_sha256 == token.package_sha256 && &asset.asset_sha256 == sha
            }
            (Operation::Record(id), Ok(PendingOutput::RawReviewRecorded(saved))) => {
                saved.decision.package_sha256 == token.package_sha256
                    && Some(&saved.decision.item_id) == token.item_id.as_ref()
                    && &saved.decision.decision_id == id
            }
            (Operation::History, Ok(PendingOutput::RawReviewHistory(history))) => {
                history.iter().all(|saved| {
                    saved.decision.package_sha256 == token.package_sha256
                        && Some(&saved.decision.item_id) == token.item_id.as_ref()
                })
            }
            _ => false,
        };
        if !matches {
            return Err(Box::new(RawReviewRejected { token, outcome }));
        }
        match outcome {
            Err(error) => self.error = Some(error),
            Ok(PendingOutput::RawReviewInspected(view)) => self.package = Some(view),
            Ok(PendingOutput::RawReviewAsset(asset)) => {
                self.assets.insert(asset.asset_sha256.clone(), asset);
            }
            Ok(PendingOutput::RawReviewRecorded(saved)) => {
                self.record_result_unknown = false;
                self.saved = Some(saved);
            }
            Ok(PendingOutput::RawReviewHistory(history)) => {
                self.record_result_unknown = false;
                self.history = Some(history);
            }
            Ok(other) => {
                return Err(Box::new(RawReviewRejected {
                    token,
                    outcome: Ok(other),
                }));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "raw_review_state/tests.rs"]
mod tests;

#[cfg(test)]
mod recovery_identity_tests {
    use super::*;

    #[test]
    fn package_only_release_identity_rejects_mutations_and_selected_items() {
        let mut token = RawReviewToken {
            owner: Arc::new(()),
            generation: 1,
            package_sha256: "a".repeat(64),
            item_id: None,
            operation: Operation::Record("original-record".into()),
        };
        assert!(!token.package_only_read());
        token.operation = Operation::History;
        assert!(!token.package_only_read());
        token.operation = Operation::Inspect;
        assert!(token.package_only_read());
        token.item_id = Some("original-item".into());
        assert!(!token.package_only_read());
    }
}
