//! Request/response and trace assertions for the original opt-in commit journey.
use super::*;

pub(super) fn transcript_passes(calls: &[Request], replies: &[Value], seed_count: usize) -> bool {
    use crate::domain::proposals::StoredProposal;
    let commands = [
        "proposals.create",
        "proposals.create",
        "proposals.inspect",
        "proposals.list",
        "proposals.inspect",
        "proposals.inspect",
        "proposals.edit",
        "proposals.inspect",
        "proposals.accept",
        "proposals.inspect",
        "proposals.list",
        "proposals.inspect",
        "proposals.inspect",
    ];
    if seed_count != 3
        || calls.len() != commands.len()
        || replies.len() != commands.len()
        || !calls
            .iter()
            .zip(commands)
            .all(|(call, command)| call.command == command)
        || !calls.iter().zip(replies).all(|(call, reply)| {
            reply["request_id"] == call.context.request_id
                && reply["ok"] == true
                && reply["error"].is_null()
        })
    {
        return false;
    }
    let decode =
        |index: usize| serde_json::from_value::<StoredProposal>(replies[index]["data"].clone());
    let (Ok(original), Ok(saved), Ok(accepted)) = (decode(2), decode(7), decode(9)) else {
        return false;
    };
    original.validate().is_ok()
        && saved.validate().is_ok()
        && accepted.validate().is_ok()
        && original.acceptance.is_none()
        && saved.acceptance.is_none()
        && accepted.acceptance.is_some()
        && [2, 4, 5, 6, 7, 8, 9, 11, 12]
            .into_iter()
            .all(|index| calls[index].payload["proposal_id"] == original.document.proposal_id)
        && [4, 5]
            .into_iter()
            .all(|index| replies[index]["data"] == replies[2]["data"])
        && serde_json::to_value(&saved.document).ok().as_ref() == Some(&replies[6]["data"])
        && serde_json::to_value(&accepted.acceptance).ok().as_ref() == Some(&replies[8]["data"])
        && [11, 12]
            .into_iter()
            .all(|index| replies[index]["data"] == replies[9]["data"])
        && serde_json::to_value(&saved.document.source).ok()
            == serde_json::to_value(&original.document.source).ok()
        && serde_json::to_value(&accepted.document.source).ok()
            == serde_json::to_value(&original.document.source).ok()
        && serde_json::to_value(&saved.document.draft).ok()
            == serde_json::to_value(&accepted.document.draft).ok()
        && serde_json::to_value(&saved.document.draft).ok()
            != serde_json::to_value(&original.document.draft).ok()
}
pub(super) fn trace_passes(events: &[Value], case: Case) -> bool {
    let of = |name: &str| {
        events
            .iter()
            .filter(|event| event["kind"] == name)
            .collect::<Vec<_>>()
    };
    if !of("LiveFault").is_empty()
        || !of("LiveHostReturnRequested").is_empty()
        || !of("LiveModelProofError").is_empty()
        || [
            "ChooserGeneration",
            "ChooserActiveIdentity",
            "LiveEntryCommitted",
            "LiveEntryFrame",
            "LiveActivated",
        ]
        .into_iter()
        .any(|kind| of(kind).len() != 2)
        || [
            "LiveSaveRequested",
            "LiveReturnCommitted",
            "LiveViewingFrame",
            "LiveSourceNavigationAction",
            "LiveSourceNavigationAdmitted",
            "LiveSourceReturnRequested",
            "ChooserJourneyComplete",
        ]
        .into_iter()
        .any(|kind| of(kind).len() != 1)
        || of("ChooserSourceAction").len() != 8
    {
        return false;
    }
    let shown = of("LiveInputDisplayed");
    let stages = [
        "journey-initial",
        "journey-saved",
        "journey-accept-requested",
        "journey-accepted",
        "journey-reopened",
    ];
    let mut previous = None;
    for stage in stages {
        let matches = shown
            .iter()
            .filter(|event| event["data"][0] == stage)
            .collect::<Vec<_>>();
        let [event] = matches.as_slice() else {
            return false;
        };
        let Some(sequence) = event["sequence"].as_u64() else {
            return false;
        };
        if previous.is_some_and(|previous| previous >= sequence) {
            return false;
        }
        previous = Some(sequence);
    }
    let changed = shown
        .iter()
        .filter(|event| event["data"][0] == "journey-changed")
        .collect::<Vec<_>>();
    let fields = if case == Case::Card { 2 } else { 1 };
    let at = |stage: &str| {
        shown
            .iter()
            .find(|event| event["data"][0] == stage)
            .unwrap()
    };
    if changed.len() != fields
        || !changed.iter().enumerate().all(|(index, event)| {
            event["data"][1] == index as u64
                && at("journey-initial")["sequence"].as_u64() < event["sequence"].as_u64()
                && event["sequence"].as_u64() < at("journey-saved")["sequence"].as_u64()
        })
    {
        return false;
    }
    let identities = of("ChooserActiveIdentity");
    let first = &identities[0]["data"]["input_selection"];
    let reopened = &identities[1]["data"]["input_selection"];
    at("journey-accepted")["data"][3] == at("journey-reopened")["data"][3]
        && at("journey-accepted")["data"][4] == at("journey-reopened")["data"][4]
        && [
            "proposal_id",
            "source_id",
            "source_revision",
            "source_sha256",
            "target",
        ]
        .into_iter()
        .all(|field| first[field] == reopened[field])
        && shown
            .iter()
            .find(|event| event["data"][0] == "journey-accepted")
            .unwrap()["sequence"]
            .as_u64()
            < of("LiveSourceNavigationAction")[0]["sequence"].as_u64()
}
