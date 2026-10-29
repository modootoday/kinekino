//! One owned upstream response chunk. No HTTP, publication or physical ACK.
use crate::domain::{MAX_DATA_BYTES, ProtocolError};
use bytes::Bytes;

/// Candidate upstream frame bound; not a TLS/OS/process RAM guarantee.
pub const MAX_UPSTREAM_BYTES: usize = 1024 * 1024;

/// No Debug: response data may be private. A returned Bytes slice retains its
/// allocation even if the owner is dropped during cancellation/partial IPC IO.
#[derive(Default)]
pub struct WorkerBuffer {
    pending: Option<Bytes>,
    offset: usize,
    offered: usize,
}
impl WorkerBuffer {
    pub fn can_poll_upstream(&self) -> bool {
        self.pending.is_none()
    }
    /// Rejected input is returned to its caller; an existing owner is unchanged.
    pub fn install(&mut self, chunk: Bytes) -> Result<(), (ProtocolError, Bytes)> {
        if self.pending.is_some() {
            return Err((ProtocolError::Order, chunk));
        }
        if chunk.is_empty() || chunk.len() > MAX_UPSTREAM_BYTES {
            return Err((ProtocolError::Limit, chunk));
        }
        self.pending = Some(chunk);
        self.offset = 0;
        Ok(())
    }
    /// Credit is supplied by the original Exchange. Repeated offers before a
    /// completion are refused; only the original IPC completion may advance it.
    pub fn offer(&mut self, credit: u64) -> Result<Option<Bytes>, ProtocolError> {
        if self.offered != 0 {
            return Err(ProtocolError::Order);
        }
        let Some(chunk) = &self.pending else {
            return Ok(None);
        };
        let count = (chunk.len() - self.offset)
            .min(MAX_DATA_BYTES)
            .min(usize::try_from(credit).unwrap_or(usize::MAX));
        if count == 0 {
            return Ok(None);
        }
        self.offered = count;
        Ok(Some(chunk.slice(self.offset..self.offset + count)))
    }
    /// A complete frame, not each underlying partial WriteFile, consumes data.
    /// Error leaves both the owned chunk and outstanding offer unchanged.
    pub fn complete(&mut self, bytes: usize) -> Result<(), ProtocolError> {
        if self.offered == 0 || bytes != self.offered {
            return Err(ProtocolError::Order);
        }
        self.offset += bytes;
        self.offered = 0;
        if self
            .pending
            .as_ref()
            .is_some_and(|chunk| self.offset == chunk.len())
        {
            self.pending = None;
            self.offset = 0;
        }
        Ok(())
    }
}
