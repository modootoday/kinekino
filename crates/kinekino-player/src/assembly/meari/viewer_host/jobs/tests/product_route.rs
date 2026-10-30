//! Real product source/callback/SDK route transactions; synthetic frame notices are policy only.
use super::*;
#[path = "product_route_fixture.rs"]
mod fixture;
use fixture::{CaseFailure, with_empty_rig, with_rig};
use std::sync::atomic::Ordering;

#[test]
#[ignore = "Assigned original LocalHost slot; original JOB active/recovery factory and exports, no OS/render"]
fn jobs_caption_factories_reject_missing_types_and_original_exports_reject_foreign_stale_retired_and_closing()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        rig.enter();
        let calls = rig.probe.calls.lock().unwrap().len();
        let tickets = rig.host().callbacks.borrow().tickets().unwrap().len();
        let jobs = rig.host().jobs.clone();
        super::super::document::caption_contract_tests::assert_original_document(
            jobs.borrow_mut().document.as_mut().unwrap(),
        );
        assert_eq!(rig.probe.calls.lock().unwrap().len(), calls);
        assert_eq!(
            rig.host().callbacks.borrow().tickets().unwrap().len(),
            tickets
        );
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; no OS window or renderer"]
fn default_jobs_waits_for_checked_frame_lists_actual_rows_and_selects_exact_sdk_ids()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        let original = rig.document().stamp().unwrap();
        rig.enter();
        assert_eq!(rig.calls("jobs.list"), 1);
        assert_eq!(rig.calls("jobs.get"), 0);
        assert!(rig.text().contains("선택"));
        assert!(rig.handle.as_ref().unwrap().attach().is_err());
        for index in 0..2 {
            rig.select(index);
            rig.wait(|r| r.calls("jobs.get") == index + 1 && r.detail_ready(index));
            assert!(rig.text().contains(rig.records[index].job_id.as_str()));
            let stamp = rig.document().stamp().unwrap();
            assert_eq!(stamp.host_id, original.host_id);
            assert_eq!(stamp.owner, original.owner);
            assert_eq!(stamp.definition_revision, original.definition_revision);
        }
        let requests = rig.probe.calls.lock().unwrap();
        let ids = requests
            .iter()
            .filter(|r| r.command == "jobs.get")
            .map(|r| r.payload["job_id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            ids,
            rig.records
                .iter()
                .map(|r| r.job_id.as_str().to_owned())
                .collect::<Vec<_>>()
        );
        assert!(
            requests
                .iter()
                .filter(|r| r.command.starts_with("jobs."))
                .all(|r| r.context.timeout_ms.is_none()
                    && r.context.idempotency_key.is_none()
                    && r.context.expected_revision.is_none())
        );
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; actual callback batch barrier, no renderer"]
fn nonzero_callback_barrier_defers_entry_and_old_viewing_callback_is_retired()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        let original = rig.pending("onOpenJobs", None);
        rig.admit(&original);
        let before = rig.document().stamp().unwrap();
        let mut turn = rig.turn();
        turn.remaining_callbacks = 1;
        rig.host_mut().after_dispatch(turn).unwrap();
        assert_eq!(rig.document().stamp().unwrap(), before);
        assert!(rig.host().jobs.borrow().entry.is_none());
        assert_eq!(rig.calls("jobs.list"), 0);
        rig.tick();
        rig.wait(|r| {
            r.host()
                .jobs
                .borrow()
                .entry
                .as_ref()
                .is_some_and(|e| e.phase == EntryPhase::Armed)
        });
        let current = rig.document().stamp().unwrap();
        assert!(matches!(
            rig.host_mut().admit_callback(&current, &original),
            IntentAdmission::Rejected(_)
        ));
        rig.frame();
        rig.tick();
        rig.wait(|r| r.calls("jobs.list") == 1 && r.enabled("onRefreshJobs"));
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; real SDK result with response-delivery fault"]
fn failed_get_refresh_keeps_last_good_record_and_never_submits_a_job_mutation()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        rig.enter();
        rig.select(0);
        rig.wait(|r| r.calls("jobs.get") == 1 && r.detail_ready(0));
        let id = rig.records[0].job_id.as_str().to_owned();
        rig.probe.fail_get.store(true, Ordering::Release);
        rig.click("onRefreshJob", None);
        rig.wait(|r| r.text().contains("Original get response delivery fault"));
        assert!(rig.text().contains(&id));
        assert_eq!(rig.calls("jobs.get"), 2);
        assert!(
            rig.probe
                .calls
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.command.starts_with("jobs."))
                .all(|r| matches!(r.command.as_str(), "jobs.list" | "jobs.get"))
        );
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; original pending get and same-owner close"]
fn pending_get_drains_before_return_and_unproved_frame_cannot_finish_observation()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        rig.enter();
        rig.probe.hold_get.store(true, Ordering::Release);
        rig.select(0);
        rig.wait(|r| r.probe.get_entered.load(Ordering::Acquire) && !r.enabled("onSelectJob"));
        // Selection rows serialize behind the original query; navigation stays explicit.
        assert!(!rig.enabled("onSelectJob"));
        rig.begin_return();
        rig.tick();
        assert_eq!(rig.observed(), "studio.jobs");
        assert!(rig.handle.as_ref().unwrap().attach().is_err());
        rig.probe.hold_get.store(false, Ordering::Release);
        rig.wait(|r| {
            r.host()
                .jobs
                .borrow()
                .returning
                .as_ref()
                .is_some_and(|request| request.phase == ReturnPhase::Armed)
        });
        assert_eq!(rig.observed(), "studio.jobs");
        rig.frame();
        rig.tick();
        assert!(rig.host().jobs.borrow().returning.is_some());
        assert_eq!(rig.observed(), "studio.jobs");
        assert!(!rig.text().contains(rig.records[0].job_id.as_str()));
        assert_eq!(rig.calls("jobs.get"), 1);
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; real retained proof rearm, synthetic frame cannot authenticate completion"]
fn committed_return_recovers_via_current_viewing_export_without_recommit_or_resubmit()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        rig.enter();
        let old_jobs = rig.pending("onRefreshJobs", None);
        rig.begin_return();
        let observer = rig.observer.as_ref().unwrap().clone();
        let busy = observer.borrow_mut();
        rig.tick();
        let (receipt, committed) = {
            let jobs = rig.host().jobs.borrow();
            let request = jobs.returning.as_ref().unwrap();
            assert_eq!(request.phase, ReturnPhase::CommittedAwaitingArm);
            assert!(request.error.is_some());
            assert!(request.first_error.is_some());
            request.committed.as_ref().unwrap().clone()
        };
        assert_eq!(rig.document().stamp().unwrap(), committed);
        assert_eq!(receipt.after(), &committed);
        let calls = rig.probe.calls.lock().unwrap().len();
        let current = rig.document().stamp().unwrap();
        assert!(matches!(
            rig.host_mut().admit_callback(&current, &old_jobs),
            IntentAdmission::Rejected(_)
        ));
        rig.tick(); // No implicit retry and no lost actual commit receipt.
        let idle = rig.document().stamp().unwrap();
        assert_eq!(idle.document_revision, committed.document_revision);
        assert!(idle.turn_sequence > committed.turn_sequence);
        drop(busy);
        rig.click("onOpenJobs", None);
        let current = rig.document().stamp().unwrap();
        assert!(current.document_revision > committed.document_revision);
        assert!(current.turn_sequence > committed.turn_sequence);
        {
            let jobs = rig.host().jobs.borrow();
            let request = jobs.returning.as_ref().unwrap();
            assert_eq!(request.phase, ReturnPhase::Armed);
            assert!(request.error.is_none());
            assert!(request.first_error.is_some());
            assert!(request.proof_error.is_none());
            let proof = request.proof.as_ref().unwrap();
            assert_eq!(proof.receipt(), &receipt);
            assert_eq!(
                rig.document().validate_retained_commit(proof).unwrap(),
                current
            );
            assert_eq!(
                request.committed.as_ref().unwrap().0.after(),
                receipt.after()
            );
        }
        assert_eq!(rig.probe.calls.lock().unwrap().len(), calls);
        assert_eq!(rig.observed(), "studio.jobs");
        rig.frame();
        rig.tick();
        assert_eq!(rig.observed(), "studio.jobs");
        assert!(rig.host().jobs.borrow().returning.is_some());
        assert!(rig.host().jobs.borrow().completed.is_none());
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; new projection invalidates retained proof, original receipt retained"]
fn new_projection_rejects_rearm_and_keeps_original_return_receipt() -> Result<(), Box<CaseFailure>>
{
    with_rig(true, |rig| {
        rig.enter();
        rig.begin_return();
        let observer = rig.observer.as_ref().unwrap().clone();
        let busy = observer.borrow_mut();
        rig.tick();
        drop(busy);
        let original = rig
            .host()
            .jobs
            .borrow()
            .returning
            .as_ref()
            .unwrap()
            .committed
            .as_ref()
            .unwrap()
            .clone();
        let bindings = rig.host().callbacks.borrow().bindings().unwrap();
        let props = rig.host().viewing_props().unwrap();
        let generation = rig.host().viewing_generation;
        rig.host()
            .commit_viewing_props(bindings, props, generation)
            .unwrap();
        let changed = rig.document().stamp().unwrap();
        assert_ne!(changed, original.1);
        let calls = rig.probe.calls.lock().unwrap().len();
        rig.click("onOpenJobs", None);
        let jobs = rig.host().jobs.borrow();
        let request = jobs.returning.as_ref().unwrap();
        assert_eq!(request.phase, ReturnPhase::CommittedAwaitingArm);
        assert!(matches!(
            request.error,
            Some(ViewerHostError::ForeignCheckpoint)
        ));
        assert!(request.proof_error.is_some());
        assert!(request.first_proof_error.is_some());
        assert!(
            rig.document()
                .validate_retained_commit(request.proof.as_ref().unwrap())
                .is_err()
        );
        assert_eq!(
            request.committed.as_ref().unwrap().0.after(),
            original.0.after()
        );
        assert!(rig.document().stamp().unwrap().document_revision > changed.document_revision);
        assert_eq!(rig.probe.calls.lock().unwrap().len(), calls);
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; supplied SDK Unsupported is not upgraded by UI"]
fn unconfigured_sdk_shows_real_list_error_and_does_not_create_an_execution_backend()
-> Result<(), Box<CaseFailure>> {
    with_rig(false, |rig| {
        rig.enter();
        assert_eq!(rig.calls("jobs.list"), 1);
        assert_eq!(rig.calls("jobs.get"), 0);
        assert!(!rig.text().contains("subtitle.convert"));
        assert!(rig.text().contains("Unsupported"));
        assert!(rig.enabled("onRefreshJobs"));
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; actual empty journal, no fabricated demo rows"]
fn empty_list_is_distinct_from_loading_error_and_no_selection() -> Result<(), Box<CaseFailure>> {
    with_empty_rig(|rig| {
        rig.enter();
        assert_eq!(rig.calls("jobs.list"), 1);
        assert_eq!(rig.calls("jobs.get"), 0);
        assert!(!rig.enabled("onSelectJob"));
        assert!(rig.enabled("onRefreshJobs"));
        assert!(!rig.text().contains("Unsupported"));
        assert!(!rig.text().contains("subtitle.convert"));
        let responses = rig.probe.responses.lock().unwrap();
        assert!(responses.iter().any(|response| {
            response["ok"] == true
                && response["data"]["jobs"]
                    .as_array()
                    .is_some_and(Vec::is_empty)
        }));
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; same-worker detail-to-list refresh"]
fn returning_to_list_consumer_submits_exactly_one_refresh_and_preserves_selected_snapshot()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        rig.enter();
        rig.select(0);
        rig.wait(|r| r.calls("jobs.get") == 1 && r.detail_ready(0));
        let selected = rig.records[0].job_id.as_str().to_owned();
        rig.click("onRefreshJobs", None);
        rig.wait(|r| r.calls("jobs.list") == 2 && r.enabled("onRefreshJobs"));
        assert_eq!(rig.calls("jobs.get"), 1);
        assert!(rig.text().contains(&selected));
        assert!(rig.handle.as_ref().unwrap().attach().is_err());
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; counted notifier requests and actual runtime Completion, no event loop"]
fn source_continuations_advance_entry_selection_and_return_then_stop_requesting_wakes()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        let before = rig.wake_requests();
        rig.click("onOpenJobs", None);
        assert_eq!(rig.calls("jobs.list"), 0);
        assert_eq!(rig.wake_requests(), before + 1);
        assert!(rig.drive_wake());
        assert!(
            rig.host()
                .jobs
                .borrow()
                .entry
                .as_ref()
                .is_some_and(|entry| entry.phase == EntryPhase::Armed)
        );
        assert!(!rig.drive_wake());
        assert_eq!(rig.wake_requests(), before + 1);
        // Synthetic frame policy delivery is explicit, not claimed as event-loop wake proof.
        rig.frame();
        rig.tick();
        rig.wait_wakes(|r| r.calls("jobs.list") == 1 && r.enabled("onRefreshJobs"));
        rig.drain_wakes();
        let before_select = rig.wake_requests();
        rig.select(0);
        assert_eq!(rig.calls("jobs.get"), 0);
        assert_eq!(rig.wake_requests(), before_select + 1);
        rig.wait_wakes(|r| r.detail_ready(0));
        rig.drain_wakes();
        let settled = rig.wake_requests();
        assert!(!rig.drive_wake());
        assert_eq!(rig.wake_requests(), settled);
        rig.click("onNavigate", Some("시청으로 돌아가기"));
        rig.wait_wakes(|r| {
            r.host()
                .jobs
                .borrow()
                .returning
                .as_ref()
                .is_some_and(|request| request.phase == ReturnPhase::Armed)
        });
        rig.drain_wakes();
        let armed = rig.wake_requests();
        assert!(!rig.drive_wake());
        assert_eq!(rig.wake_requests(), armed);
        rig.frame();
        rig.tick();
        assert!(rig.host().jobs.borrow().returning.is_some());
        assert_eq!(rig.observed(), "studio.jobs");
        assert!(!rig.drive_wake());
        assert_eq!(rig.wake_requests(), armed);
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; notifier failure retained without automatic retry"]
fn continuation_notification_failure_keeps_original_owners_and_does_not_spin()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        rig.fail_wake();
        rig.click("onOpenJobs", None);
        let requested = rig.wake_requests();
        assert_eq!(requested, 1);
        assert!(
            rig.host()
                .jobs
                .borrow()
                .wake_error
                .as_ref()
                .is_some_and(|error| error.message == "Original continuation wake delivery fault")
        );
        assert!(rig.host().jobs.borrow().entry.is_some());
        assert!(rig.handle.as_ref().unwrap().attach().is_err());
        assert!(!rig.drive_wake());
        assert_eq!(rig.wake_requests(), requested);
        assert_eq!(rig.calls("jobs.list"), 0);
        assert_eq!(rig.calls("jobs.get"), 0);
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; actual current-key batch duplicates, no checked-frame success claim"]
fn armed_current_duplicates_clear_native_batch_and_old_or_other_intents_are_rejected()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        let old = rig.pending("onOpenJobs", None);
        rig.enter();
        rig.drain_wakes();
        rig.click("onNavigate", Some("시청으로 돌아가기"));
        rig.wait_wakes(|r| {
            r.host()
                .jobs
                .borrow()
                .returning
                .as_ref()
                .is_some_and(|request| request.phase == ReturnPhase::Armed)
        });
        rig.drain_wakes();
        let original = rig
            .host()
            .jobs
            .borrow()
            .returning
            .as_ref()
            .unwrap()
            .committed
            .as_ref()
            .unwrap()
            .0
            .clone();
        let calls = rig.probe.calls.lock().unwrap().len();
        let wakes = rig.wake_requests();
        let document = rig.document();
        let path = document
            .try_with(|document| {
                document
                    .module_projection()
                    .unwrap()
                    .callback_bindings()
                    .into_iter()
                    .find(|(_, callback)| callback.reference.binding == "onOpenJobs")
                    .unwrap()
                    .0
            })
            .unwrap();
        for _ in 0..2 {
            assert!(
                document
                    .dispatch_action(meari_native::domain::widgets::Action::Activate {
                        path: path.clone()
                    })
                    .unwrap()
            );
        }
        rig.tick();
        // A stalled Capacity batch would leave input_blocked and reject this action.
        assert!(
            document
                .dispatch_action(meari_native::domain::widgets::Action::Activate { path })
                .unwrap()
        );
        rig.tick();
        let current = document.stamp().unwrap();
        assert!(matches!(
            rig.host_mut().admit_callback(&current, &old),
            IntentAdmission::Rejected(_)
        ));
        let other = rig.pending("onRefreshReadProjection", None);
        let current = document.stamp().unwrap();
        assert!(matches!(
            rig.host_mut().admit_callback(&current, &other),
            IntentAdmission::Rejected(_)
        ));
        assert!(!rig.host().callbacks.borrow().has_jobs_entry());
        assert_eq!(rig.probe.calls.lock().unwrap().len(), calls);
        assert_eq!(rig.wake_requests(), wakes);
        assert_eq!(
            &rig.host()
                .jobs
                .borrow()
                .returning
                .as_ref()
                .unwrap()
                .committed
                .as_ref()
                .unwrap()
                .0,
            &original
        );
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; rearm emits receipt-free redraw and one original notifier request"]
fn explicit_rearm_requests_redraw_and_one_wake_without_recommit_or_idle_retry()
-> Result<(), Box<CaseFailure>> {
    with_rig(true, |rig| {
        rig.enter();
        rig.drain_wakes();
        rig.begin_return();
        let observer = rig.observer.as_ref().unwrap().clone();
        let busy = observer.borrow_mut();
        rig.tick();
        drop(busy);
        let original = rig
            .host()
            .jobs
            .borrow()
            .returning
            .as_ref()
            .unwrap()
            .committed
            .as_ref()
            .unwrap()
            .0
            .clone();
        let pending = rig.pending("onOpenJobs", None);
        rig.admit(&pending);
        let before = rig.wake_requests();
        let calls = rig.probe.calls.lock().unwrap().len();
        let turn = rig.turn();
        let effect = rig.host_mut().after_dispatch(turn).unwrap();
        assert!(effect.redraw);
        assert!(effect.committed.is_none());
        assert_eq!(rig.wake_requests(), before + 1);
        let turn = rig.turn();
        let idle = rig.host_mut().after_dispatch(turn).unwrap();
        assert!(!idle.redraw);
        assert!(idle.committed.is_none());
        assert_eq!(rig.wake_requests(), before + 1);
        assert_eq!(rig.probe.calls.lock().unwrap().len(), calls);
        assert_eq!(
            &rig.host()
                .jobs
                .borrow()
                .returning
                .as_ref()
                .unwrap()
                .committed
                .as_ref()
                .unwrap()
                .0,
            &original
        );
        rig.drain_wakes();
        assert_eq!(rig.wake_requests(), before + 1);
    })
}
