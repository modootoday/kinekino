use super::*;

pub(crate) fn proposal() -> StoredProposal {
    serde_json::from_value(serde_json::json!({"document":{
        "schema_version":"1","proposal_id":"draft-a","source":{"source_id":"source-a","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"original","start_us":null,"end_us":null}]},
        "generated":{"task":"correction","raw_text":"invalid model output"},"draft_revision":"1","state":"needs_repair","draft":null,"import_error":"Needs repair"
    },"acceptance":null})).unwrap()
}
#[test]
fn captured_work_is_separate_and_wrong_source_kind_is_rejected() {
    let captured = proposal();
    let mut snapshot = ProjectProposalSnapshot {
        capture_sha256: canonical_sha256(&captured).unwrap(),
        captured,
        working_draft: Some(ProposalPayload::Correction { edits: vec![] }),
        working_state: WorkingState::NeedsInput,
        proposal_origin: None,
    };
    assert_eq!(snapshot.working_state().unwrap(), WorkingState::NeedsInput);
    let original = canonical_sha256(&snapshot.captured).unwrap();
    snapshot.working_draft = Some(ProposalPayload::Correction {
        edits: vec![crate::domain::proposals::Correction {
            cue_id: "cue-1".into(),
            text: "repaired".into(),
        }],
    });
    assert_eq!(snapshot.working_state().unwrap(), WorkingState::Valid);
    assert_eq!(original, canonical_sha256(&snapshot.captured).unwrap());
    snapshot.working_draft = Some(ProposalPayload::DocumentCorrection { edits: vec![] });
    assert!(snapshot.working_state().is_err());
}
#[test]
fn episode_locator_rejects_credentials_query_fragment_and_lookalike() {
    for url in [
        "https://evil.test/e/a",
        "https://name@catalog.example.test/e/a",
        "https://catalog.example.test/e/a?x=1",
        "https://catalog.example.test/e/a#x",
        "https://catalog.example.test/e/",
    ] {
        let selection = WatchingSelection {
            label: "Episode".into(),
            source: WatchingSource::EpisodePage {
                source_id: "episode-a".into(),
                episode_url: url.into(),
            },
        };
        assert!(selection.validate().is_err(), "{url}");
    }
    WatchingSelection {
        label: "Episode".into(),
        source: WatchingSource::EpisodePage {
            source_id: "episode-a".into(),
            episode_url: "https://catalog.example.test/e/episode-a".into(),
        },
    }
    .validate()
    .unwrap();
}
#[test]
fn aggregate_record_and_working_limits_are_independent_of_individual_proposals() {
    let mut project = ProjectRevision {
        schema_version: "1".into(),
        project_id: "work".into(),
        revision: crate::domain::api_values::DecimalU64(1),
        parent_revision: None,
        title: "Work".into(),
        watching: None,
        proposals: vec![],
        work_state: ProjectWorkState::default(),
    };
    for n in 0..12 {
        let mut captured = proposal();
        captured.document.proposal_id = format!("draft-{n}");
        captured.document.generated.raw_text = "x".repeat(262144);
        captured.validate().unwrap();
        project.proposals.push(ProjectProposalSnapshot {
            capture_sha256: canonical_sha256(&captured).unwrap(),
            captured,
            working_draft: None,
            working_state: WorkingState::Absent,
            proposal_origin: None,
        });
    }
    project.work_state = project.derived_work_state();
    assert!(project.validate().is_err());
    project.proposals.truncate(2);
    for snapshot in &mut project.proposals {
        snapshot.working_draft = Some(ProposalPayload::Correction {
            edits: vec![crate::domain::proposals::Correction {
                cue_id: "cue-1".into(),
                text: "x".repeat(270000),
            }],
        });
        snapshot.working_state = WorkingState::NeedsInput;
    }
    project.work_state = project.derived_work_state();
    assert!(serde_json::to_vec(&project).unwrap().len() < MAX_RECORD_BYTES);
    assert!(project.validate().is_err());
}
