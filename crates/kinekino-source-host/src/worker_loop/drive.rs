use super::*;
use crate::{domain::Kind, worker_ipc::ControlEvent};
use std::{
    future::{Future, poll_fn},
    pin::pin,
    task::Poll,
};

impl WorkerLoop {
    pub async fn run(&mut self) -> LoopStatus {
        loop {
            if self.closing {
                return self.close().await;
            }
            let now = uptime_millis();
            if now >= self.current_cutoffs().operation {
                self.fail(LoopError::Cutoff, now);
                continue;
            }
            // At most one bounded control event per turn, before HTTP/data work.
            match self.ipc.poll_control() {
                Ok(ControlEvent::Pending) => {}
                Ok(ControlEvent::Bootstrap(request)) => {
                    if self.session.is_some() {
                        self.fail(LoopError::Order, now);
                        continue;
                    }
                    match WorkerSession::new(request, self.current_cutoffs()) {
                        Ok(session) => self.session = Some(session),
                        Err(error) => {
                            self.fail(LoopError::Worker(error), now);
                            continue;
                        }
                    }
                }
                Ok(ControlEvent::Frame(frame)) => {
                    let Some(session) = &mut self.session else {
                        self.fail(LoopError::Order, now);
                        continue;
                    };
                    if let Err(error) = session.control(&frame, now) {
                        self.fail(LoopError::Worker(error), now);
                        continue;
                    }
                    if frame.kind == Kind::Abort {
                        self.fail(
                            LoopError::Worker(WorkerError::Http(SendError::Cancelled)),
                            now,
                        );
                        continue;
                    }
                }
                Ok(ControlEvent::Eof) => {
                    self.fail(LoopError::PeerEof, now);
                    continue;
                }
                Err(error) => {
                    self.fail(LoopError::Ipc(error), now);
                    continue;
                }
            }
            if self.draining_failure {
                if let Some(session) = &mut self.session {
                    match self.ipc.poll_data_write(session) {
                        Err(error) => {
                            self.outcome.remember(LoopError::Ipc(error));
                            self.closing = true;
                        }
                        Ok(_) if session.failure_written() => self.closing = true,
                        Ok(_) => {}
                    }
                }
            } else if let Err(error) = self.progress_http(now).await {
                self.fail(error, now);
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }
    async fn progress_http(&mut self, now: u64) -> Result<(), LoopError> {
        let cutoffs = self.current_cutoffs();
        let Some(session) = &mut self.session else {
            return Ok(());
        };
        if session.needs_open() {
            let polled =
                poll_fn(|cx| Poll::Ready(session.poll_open(cx, now, || check_cutoff(cutoffs))))
                    .await;
            if let Poll::Ready(Err(error)) = polled {
                return Err(LoopError::Worker(error));
            }
        }
        if session.can_read() {
            // Poll a borrowing chunk future once. Response/body state remains in
            // session across Pending; no detached reader or fresh send is created.
            let result = poll_fn(|cx| {
                let mut read = pin!(session.read_body(|| check_cutoff(cutoffs)));
                Poll::Ready(read.as_mut().poll(cx))
            })
            .await;
            if let Poll::Ready(Err(error)) = result {
                return Err(LoopError::Worker(error));
            }
        }
        if session.can_pump() {
            session
                .pump_body(uptime_millis())
                .map_err(LoopError::Worker)?;
        }
        // Open may narrow the inherited cutoff. Use that same boundary around
        // IPC writes too; a late completion never becomes a successful body end.
        if uptime_millis() >= cutoffs.operation {
            return Err(LoopError::Cutoff);
        }
        self.ipc.poll_data_write(session).map_err(LoopError::Ipc)?;
        if uptime_millis() >= cutoffs.operation {
            return Err(LoopError::Cutoff);
        }
        if session.body_end_written() {
            self.closing = true;
        }
        Ok(())
    }
}
