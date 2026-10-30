//! No-admission wire boundaries on one original playback instance.
use super::{manual_fixture::*, *};
use kinekino_player::application::skip_segments::CancelStatus;

pub(super) fn run() -> Result<(), ScenarioFailure> {
    let scenario = Scenario::new("native-manual-negative")?;
    scenario.with_host("negative", |calls, engine| {
        let view = media(calls, &scenario.input, engine)?;
        ensure(
            calls,
            view.snapshot.revision == DecimalU64(1) && view.snapshot.pass_id == DecimalU64(1),
            "Initial session differs",
        )?;
        let session = view.session;
        let scope = scope(calls, "negative-capabilities")?;
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
            edited.revision == DecimalU64(2)
                && edited.segments.iter().any(|item| {
                    item.id == SEGMENT
                        && item.revision == DecimalU64(1)
                        && item.verified_revision.is_none()
                }),
            "Original unverified annotation differs",
        )?;
        let unverified = export(calls, "unverified", scope, &session, 2, "manual", false)?;
        state(calls, &unverified, &session, 2, 1)?;
        blocked(calls, &unverified, "unverified")?;
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
            "Accepted revision differs",
        )?;
        calls.record("before-stale.json", &accepted)?;

        let stale = export(calls, "stale-cancel", scope, &session, 2, "manual", true)?;
        state(calls, &stale, &session, 3, 1)?;
        conflict(calls, &stale)?;
        let after = snapshot(calls, "after-stale", &session, 3, 1)?;
        let encoded = |value: &SkipSnapshot| {
            serde_json::to_value(value).map_err(|error| calls.error(io::Error::other(error)))
        };
        ensure(
            calls,
            encoded(&accepted)? == encoded(&after)?,
            "Stale cancelled request changed stored state",
        )?;

        let mut foreign = session.clone();
        foreign.binding.engine_instance_id = DecimalU64(
            session
                .binding
                .engine_instance_id
                .0
                .checked_add(1)
                .ok_or_else(|| calls.error(invalid("No distinct bounded engine id")))?,
        );
        rejected(
            calls,
            "foreign-binding",
            SKIP_EXPORT,
            json!({"issuer_scope":scope,"session":foreign,"input":{
                "segment_id":SEGMENT,"intent":"manual","cancelled":false}}),
            Some(DecimalU64(3)),
            ErrorCode::RevisionConflict,
        )?;
        let unchanged = snapshot(calls, "after-foreign", &session, 3, 1)?;
        ensure(
            calls,
            encoded(&accepted)? == encoded(&unchanged)?,
            "Foreign binding changed stored state",
        )?;

        let cancelled = export(calls, "cancel-before", scope, &session, 3, "manual", true)?;
        state(calls, &cancelled, &session, 4, 1)?;
        ensure(
            calls,
            matches!(
                cancelled.result,
                SkipExportAction::Cancelled {
                    status: CancelStatus::BeforeAdmission
                }
            ),
            "Before-admission cancellation differs",
        )?;
        let cancelled_state = snapshot(calls, "after-cancel", &session, 4, 1)?;
        rejected(
            calls,
            "general-seek-closed",
            "segments.skip",
            json!({"session":session,"input":{"segment_id":SEGMENT,
                "intent":"manual","cancelled":false}}),
            Some(DecimalU64(4)),
            ErrorCode::Unsupported,
        )?;
        let general_state = snapshot(calls, "after-general-rejection", &session, 4, 1)?;
        ensure(
            calls,
            encoded(&cancelled_state)? == encoded(&general_state)?,
            "Unavailable general seek changed canonical state",
        )?;
        for revision in [2, 3] {
            let origin = SkipExportOrigin {
                session: session.clone(),
                pass_id: DecimalU64(1),
                request_revision: DecimalU64(revision),
                segment_id: SEGMENT.into(),
            };
            let listed: v1::NativeExportListV1 = data(
                calls,
                &format!("empty-origin-{revision}"),
                EXPORT_LIST,
                json!({"issuer_scope":scope,"origin":origin}),
                None,
            )?;
            ensure(
                calls,
                listed.scope() == scope && listed.items().is_empty(),
                "Rejected original origin acquired an export row",
            )?;
        }
        calls.record(
            "negative-summary.json",
            &json!({"session":session,
            "revision":"4","pass":"1","observed_admitted_results":0,
            "native_attempt_counter":"not exposed by default artifact"}),
        )?;
        Ok(())
    })
}
