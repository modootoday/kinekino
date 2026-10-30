use super::*;

struct Observation {
    unacknowledged: usize,
    loop_finished: bool,
    joined: bool,
    fault: Option<Arc<ApiError>>,
    join_failure: Option<Arc<ApiError>>,
}
impl Observation {
    fn worker(&self) -> WorkerCloseState {
        if self.joined {
            WorkerCloseState::Joined
        } else {
            WorkerCloseState::ExitPending {
                loop_finished: self.loop_finished,
            }
        }
    }
    fn progress(self) -> ExecutionCloseProgress {
        let worker = self.worker();
        if let Some(error) = self.join_failure {
            return ExecutionCloseProgress::Faulted {
                kind: ExecutionCloseFaultKind::WorkerJoin,
                error,
                unacknowledged: self.unacknowledged,
                worker,
            };
        }
        if let Some(error) = self.fault
            && (!self.joined || self.unacknowledged != 0)
        {
            return ExecutionCloseProgress::Faulted {
                kind: ExecutionCloseFaultKind::Terminal,
                error,
                unacknowledged: self.unacknowledged,
                worker,
            };
        }
        if self.unacknowledged != 0 {
            ExecutionCloseProgress::TerminalAckPending {
                unacknowledged: self.unacknowledged,
                worker,
            }
        } else if !self.joined {
            ExecutionCloseProgress::WorkerExitPending {
                loop_finished: self.loop_finished,
            }
        } else {
            ExecutionCloseProgress::Closed(ExecutionCloseReceipt {
                worker_stopped: true,
                unacknowledged: 0,
            })
        }
    }
    fn legacy(self) -> Result<ExecutionCloseReceipt, ApiError> {
        if let Some(error) = self.join_failure {
            return Err(ApiError {
                code: error.code,
                message: error.message.clone(),
                retryable: error.retryable,
                details: error.details.clone(),
                remediation: error.remediation.clone(),
                request_id: error.request_id.clone(),
            });
        }
        if !self.joined || self.unacknowledged != 0 {
            let mut error = pending();
            error.details = serde_json::json!({"worker_stopped":self.joined,"worker_returned":self.loop_finished,"unacknowledged":self.unacknowledged,"closing":true});
            return Err(error);
        }
        Ok(ExecutionCloseReceipt {
            worker_stopped: true,
            unacknowledged: 0,
        })
    }
}
impl ExecutionHandle {
    /// Initiates closing; never waits for an unfinished thread or acknowledges an operation.
    /// Hosts must arrange a future bounded retry for WorkerExitPending; a wake is not join ACK.
    pub fn poll_close(&self) -> Result<ExecutionCloseProgress, ApiError> {
        self.poll_close_for(None)
    }
    pub(super) fn poll_close_for(
        &self,
        authority: Option<&Arc<()>>,
    ) -> Result<ExecutionCloseProgress, ApiError> {
        Ok(self.observe_close(authority, false)?.progress())
    }
    pub fn close(&self) -> Result<ExecutionCloseReceipt, ApiError> {
        self.close_for(None, false)
    }
    pub(super) fn close_for(
        &self,
        authority: Option<&Arc<()>>,
        owner: bool,
    ) -> Result<ExecutionCloseReceipt, ApiError> {
        self.observe_close(authority, owner)?.legacy()
    }
    fn observe_close(
        &self,
        authority: Option<&Arc<()>>,
        owner: bool,
    ) -> Result<Observation, ApiError> {
        let notify = {
            let mut state = self.shared.lock()?;
            if !owner {
                state.authorize(authority)?;
            }
            state.closing = true;
            control::cancel_queued(&self.shared, &mut state)
        };
        self.shared.changed.notify_all();
        if notify {
            self.shared.notify();
        }
        let mut worker = self
            .shared
            .worker
            .lock()
            .map_err(|_| internal("SDK worker ownership is unavailable"))?;
        if worker.as_ref().is_some_and(|handle| handle.is_finished()) {
            #[cfg(test)]
            self.shared
                .join_attempts
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let result = worker.take().ok_or_else(pending)?.join();
            let mut state = self.shared.lock()?;
            if result.is_ok() {
                state.worker_joined = true;
            } else {
                state.join_failure.get_or_insert_with(|| {
                    Arc::new(internal(
                        "SDK worker stopped unexpectedly; inspect retained operations",
                    ))
                });
            }
        }
        let mut state = self.shared.lock()?;
        if state.faulted && state.close_fault.is_none() {
            let error = state.entries.values().find_map(|entry| {
                let state::Phase::Terminal(terminal) = &entry.phase else { return None; };
                terminal.validate_response().err().or_else(|| terminal.measurement.is_faulted().then(|| {
                    let mut error = internal("SDK terminal encoding failed or exceeded its reservation; original response retained");
                    error.details = serde_json::json!({"execution_may_have_occurred":true,"measurement":format!("{:?}",terminal.measurement)});
                    error
                }))
            }).unwrap_or_else(|| internal("SDK execution faulted; inspect retained operations"));
            state.close_fault = Some(Arc::new(error));
        }
        Ok(Observation {
            unacknowledged: state.entries.len(),
            loop_finished: state.worker_finished,
            joined: state.worker_joined,
            fault: state.close_fault.clone(),
            join_failure: state.join_failure.clone(),
        })
    }
}
