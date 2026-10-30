//! Explicit override, original result retention and owner-generation rejection.
use super::{manual_fixture::*, *};
use kinekino_player::application::skip_segments::CancelStatus;

struct Saved {
    endpoint: String,
    scope: v1::NativeIssuerScopeV1,
    origin: SkipExportOrigin,
    admission: Admission,
}

fn change(
    calls: &Calls<'_>,
    label: &str,
    session: &SessionRef,
    revision: u64,
    input: Value,
    expected_revision: u64,
    pass: u64,
) -> Check<OverrideResult> {
    let result: OverrideResult = data(
        calls,
        label,
        "segments.override",
        json!({"session":session,"input":input}),
        Some(DecimalU64(revision)),
    )?;
    ensure(
        calls,
        result.snapshot.revision == DecimalU64(expected_revision)
            && result.snapshot.pass_id == DecimalU64(pass),
        "Override state differs",
    )?;
    Ok(result)
}

pub(super) fn run() -> Result<(), ScenarioFailure> {
    let scenario = Scenario::new("native-manual-owner")?;
    let saved = scenario.with_host("original", |calls, engine| {
        let view = media(calls, &scenario.input, engine)?;
        ensure(
            calls,
            view.snapshot.revision == DecimalU64(1) && view.snapshot.pass_id == DecimalU64(1),
            "Initial session differs",
        )?;
        let session = view.session;
        let scope = scope(calls, "owner-capabilities")?;
        let edited: SkipSnapshot = data(
            calls,
            "edit",
            "segments.edit",
            json!({"session":session,"input":{"segment_id":SEGMENT,"kind":"opening",
                "in_us":"0","out_us":scenario.input.target}}),
            Some(DecimalU64(1)),
        )?;
        ensure(
            calls,
            edited.revision == DecimalU64(2),
            "Edit revision differs",
        )?;
        let accepted: SkipSnapshot = data(
            calls,
            "accept",
            "segments.accept_skip",
            json!({"session":session,"input":{"segment_id":SEGMENT,
                "annotation_revision":"1"}}),
            Some(DecimalU64(2)),
        )?;
        ensure(
            calls,
            accepted.revision == DecimalU64(3),
            "Accept revision differs",
        )?;
        let policy: SkipSnapshot = data(
            calls,
            "policy",
            "segments.set_skip_policy",
            json!({"session":session,"input":{"policy":{
                "opening":"auto","ending":"manual"}}}),
            Some(DecimalU64(3)),
        )?;
        ensure(
            calls,
            policy.revision == DecimalU64(4),
            "Policy revision differs",
        )?;
        change(
            calls,
            "manual-override",
            &session,
            4,
            json!({"action":"manual_seek"}),
            5,
            1,
        )?;
        let overridden = export(calls, "overridden", scope, &session, 5, "automatic", false)?;
        state(calls, &overridden, &session, 5, 1)?;
        blocked(calls, &overridden, "overridden")?;
        change(
            calls,
            "explicit-new-pass",
            &session,
            5,
            json!({"action":"begin_new_pass"}),
            6,
            2,
        )?;
        let paused = export(
            calls,
            "paused-new-pass",
            scope,
            &session,
            6,
            "automatic",
            false,
        )?;
        state(calls, &paused, &session, 6, 2)?;
        blocked(calls, &paused, "paused_or_unknown")?;

        let origin = SkipExportOrigin {
            session: session.clone(),
            pass_id: DecimalU64(2),
            request_revision: DecimalU64(6),
            segment_id: SEGMENT.into(),
        };
        // One explicit manual admission creates the row needed by the owner negatives.
        let result = export(
            calls,
            "manual-original-once",
            scope,
            &session,
            6,
            "manual",
            false,
        )?;
        state(calls, &result, &session, 7, 2)?;
        let original = admission(result, &origin, scope, scenario.input.target)
            .map_err(|error| calls.error(error))?;
        calls.record(
            "original-admission.json",
            &json!({"reference":original.reference,
            "metadata":original.metadata,"post_error":original.post_error}),
        )?;
        ensure(
            calls,
            original.post_error.is_none(),
            "Admitted-with-error retained; stop without replay",
        )?;
        ensure(
            calls,
            original.metadata.annotation().annotation_revision() == DecimalU64(1)
                && original.metadata.annotation().verified_revision() == DecimalU64(1),
            "Original accepted annotation metadata changed",
        )?;
        let replay = export(
            calls,
            "explicit-stale-replay",
            scope,
            &session,
            6,
            "manual",
            false,
        )?;
        state(calls, &replay, &session, 7, 2)?;
        conflict(calls, &replay)?;
        let duplicate = export(
            calls,
            "already-attempted",
            scope,
            &session,
            7,
            "automatic",
            false,
        )?;
        state(calls, &duplicate, &session, 7, 2)?;
        blocked(calls, &duplicate, "already_attempted")?;
        let cancelled = change(
            calls,
            "cancel-after",
            &session,
            7,
            json!({"action":"cancel","segment_id":SEGMENT}),
            8,
            2,
        )?;
        ensure(
            calls,
            cancelled.cancel_status == Some(CancelStatus::TooLateOrUnobserved),
            "Post-admission cancellation claimed withdrawal",
        )?;
        let suppressed = export(
            calls,
            "after-cancel",
            scope,
            &session,
            8,
            "automatic",
            false,
        )?;
        state(calls, &suppressed, &session, 8, 2)?;
        blocked(calls, &suppressed, "overridden")?;
        let before = snapshot(calls, "before-stale-override", &session, 8, 2)?;
        rejected(
            calls,
            "stale-override",
            "segments.override",
            json!({"session":session,"input":{"action":"manual_seek"}}),
            Some(DecimalU64(7)),
            ErrorCode::RevisionConflict,
        )?;
        let after = snapshot(calls, "after-stale-override", &session, 8, 2)?;
        let encode = |value: &SkipSnapshot| {
            serde_json::to_value(value).map_err(|error| calls.error(io::Error::other(error)))
        };
        ensure(
            calls,
            encode(&before)? == encode(&after)?,
            "Stale override mutated state",
        )?;
        let closed: ClosedSkipSession = data(
            calls,
            "close-policy",
            "segments.close",
            json!(session),
            Some(DecimalU64(8)),
        )?;
        ensure(
            calls,
            closed.closed
                && !closed.playback_closed
                && closed.skip_session_id == session.skip_session_id,
            "Policy close changed playback ownership",
        )?;
        rejected(
            calls,
            "closed-policy-inspect",
            "segments.inspect",
            json!(session),
            None,
            ErrorCode::NotFound,
        )?;

        let payload = json!({"issuer_scope":scope,"origin":origin,
            "reference":original.reference});
        let listed: v1::NativeExportListV1 = data(
            calls,
            "original-row-after-policy-close",
            EXPORT_LIST,
            json!({"issuer_scope":scope,"origin":origin}),
            None,
        )?;
        ensure(
            calls,
            listed.scope() == scope
                && listed.items().len() == 1
                && listed.items()[0].reference() == original.reference
                && listed.items()[0].metadata() == &original.metadata,
            "Policy close lost or replaced the original native row",
        )?;
        let mut terminal = false;
        for index in 0..32 {
            let result: v1::NativeExportResultV1 = data(
                calls,
                &format!("owner-peek-{index}"),
                EXPORT_PEEK,
                payload.clone(),
                None,
            )?;
            calls.record(&format!("owner-result-{index}.json"), &result)?;
            if acknowledged(&result, &original).map_err(|error| calls.error(error))? {
                terminal = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        ensure(
            calls,
            terminal,
            "Original terminal result not observed after policy close",
        )?;
        let consumed: v1::NativeExportResultV1 = data(
            calls,
            "owner-cleanup-consume",
            EXPORT_CONSUME,
            payload,
            None,
        )?;
        ensure(
            calls,
            acknowledged(&consumed, &original).map_err(|error| calls.error(error))?,
            "Cleanup consumed a different original result",
        )?;
        Ok(Saved {
            endpoint: calls.endpoint.into(),
            scope,
            origin,
            admission: original,
        })
    })?;

    // The first original host has already passed EOF/worker/family checks here.
    let old_calls = scenario.calls(&saved.endpoint);
    let old_payload = json!({"issuer_scope":saved.scope,"origin":saved.origin,
        "reference":saved.admission.reference});
    let stopped = rejected(
        &old_calls,
        "old-endpoint-not-sent",
        EXPORT_PEEK,
        old_payload.clone(),
        None,
        ErrorCode::NotFound,
    )
    .map_err(ScenarioFailure::host)?;
    ensure(
        &old_calls,
        stopped["error"]["details"]["delivery"] == "not_sent"
            && stopped["error"]["details"]["automatic_retry"] == false,
        "Closed endpoint was retargeted or retried",
    )
    .map_err(ScenarioFailure::host)?;
    scenario.with_host("restarted-no-media", |calls, engine| {
        let replacement_scope = scope(calls, "replacement-capabilities")?;
        ensure(
            calls,
            calls.endpoint != saved.endpoint
                && replacement_scope != saved.scope
                && engine.get().is_none(),
            "Replacement owner identity did not change",
        )?;
        rejected(
            calls,
            "old-issuer-rejected",
            EXPORT_PEEK,
            old_payload,
            None,
            ErrorCode::RevisionConflict,
        )?;
        rejected(
            calls,
            "old-session-rejected",
            "segments.inspect",
            json!(saved.origin.session),
            None,
            ErrorCode::NotFound,
        )?;
        calls.record(
            "owner-summary.json",
            &json!({"old_scope":saved.scope,
            "new_scope":replacement_scope,"original_reference":saved.admission.reference,
            "new_media_open":false,"original_manual_admitted_results":1,
            "native_attempt_counter":"not exposed by default artifact"}),
        )?;
        Ok(())
    })
}
