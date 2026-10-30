//! Pure UI identity joins; not browser execution, transport ACK faults or media evidence.
use super::{contracts::ResolutionChoice, model::Listing};
use crate::application::{
    catalog::CatalogResult,
    jobs::{JobRecord, JobState, VerifiedJobOutput},
};
use kinekino_jobs::domain::{PreparedJobInput, SourceBlobRef};
use kinekino_media_contracts::api_values::DecimalU64;
use serde_json::json;

#[test]
fn automatic_catalog_progress_stops_on_cancelled_terminal_or_uncertain_submission() {
    use super::{
        CatalogConsumer,
        model::{Active, Intent},
    };
    use crate::{
        application::catalog::Category, assembly::meari::viewer_callbacks::progression::Action,
    };
    let intent = Intent::Category {
        category: Category::Airing,
        page: 1,
    };
    let input = intent.submission().unwrap();
    let job = JobRecord::queued(
        "job-00000000000000000000000000000001"
            .to_owned()
            .try_into()
            .unwrap(),
        PreparedJobInput {
            operation: input.operation,
            parameters: input.parameters,
            source: SourceBlobRef {
                sha256: "a".repeat(64),
                bytes: DecimalU64(1),
            },
            execution_timeout_ms: input.execution_timeout_ms,
        },
    )
    .unwrap();
    let mut owner = CatalogConsumer {
        available: true,
        automatic_enabled: true,
        ..Default::default()
    };
    owner.model.active = Some(Active { intent, job });
    assert!(owner.automatic_catalog_step(false).is_none());
    let old = owner.automatic_catalog_step(true).unwrap();
    assert_eq!(old.action, Action::PollCatalog);
    owner.model.active.as_mut().unwrap().job.revision.0 += 1;
    assert_ne!(old, owner.automatic_catalog_step(true).unwrap());
    owner.model.cancel_requested = true;
    owner.model.active.as_mut().unwrap().job.state = JobState::Completed;
    assert!(owner.automatic_catalog_step(true).is_none());
    owner.model.active.as_mut().unwrap().job.state = JobState::Cancelled;
    assert!(owner.automatic_catalog_step(true).is_none());
    owner.model.active.as_mut().unwrap().job.state = JobState::Running;
    owner.open_unconfirmed = true;
    assert!(owner.automatic_catalog_step(true).is_none());
}

fn completed_resolution() -> JobRecord {
    let mut job = JobRecord::queued(
        "job-00000000000000000000000000000001"
            .to_owned()
            .try_into()
            .unwrap(),
        PreparedJobInput {
            operation: "episodes.resolve".into(),
            parameters: json!({}),
            source: SourceBlobRef {
                sha256: "a".repeat(64),
                bytes: DecimalU64(1),
            },
            execution_timeout_ms: DecimalU64(60_000),
        },
    )
    .unwrap();
    job.state = JobState::Completed;
    job.output = Some(VerifiedJobOutput {
        path: "fixture-only-unopened".into(),
        sha256: "b".repeat(64),
        bytes: DecimalU64(1),
        receipt: json!({"kind":"episode_resolution", "source_kind":"request_descriptor",
            "source_descriptor_sha256":"a".repeat(64), "resolution_artifact":{
                "artifact_id":"c".repeat(64), "body_sha256":"b".repeat(64), "body_bytes":"1",
                "body_path":"fixture-only-unopened", "metadata_path":"fixture-only-unopened"}}),
    });
    job
}

#[test]
fn resolution_inspector_uses_artifact_identity_not_output_body_hash() {
    let job = completed_resolution();
    let id = ResolutionChoice::inspector_id(&job).unwrap();
    assert_eq!(id, "c".repeat(64));
    assert_ne!(id, job.output.unwrap().sha256);
}

#[test]
fn late_resolution_from_another_source_or_nonterminal_job_is_rejected() {
    let mut job = completed_resolution();
    job.input.source.sha256 = "d".repeat(64);
    assert!(ResolutionChoice::inspector_id(&job).is_err());
    job.input.source.sha256 = "a".repeat(64);
    job.state = JobState::Cancelling;
    assert!(ResolutionChoice::inspector_id(&job).is_err());
}

fn listing(hash: &str, title: &str) -> Listing {
    let result: CatalogResult = serde_json::from_value(json!({
        "title":"fixture", "series":null, "entries":[{"id":"same-row", "title":title, "url":"https://catalog.example.test/fixture"}],
        "next_page":null, "site_verified":false,
        "raw_artifact":{"artifact_id":"e".repeat(64), "body_sha256":"f".repeat(64), "body_bytes":"1", "body_path":"unopened", "metadata_path":"unopened"},
        "capture":{"requested_url":"https://catalog.example.test/fixture", "final_url":"https://catalog.example.test/fixture", "redirects":[],
            "observed_at":"fixture", "parser_revision":"fixture", "status":200, "content_type":"text/html", "content_encoding":null,
            "body_sha256":"f".repeat(64), "body_bytes":"1", "browser":null},
        "result_artifact":{"sha256":hash, "bytes":"1", "kind":"catalog_result", "operation":"catalog.search", "source_descriptor_sha256":"a".repeat(64)}
    })).unwrap();
    Listing {
        artifact: result.result_artifact.clone().unwrap(),
        result,
    }
}

