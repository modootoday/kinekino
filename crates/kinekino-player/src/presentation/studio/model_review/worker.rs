use crate::application::api::{AssetClient, ReviewClient};
use crate::application::{
    api::{ApiError, ApplicationClientHandle, Request},
    model_review::{
        InspectReviewPackage, ListReviewDecisions, ReadReviewTextAsset, ReviewPackageView,
        ReviewTextAsset, StoredDecision,
    },
};
use crate::domain::model_review::RecordReviewDecision;
use eframe::egui;
use std::sync::mpsc::{self, Receiver};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Key {
    pub generation: u64,
    pub package_sha256: String,
    pub item_id: Option<String>,
}

pub(super) enum Action {
    Inspect,
    ReadAsset { asset_sha256: String },
    History,
    Record(Box<RecordReviewDecision>),
}

pub(super) enum Outcome {
    Package(Result<Box<ReviewPackageView>, ApiError>),
    Asset {
        asset_sha256: String,
        result: Result<Box<ReviewTextAsset>, ApiError>,
    },
    History(Result<Vec<StoredDecision>, ApiError>),
    Recorded(Result<Box<StoredDecision>, ApiError>),
}

pub(super) struct Completion {
    pub key: Key,
    pub outcome: Outcome,
}

pub(super) fn start(
    client: ApplicationClientHandle,
    key: Key,
    action: Action,
    repaint: egui::Context,
) -> Receiver<Completion> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let outcome = match action {
            Action::Inspect => Outcome::Package(
                client
                    .inspect_review(
                        Request::new("reviews.inspect").context,
                        InspectReviewPackage {
                            package_sha256: key.package_sha256.clone(),
                        },
                    )
                    .map(Box::new),
            ),
            Action::ReadAsset { asset_sha256 } => Outcome::Asset {
                result: client
                    .read_review_text_asset(
                        Request::new("reviews.read_text_asset").context,
                        ReadReviewTextAsset {
                            package_sha256: key.package_sha256.clone(),
                            asset_sha256: asset_sha256.clone(),
                        },
                    )
                    .map(Box::new),
                asset_sha256,
            },
            Action::History => Outcome::History(client.review_decisions(
                Request::new("reviews.decisions").context,
                ListReviewDecisions {
                    package_sha256: key.package_sha256.clone(),
                    item_id: key.item_id.clone().unwrap_or_default(),
                },
            )),
            Action::Record(input) => Outcome::Recorded(
                client
                    .record_review_decision(Request::new("reviews.record_decision").context, *input)
                    .map(Box::new),
            ),
        };
        let _ = sender.send(Completion { key, outcome });
        repaint.request_repaint();
    });
    receiver
}
