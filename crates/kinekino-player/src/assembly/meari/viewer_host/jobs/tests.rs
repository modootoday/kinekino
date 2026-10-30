//! Original get-document/SDK handoff fixture only; not default JOB route or rendered-frame proof.
use super::handoff::*;
use crate::{
    application::{
        api::{ApiError, PlayerSdk, RequestContext},
        jobs::GetJob,
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
        },
    },
    assembly::meari::{
        job_details::document::{JobDetailsMountFailure, MountedJobDetails},
        player_shell_document::DocumentOptions,
    },
};
use meari::{application::host_callbacks::PendingCallback, domain::Value};
use meari_native::{
    domain::{Viewport, widgets::Action},
    infrastructure::native_host::{HostCommitReceipt, NativeDocumentHandle},
};
use std::{
    any::Any,
    collections::BTreeMap,
    io::Write,
    num::NonZeroU64,
    sync::Arc,
    time::{Duration, Instant},
};

struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Fixture {
    execution: ExecutionOwner,
    panel: Option<MountedJobDetails>,
    mount_failure: Option<JobDetailsMountFailure>,
    handoff: Option<QueryHandoff>,
    committed: Option<CommittedHandoff>,
    props: BTreeMap<String, Value>,
    old_callback: Option<PendingCallback>,
    earlier_receipt: Option<HostCommitReceipt>,
}
impl Fixture {
    fn document(&self) -> NativeDocumentHandle {
        self.panel.as_ref().unwrap().document.clone()
    }
    fn mount(&mut self) -> Result<(), String> {
        let attachment = self
            .execution
            .handle()
            .attach()
            .map_err(|error| error.message)?;
        match MountedJobDetails::mount(
            attachment,
            GetJob {
                job_id: format!("job-{}", "a".repeat(32)).try_into().unwrap(),
            },
            RequestContext::default(),
            DocumentOptions {
                namespace: "jobs-handoff-fixture".into(),
                epoch: 1,
                source_generation: 1,
                viewport: Viewport {
                    width: 320,
                    height: 720,
                    dark: true,
                },
            },
            "",
            None,
        ) {
            Ok(panel) => {
                self.props = panel.owner.borrow().as_ref().unwrap().view.props();
                self.panel = Some(panel);
                Ok(())
            }
            Err(failure) => {
                let message = failure.diagnostic.clone();
                self.mount_failure = Some(failure);
                Err(message)
            }
        }
    }
    fn begin(&mut self) -> Result<(), String> {
        let document = self.document();
        let path = document
            .try_with(|document| {
                document
                    .module_projection()
                    .unwrap()
                    .callback_bindings()
                    .into_iter()
                    .find(|(_, value)| value.reference.binding == "onRefreshJob")
                    .unwrap()
                    .0
            })
            .map_err(|error| format!("{error:?}"))?;
        document
            .dispatch_action(Action::Activate { path })
            .map_err(|error| format!("{error:?}"))?;
        let (mut pending, receipt) = document
            .try_reproject(
                &document.stamp().map_err(|error| format!("{error:?}"))?,
                |document| Ok::<_, String>(document.drain_callbacks()),
            )
            .map_err(|error| format!("{error:?}"))?;
        if pending.len() != 1 {
            return Err("Expected original get callback".into());
        }
        self.old_callback = pending.pop();
        self.earlier_receipt = Some(receipt);
        let panel = self.panel.as_ref().unwrap();
        let scope = document.stamp().map_err(|error| format!("{error:?}"))?;
        {
            let mut owner = panel.owner.borrow_mut();
            owner
                .as_mut()
                .unwrap()
                .dispatch(self.old_callback.as_ref().unwrap(), &scope.owner)
                .map_err(|error| error.message)?;
            assert!(owner.as_ref().unwrap().pending().is_some());
        }
        let original = panel.owner.borrow_mut().take().unwrap();
        // No fallible work between moving the original owner and storing its new carrier.
        self.handoff = Some(QueryHandoff::new(
            self.execution.handle(),
            RequestContext::default(),
            scope,
            NonZeroU64::new(1).unwrap(),
            QueryTarget::List,
            Some(ReadyConsumer {
                owner: QueryOwner::Detail(Box::new(original)),
                keys: BTreeMap::new(),
            }),
        ));
        let end = Instant::now() + Duration::from_secs(3);
        while !self
            .handoff
            .as_mut()
            .unwrap()
            .prepare()
            .map_err(|error| error.message)?
        {
            if Instant::now() >= end {
                return Err("Original handoff remains pending".into());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
    fn commit(&mut self) -> Result<(), String> {
        let document = self.document();
        let current = document.stamp().map_err(|error| format!("{error:?}"))?;
        let handoff = self.handoff.as_mut().unwrap();
        handoff
            .stage_commit(&current)
            .map_err(|error| error.message)?;
        let bindings = handoff
            .prepared()
            .unwrap()
            .owner
            .bindings()
            .map_err(|error| error.message)?;
        let (_, receipt) = document
            .try_reproject(&current, |document| {
                document.update_module_props_with_callbacks(self.props.clone(), bindings)
            })
            .map_err(|error| format!("{error:?}"))?;
        self.committed = Some(
            handoff
                .commit(
                    receipt,
                    &document.stamp().map_err(|error| format!("{error:?}"))?,
                )
                .map_err(|error| error.message)?,
        );
        Ok(())
    }
    fn cleanup(&mut self) -> Result<(), String> {
        let end = Instant::now() + Duration::from_secs(3);
        loop {
            let mut complete = true;
            if let Some(handoff) = &mut self.handoff {
                complete &= handoff.close().map_err(|error| error.message)?;
            }
            if let Some(committed) = &mut self.committed {
                complete &= committed
                    .current
                    .owner
                    .close()
                    .map_err(|error| error.message)?;
                if let Some(previous) = &mut committed.previous {
                    complete &= previous.owner.close().map_err(|error| error.message)?;
                }
            }
            if let Some(panel) = &self.panel
                && panel.owner.borrow().is_some()
            {
                complete &= panel.close_query_owner().map_err(|error| error.message)?;
            }
            if let Some(failure) = &mut self.mount_failure {
                if let Some(owner) = failure.owner.borrow_mut().as_mut() {
                    complete &= owner.close().map_err(|error| error.message)?;
                }
                if complete {
                    failure
                        .attachment
                        .release()
                        .map_err(|error| error.message)?;
                }
            }
            if complete {
                break;
            }
            if Instant::now() >= end {
                return Err("Original query owner cleanup remains".into());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        loop {
            match self.execution.poll_close().map_err(|error| error.message)? {
                ExecutionCloseProgress::Closed(receipt)
                    if receipt.worker_stopped && receipt.unacknowledged == 0 =>
                {
                    return Ok(());
                }
                ExecutionCloseProgress::Closed(receipt) => {
                    return Err(format!("Incomplete original SDK: {receipt:?}"));
                }
                _ if Instant::now() < end => std::thread::sleep(Duration::from_millis(1)),
                _ => return Err("Original SDK worker close remains".into()),
            }
        }
    }
}
struct Failure {
    primary: Option<Box<dyn Any + Send>>,
    setup: Option<String>,
    setup_panic: Option<Box<dyn Any + Send>>,
    cleanup: Option<String>,
    cleanup_panic: Option<Box<dyn Any + Send>>,
    diagnostic: Option<std::io::Error>,
    _fixture: Fixture,
}
impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalJobsHandoffFailure")
            .field("primary_panic", &self.primary.is_some())
            .field("setup", &self.setup)
            .field("setup_panic", &self.setup_panic.is_some())
            .field("cleanup", &self.cleanup)
            .field("cleanup_panic", &self.cleanup_panic.is_some())
            .field("diagnostic", &self.diagnostic)
            .finish()
    }
}
fn with_fixture(body: impl FnOnce(&mut Fixture)) -> Result<(), Box<Failure>> {
    let execution = ExecutionOwner::start(
        Arc::new(PlayerSdk::default()),
        ExecutionBudget {
            max_operations: 2,
            max_request_bytes: 262144,
            max_total_request_bytes: 524288,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 524288,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let mut fixture = Fixture {
        execution,
        panel: None,
        mount_failure: None,
        handoff: None,
        committed: None,
        props: BTreeMap::new(),
        old_callback: None,
        earlier_receipt: None,
    };
    let (setup, setup_panic) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fixture.mount())) {
            Ok(result) => (result.err(), None),
            Err(panic) => (None, Some(panic)),
        };
    let primary = if setup.is_none() && setup_panic.is_none() {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(&mut fixture))).err()
    } else {
        None
    };
    let (cleanup, cleanup_panic) =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fixture.cleanup())) {
            Ok(result) => (result.err(), None),
            Err(panic) => (None, Some(panic)),
        };
    let mut retained = Box::new(Failure {
        primary,
        setup,
        setup_panic,
        cleanup,
        cleanup_panic,
        diagnostic: None,
        _fixture: fixture,
    });
    let mut output = std::io::stderr().lock();
    retained.diagnostic = writeln!(output, "jobs_handoff_case_closeout complete={} primary_panic={} setup_panic={} cleanup_panic={} setup_error={:?} cleanup_error={:?}",
        retained.cleanup.is_none() && retained.cleanup_panic.is_none(), retained.primary.is_some(), retained.setup_panic.is_some(), retained.cleanup_panic.is_some(), retained.setup, retained.cleanup)
        .and_then(|()| output.flush()).err();
    if retained.primary.is_some()
        || retained.setup.is_some()
        || retained.setup_panic.is_some()
        || retained.cleanup.is_some()
        || retained.cleanup_panic.is_some()
        || retained.diagnostic.is_some()
    {
        return Err(retained);
    }
    Ok(())
}