#[test]
fn original_result_key_selects_original_entry_and_rejects_replaced_list() {
    let first = listing(&"a".repeat(64), "original A");
    let key = first.key(0);
    assert_eq!(first.select(&key).unwrap().title, "original A");
    let second = listing(&"b".repeat(64), "replacement B");
    assert!(second.select(&key).is_err());
    assert!(second.select(&second.key(1)).is_err());
    assert_eq!(
        second.select(&second.key(0)).unwrap().title,
        "replacement B"
    );
}

fn episode_identity_fixture() -> (
    super::CatalogConsumer,
    crate::application::playback_session::EpisodePlaybackObservation,
    String,
    String,
) {
    use super::{
        OpenedChoice,
        contracts::{SelectedEpisode, SelectedSeries},
        model::{Active, Intent},
    };
    let series = listing(&"b".repeat(64), "Original series");
    let series = SelectedSeries {
        key: series.key(0),
        list: series.artifact.clone(),
        row: series.result.entries[0].clone(),
    };
    let mut episodes = listing(&"a".repeat(64), "Original current title");
    let mut second = episodes.result.entries[0].clone();
    second.id = "other-row".into();
    second.title = "Browse candidate title".into();
    second.url = "https://catalog.example.test/fixture-next".into();
    episodes.result.entries.push(second);
    let current_key = episodes.key(0);
    let browse_key = episodes.key(1);
    let selected = SelectedEpisode {
        key: current_key.clone(),
        series: series.clone(),
        list: episodes.artifact.clone(),
        row: episodes.result.entries[0].clone(),
    };
    let browse = SelectedEpisode {
        key: browse_key.clone(),
        series,
        list: episodes.artifact.clone(),
        row: episodes.result.entries[1].clone(),
    };
    let mut consumer = super::CatalogConsumer::default();
    consumer.model.episodes = Some(episodes);
    consumer.model.active = Some(Active {
        intent: Intent::Episode(browse),
        job: completed_resolution(),
    });
    consumer.opened = Some(OpenedChoice {
        selected,
        opened: crate::application::playback_session::OpenedEpisodePlayback {
            engine_instance_id: DecimalU64(7),
            resolution_artifact_id: "c".repeat(64),
            completion: crate::application::playback_control::ControlCompletion::Unobserved,
        },
    });
    let actual = serde_json::from_value(json!({ "engine_instance_id":"7", "resolution_artifact_id":"c".repeat(64),
        "playback":{"origin":"native_observation","published_age_ms":"0","connected":true,
            "paused":false,"idle":false,"position_us":"10","duration_us":"100","volume_percent":50.0,
            "muted":false,"video_size":null,"audio_channels":null,"has_error":false}, "diagnostics":null,"options":null })).unwrap();
    (consumer, actual, current_key, browse_key)
}
#[test]
fn actual_current_episode_requires_original_engine_artifact_and_differs_from_browse_selection() {
    use meari::domain::Value;
    let (consumer, actual, current, browse) = episode_identity_fixture();
    assert_eq!(
        consumer.props().unwrap()["catalogSelectedEpisode"],
        Value::String(browse)
    );
    let props = consumer.episode_identity_props(Some(&actual));
    assert_eq!(
        props["catalogCurrentEpisode"],
        Value::String(current.clone())
    );
    assert_eq!(
        props["catalogConnectingEpisode"],
        Value::String(String::new())
    );
    let pending = consumer.episode_identity_props(None);
    assert_eq!(
        pending["catalogCurrentEpisode"],
        Value::String(String::new())
    );
    assert_eq!(pending["catalogConnectingEpisode"], Value::String(current));
}
#[test]
fn foreign_disconnected_or_replaced_listing_never_marks_an_episode_current() {
    use meari::domain::Value;
    for mutation in 0..4 {
        let (mut consumer, mut actual, _, _) = episode_identity_fixture();
        match mutation {
            0 => actual.engine_instance_id.0 = 8,
            1 => actual.resolution_artifact_id = "d".repeat(64),
            2 => actual.playback.connected = false,
            _ => consumer.model.episodes = Some(listing(&"e".repeat(64), "Different source")),
        }
        let props = consumer.episode_identity_props(Some(&actual));
        assert_eq!(props["catalogCurrentEpisode"], Value::String(String::new()));
        if mutation == 3 {
            assert_eq!(
                props["catalogConnectingEpisode"],
                Value::String(String::new())
            );
        }
    }
}
