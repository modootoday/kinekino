//! Single worker transfer state; outer loop owns runtime, IPC and physical close.
mod failure;
#[cfg(test)]
mod tests;
use crate::{
    domain::{
        Cutoffs, Frame, Kind, ProtocolError,
        state::{BodyStatus, Direction, Exchange},
    },
    http_send::{HttpSendOwner, SendError},
    request::HttpBootstrap,
    transport::FrameWriter,
};
use std::task::{Context, Poll};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerError {
    Protocol(ProtocolError),
    Http(SendError),
    Order,
}
impl From<ProtocolError> for WorkerError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}
impl From<SendError> for WorkerError {
    fn from(error: SendError) -> Self {
        Self::Http(error)
    }
}
struct Outgoing {
    kind: Kind,
    writer: FrameWriter,
    data_bytes: usize,
}
/// No Debug/Clone. All HTTP owners and pending wire bytes remain here on failure.
/// Construction is inert and does not issue replay permission or launch a child.
pub struct WorkerSession {
    bootstrap: Option<HttpBootstrap>,
    binding: [u8; 32],
    exchange: Exchange,
    send: Option<HttpSendOwner>,
    opened: bool,
    next_child: u64,
    outgoing: Option<Outgoing>,
    failure: Option<WorkerError>,
}
impl WorkerSession {
    pub fn cutoffs(&self) -> Cutoffs {
        self.exchange.cutoffs()
    }
    pub fn needs_open(&self) -> bool {
        self.failure.is_none()
            && !self.opened
            && self.exchange.status() == BodyStatus::AwaitingMetadata
    }
    pub fn can_pump(&self) -> bool {
        self.failure.is_none()
            && self.outgoing.is_none()
            && self.exchange.status() == BodyStatus::Streaming
    }
    /// Release HTTP progress while retaining its original owner until outer close.
    /// A completed body is not changed into a synthetic cancellation error.
    pub fn close_http(&mut self) {
        if let Some(send) = &mut self.send {
            send.abort();
        }
    }
    /// Cleanup entry remains callable after a primary failure. Keep queued bytes
    /// until original IPC operations have been reaped by the outer owner.
    /// No wire/HTTP close acknowledgement is manufactured here.
    pub fn abort(&mut self) {
        self.failure
            .get_or_insert(WorkerError::Http(SendError::Cancelled));
        if let Some(send) = &mut self.send {
            send.abort();
        }
    }
    pub fn new(bootstrap: HttpBootstrap, cutoffs: Cutoffs) -> Result<Self, WorkerError> {
        let binding = *bootstrap.binding();
        Ok(Self {
            bootstrap: Some(bootstrap),
            binding,
            exchange: Exchange::new(binding, cutoffs)?,
            send: None,
            opened: false,
            next_child: 0,
            outgoing: None,
            failure: None,
        })
    }
    fn record<T>(&mut self, result: Result<T, WorkerError>) -> Result<T, WorkerError> {
        if let Err(error) = result {
            self.failure.get_or_insert(error);
        }
        result
    }
    fn healthy(&self) -> Result<(), WorkerError> {
        self.failure.map_or(Ok(()), Err)
    }
    /// Control lane remains independently callable while a data write is pending.
    /// Abort retains HTTP/IPC frame owners; outer close must reap original IO.
    pub fn control(&mut self, frame: &Frame, now: u64) -> Result<(), WorkerError> {
        self.healthy()?;
        let result = self
            .exchange
            .accept(Direction::Parent, frame, now)
            .map_err(Into::into);
        self.record(result)?;
        if frame.kind == Kind::Abort
            && let Some(send) = &mut self.send
        {
            send.abort();
        }
        Ok(())
    }
    /// Only after original Open and external replay admission. The one bootstrap
    /// is consumed before setup; a setup/send failure cannot prepare another GET.
    pub fn poll_open(
        &mut self,
        cx: &mut Context<'_>,
        now: u64,
        mut check: impl FnMut() -> Result<(), SendError>,
    ) -> Poll<Result<(), WorkerError>> {
        if let Err(error) = self.healthy() {
            return Poll::Ready(Err(error));
        }
        if self.opened || self.exchange.status() != BodyStatus::AwaitingMetadata {
            return Poll::Ready(Err(WorkerError::Order));
        }
        if self.send.is_none() {
            if let Err(error) = check() {
                let result = self.record::<()>(Err(error.into()));
                return Poll::Ready(result);
            }
            let Some(bootstrap) = self.bootstrap.take() else {
                return Poll::Ready(Err(WorkerError::Order));
            };
            match HttpSendOwner::prepare(bootstrap) {
                Ok(send) => self.send = Some(send),
                Err(error) => {
                    let result = self.record::<()>(Err(error.into()));
                    return Poll::Ready(result);
                }
            }
        }
        let result = self
            .send
            .as_mut()
            .expect("retained send")
            .poll_open(cx, &mut check);
        match result {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error)) => Poll::Ready(self.record(Err(error.into()))),
            Poll::Ready(Ok(())) => {
                self.opened = true;
                let result = self.metadata_frame(now);
                Poll::Ready(self.record(result))
            }
        }
    }
    fn response(&mut self) -> Result<&mut crate::http_response::HttpResponseOwner, WorkerError> {
        self.send
            .as_mut()
            .ok_or(WorkerError::Order)?
            .response_mut()
            .map_err(Into::into)
    }
    fn metadata_frame(&mut self, now: u64) -> Result<(), WorkerError> {
        let payload = self
            .response()?
            .wire_metadata()
            .map_err(|e| WorkerError::Http(SendError::Response(e)))?
            .encode()
            .map_err(|_| WorkerError::Protocol(ProtocolError::Framing))?;
        self.queue(Kind::Metadata, payload, now)
    }
    fn queue(&mut self, kind: Kind, payload: Vec<u8>, now: u64) -> Result<(), WorkerError> {
        if self.outgoing.is_some() {
            return Err(WorkerError::Order);
        }
        let frame = Frame {
            binding: self.binding,
            sequence: self.next_child,
            offset: if matches!(kind, Kind::Data | Kind::BodyEnd) {
                self.exchange.offset()
            } else {
                0
            },
            kind,
            payload,
        };
        // Reserve encoded bytes before advancing wire state/credit.
        let writer = FrameWriter::new(&frame)?;
        self.exchange.accept(Direction::Child, &frame, now)?;
        self.next_child = self.next_child.checked_add(1).ok_or(WorkerError::Order)?;
        self.outgoing = Some(Outgoing {
            kind,
            writer,
            data_bytes: if kind == Kind::Data {
                frame.payload.len()
            } else {
                0
            },
        });
        Ok(())
    }
    /// No HTTP read-ahead with credit0, pending wire frame, or retained body bytes.
    pub fn can_read(&mut self) -> bool {
        self.failure.is_none()
            && self.outgoing.is_none()
            && self.exchange.status() == BodyStatus::Streaming
            && self.exchange.credit() != 0
            && self.response().is_ok_and(|r| r.can_read())
    }
    /// Caller may select this borrowed future against independent control IO.
    /// On cancellation it must process Abort and close the same session/IPC owner.
    pub async fn read_body(
        &mut self,
        mut check: impl FnMut() -> Result<(), SendError>,
    ) -> Result<(), WorkerError> {
        self.healthy()?;
        if !self.can_read() {
            return Err(WorkerError::Order);
        }
        let result = async {
            check()?;
            let read = self
                .response()?
                .read_next()
                .await
                .map_err(|e| WorkerError::Http(SendError::Response(e)));
            // Preserve actual read error as primary; response was already retained.
            read?;
            check()?;
            Ok(())
        }
        .await;
        self.record(result)
    }
    /// Form at most one bounded Data/BodyEnd. Actual EOF is required for BodyEnd.
    pub fn pump_body(&mut self, now: u64) -> Result<bool, WorkerError> {
        self.healthy()?;
        if self.outgoing.is_some() || self.exchange.status() != BodyStatus::Streaming {
            return Err(WorkerError::Order);
        }
        let credit = self.exchange.credit();
        let result = (|| {
            if self.response()?.body_ended() {
                self.queue(Kind::BodyEnd, Vec::new(), now)?;
                return Ok(true);
            }
            let bytes = self
                .response()?
                .offer(credit)
                .map_err(|e| WorkerError::Http(SendError::Response(e)))?;
            let Some(bytes) = bytes else {
                return Ok(false);
            };
            self.queue(Kind::Data, bytes.to_vec(), now)?;
            Ok(true)
        })();
        self.record(result)
    }
    pub fn offer_write(&mut self, maximum: usize) -> Result<Option<&[u8]>, WorkerError> {
        self.writable()?;
        if self.exchange.status() == BodyStatus::Aborted {
            return Err(WorkerError::Order);
        }
        let Some(outgoing) = &mut self.outgoing else {
            return Ok(None);
        };
        outgoing.writer.offer(maximum).map_err(Into::into)
    }
    pub fn complete_write(&mut self, count: usize) -> Result<(), WorkerError> {
        self.writable()?;
        let result = (|| {
            let outgoing = self.outgoing.as_mut().ok_or(WorkerError::Order)?;
            outgoing.writer.complete(count)?;
            if outgoing.writer.done() {
                let bytes = outgoing.data_bytes;
                if bytes != 0 {
                    self.response()?
                        .complete(bytes)
                        .map_err(|e| WorkerError::Http(SendError::Response(e)))?;
                }
                self.outgoing = None;
            }
            Ok(())
        })();
        self.record(result)
    }
    /// Wire completion only, never child exit/Job empty/diagnostic EOF/receipt.
    pub fn body_end_written(&self) -> bool {
        self.failure.is_none()
            && self.exchange.status() == BodyStatus::Ended
            && self.outgoing.is_none()
    }
}
