use super::worker::{self, Action, Completion, Key, Outcome};
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle, ErrorCode},
        model_review::{ReviewPackageView, ReviewTextAsset, StoredDecision},
    },
    domain::model_review::{RecordReviewDecision, ReviewItem, ReviewVerdict, valid_sha256},
};
use eframe::egui;
use std::{
    collections::BTreeMap,
    sync::mpsc::{Receiver, TryRecvError},
};

struct Pending {
    key: Key,
    receiver: Receiver<Completion>,
}

pub(super) struct ReviewState {
    client: ApplicationClientHandle,
    generation: u64,
    pending: Option<Pending>,
    pub available: bool,
    pub package_input: String,
    pub package: Option<Box<ReviewPackageView>>,
    pub selected: Option<String>,
    pub assets: BTreeMap<(String, String), Result<Box<ReviewTextAsset>, ApiError>>,
    pub history: Option<Result<Vec<StoredDecision>, ApiError>>,
    pub saved: Option<Result<Box<StoredDecision>, ApiError>>,
    pub error: Option<ApiError>,
    pub verdict: Option<ReviewVerdict>,
    pub note: String,
    pub compare_task: bool,
}

impl ReviewState {
    pub fn new(client: ApplicationClientHandle, available: bool) -> Self {
        Self {
            client,
            generation: 0,
            pending: None,
            available,
            package_input: String::new(),
            package: None,
            selected: None,
            assets: BTreeMap::new(),
            history: None,
            saved: None,
            error: None,
            verdict: None,
            note: String::new(),
            compare_task: true,
        }
    }

    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn item(&self) -> Option<&ReviewItem> {
        self.package
            .as_ref()?
            .package
            .items
            .iter()
            .find(|item| Some(&item.item_id) == self.selected.as_ref())
    }

    fn dispatch(&mut self, action: Action, package: String, ctx: &egui::Context) {
        self.generation = self.generation.wrapping_add(1);
        let key = Key {
            generation: self.generation,
            package_sha256: package,
            item_id: self.selected.clone(),
        };
        self.pending = Some(Pending {
            receiver: worker::start(self.client.clone(), key.clone(), action, ctx.clone()),
            key,
        });
        self.error = None;
    }

    pub fn inspect(&mut self, ctx: &egui::Context) {
        if self.busy() || !self.available || !valid_sha256(&self.package_input) {
            return;
        }
        self.package = None;
        self.selected = None;
        self.assets.clear();
        self.clear_draft();
        self.dispatch(Action::Inspect, self.package_input.clone(), ctx);
    }

    pub fn select(&mut self, item_id: &str) {
        if self.busy()
            || self.selected.as_deref() == Some(item_id)
            || self.package.as_ref().is_none_or(|view| {
                !view
                    .package
                    .items
                    .iter()
                    .any(|item| item.item_id == item_id)
            })
        {
            return;
        }
        self.selected = Some(item_id.into());
        self.clear_draft();
    }

    fn clear_draft(&mut self) {
        self.history = None;
        self.saved = None;
        self.error = None;
        self.verdict = None;
        self.note.clear();
    }

    pub fn read_asset(&mut self, asset_sha256: &str, ctx: &egui::Context) {
        if self.busy() {
            return;
        }
        let Some(view) = &self.package else {
            return;
        };
        self.dispatch(
            Action::ReadAsset {
                asset_sha256: asset_sha256.into(),
            },
            view.package_sha256.clone(),
            ctx,
        );
    }

    pub fn load_history(&mut self, ctx: &egui::Context) {
        if self.busy() || self.item().is_none() {
            return;
        }
        let Some(view) = &self.package else {
            return;
        };
        self.dispatch(Action::History, view.package_sha256.clone(), ctx);
    }

    fn draft(&self, decision_id: String) -> Option<RecordReviewDecision> {
        Some(RecordReviewDecision {
            package_sha256: self.package.as_ref()?.package_sha256.clone(),
            item_id: self.item()?.item_id.clone(),
            decision_id,
            verdict: self.verdict?,
            note: self.note.clone(),
            issue_ranges: Vec::new(),
        })
    }

    pub fn can_record(&self) -> bool {
        self.available
            && !self.busy()
            && self
                .draft("ui-draft".into())
                .is_some_and(|input| input.validate().is_ok())
    }

    pub fn record(&mut self, ctx: &egui::Context) {
        if !self.can_record() {
            return;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let Some(input) = self.draft(format!("ui-{stamp}-{}", self.generation)) else {
            return;
        };
        let package = input.package_sha256.clone();
        self.saved = None;
        self.dispatch(Action::Record(Box::new(input)), package, ctx);
    }

    pub fn poll(&mut self) {
        let Some(pending) = &self.pending else {
            return;
        };
        let expected = pending.key.clone();
        let result = pending.receiver.try_recv();
        let completion = match result {
            Ok(completion) => completion,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                self.error = Some(ApiError::new(
                    ErrorCode::Internal,
                    "검토 요청의 응답을 확인할 수 없습니다. 저장 요청은 자동 재시도하지 않습니다.",
                ));
                return;
            }
        };
        self.pending = None;
        if completion.key != expected || expected.generation != self.generation {
            self.error = Some(identity_error());
            return;
        }
        match completion.outcome {
            Outcome::Package(Ok(view))
                if view.package_sha256 == expected.package_sha256
                    && !view.automatic_application_enabled =>
            {
                self.selected = view.package.items.first().map(|item| item.item_id.clone());
                self.package = Some(view);
            }
            Outcome::Package(Err(error)) => self.error = Some(error),
            Outcome::Package(Ok(_)) => self.error = Some(identity_error()),
            Outcome::Asset {
                asset_sha256,
                result,
            } => {
                if result.as_ref().is_ok_and(|asset| {
                    asset.package_sha256 != expected.package_sha256
                        || asset.asset_sha256 != asset_sha256
                }) {
                    self.error = Some(identity_error());
                } else {
                    self.assets
                        .insert((expected.package_sha256, asset_sha256), result);
                }
            }
            Outcome::History(result) => {
                if result.as_ref().is_ok_and(|history| {
                    history.iter().any(|stored| {
                        stored.decision.package_sha256 != expected.package_sha256
                            || Some(&stored.decision.item_id) != expected.item_id.as_ref()
                            || stored.decision.scope != "sample_only"
                            || stored.decision.automatic_application_enabled
                    })
                }) {
                    self.error = Some(identity_error());
                } else {
                    self.history = Some(result);
                }
            }
            Outcome::Recorded(result) => {
                if result.as_ref().is_ok_and(|stored| {
                    stored.decision.package_sha256 != expected.package_sha256
                        || Some(&stored.decision.item_id) != expected.item_id.as_ref()
                        || stored.decision.scope != "sample_only"
                        || stored.decision.automatic_application_enabled
                }) {
                    self.error = Some(identity_error());
                    return;
                }
                if result.is_ok() {
                    self.history = None;
                    self.verdict = None;
                    self.note.clear();
                }
                self.saved = Some(result);
            }
        }
    }
}

fn identity_error() -> ApiError {
    ApiError::new(
        ErrorCode::Internal,
        "검토 응답의 패키지·표본 식별자가 일치하지 않습니다.",
    )
}

#[cfg(test)]
mod tests;