#[test]
fn original_pending_get_drains_before_same_worker_list_attachment_and_old_callback_stays_retired()
-> Result<(), Box<Failure>> {
    with_fixture(|fixture| {
        fixture.begin().unwrap();
        assert!(fixture.execution.handle().attach().is_err());
        fixture.commit().unwrap();
        let committed = fixture.committed.as_ref().unwrap();
        let current = fixture.document().stamp().unwrap();
        assert!(
            committed
                .current
                .owner
                .local(fixture.old_callback.as_ref().unwrap(), &current.owner)
                .is_err()
        );
        match &committed.previous.as_ref().unwrap().owner {
            QueryOwner::Detail(owner) => {
                assert!(owner.pending().is_none());
                assert!(owner.retained().is_some());
            }
            QueryOwner::List(_) => panic!("Original get owner was replaced"),
        }
        assert!(fixture.execution.handle().attach().is_err());
    })
}
#[test]
fn stale_receipt_and_failed_callback_preflight_keep_prepared_owner_and_last_good_document()
-> Result<(), Box<Failure>> {
    with_fixture(|fixture| {
        fixture.begin().unwrap();
        let document = fixture.document();
        let current = document.stamp().unwrap();
        let handoff = fixture.handoff.as_mut().unwrap();
        handoff.stage_commit(&current).unwrap();
        assert!(
            handoff
                .commit(fixture.earlier_receipt.as_ref().unwrap().clone(), &current)
                .is_err()
        );
        assert!(handoff.prepared().is_some());
        let mut invalid = fixture.props.clone();
        invalid.insert(
            "onRefreshJob".into(),
            Value::CallbackReference(meari::domain::host_callbacks::CallbackReference {
                binding: "unexportedJobsHandoffAction".into(),
                signature: meari::domain::host_callbacks::CallbackSignature::Action,
            }),
        );
        let bindings = handoff.prepared().unwrap().owner.bindings().unwrap();
        assert!(
            document
                .try_reproject(&current, |document| document
                    .update_module_props_with_callbacks(invalid, bindings))
                .is_err()
        );
        assert_eq!(document.stamp().unwrap(), current);
        assert!(handoff.prepared().is_some());
        assert!(fixture.execution.handle().attach().is_err());
        fixture.commit().unwrap();
    })
}
