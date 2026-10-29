//! Parent wire receiver only; original replay/current/physical owner stays external.
mod incoming;
use crate::{
    domain::{
        Cutoffs, Frame, Kind, MAX_CREDIT_BYTES, ProtocolError,
        state::{BodyStatus, Direction, Exchange},
    },
    http_metadata::{HttpMetadata, MetadataError},
    request::HttpBootstrap,
};
use std::collections::VecDeque;
pub const MAX_PENDING_CHUNKS: usize = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParentError {
    Protocol(ProtocolError),
    Metadata(MetadataError),
    Limit,
    Order,
    WorkerFailed,
    Length,
}
impl From<ProtocolError> for ParentError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadStatus {
    Pending,
    Data(usize),
    End,
}
struct Chunk {
    bytes: Vec<u8>,
    offset: usize,
}
/// No Debug: original URL, metadata and pending media bytes can be private.
pub struct ParentStream {
    exchange: Exchange,
    binding: [u8; 32],
    expected_url: url::Url,
    maximum: u64,
    metadata: Option<HttpMetadata>,
    chunks: VecDeque<Chunk>,
    queued_bytes: u64,
    next_parent: u64,
    first: Option<ParentError>,
}
impl ParentStream {
    pub fn cutoffs(&self) -> Cutoffs {
        self.exchange.cutoffs()
    }
    /// Descriptive wire binding, never permission to launch/replay a request.
    pub fn new(request: &HttpBootstrap, cutoffs: Cutoffs) -> Result<Self, ParentError> {
        Ok(Self {
            exchange: Exchange::new(*request.binding(), cutoffs)?,
            binding: *request.binding(),
            expected_url: request.target().clone(),
            maximum: request.maximum_bytes(),
            metadata: None,
            chunks: VecDeque::new(),
            queued_bytes: 0,
            next_parent: 0,
            first: None,
        })
    }
    fn healthy(&self) -> Result<(), ParentError> {
        self.first.map_or(Ok(()), Err)
    }
    pub fn first_error(&self) -> Option<ParentError> {
        self.first
    }
    pub fn metadata(&self) -> Option<&HttpMetadata> {
        self.metadata.as_ref()
    }
    fn parent_frame(
        &mut self,
        kind: Kind,
        payload: Vec<u8>,
        now: u64,
    ) -> Result<Frame, ParentError> {
        self.healthy()?;
        let frame = Frame {
            binding: self.binding,
            sequence: self.next_parent,
            offset: 0,
            kind,
            payload,
        };
        let result = self.exchange.accept(Direction::Parent, &frame, now);
        if let Err(error) = result {
            let error = error.into();
            self.first.get_or_insert(error);
            return Err(error);
        }
        self.next_parent = self.next_parent.checked_add(1).ok_or(ParentError::Order)?;
        Ok(frame)
    }
    pub fn open_frame(&mut self, now: u64) -> Result<Frame, ParentError> {
        let cutoffs = self.exchange.cutoffs();
        let mut payload = cutoffs.operation.to_le_bytes().to_vec();
        payload.extend_from_slice(&cutoffs.end.to_le_bytes());
        self.parent_frame(Kind::Open, payload, now)
    }
    pub fn abort_frame(&mut self, now: u64) -> Result<Frame, ParentError> {
        self.parent_frame(Kind::Abort, Vec::new(), now)
    }
    /// Outstanding byte credit plus retained bytes never exceeds256KiB. Do not
    /// grant more when all16 chunk slots are occupied by partially consumed data.
    pub fn available_credit(&self) -> u64 {
        if self.first.is_some()
            || self.metadata.is_none()
            || self.exchange.status() != BodyStatus::Streaming
            || self.chunks.len() >= MAX_PENDING_CHUNKS
        {
            return 0;
        }
        MAX_CREDIT_BYTES.saturating_sub(self.queued_bytes + self.exchange.credit())
    }
    pub fn credit_frame(&mut self, amount: u64, now: u64) -> Result<Frame, ParentError> {
        self.healthy()?;
        if amount == 0 || amount > self.available_credit() {
            return Err(ParentError::Limit);
        }
        self.parent_frame(Kind::Credit, amount.to_le_bytes().to_vec(), now)
    }
    /// Stop physical data reads when full; control/abort/close stay independent.
    pub fn can_receive(&self) -> bool {
        self.first.is_none()
            && self.chunks.len() < MAX_PENDING_CHUNKS
            && self.queued_bytes < MAX_CREDIT_BYTES
            && !matches!(
                self.exchange.status(),
                BodyStatus::Ended | BodyStatus::Failed | BodyStatus::Aborted
            )
    }
    /// Pending is not EOF. Only an accepted, length-checked BodyEnd and drained
    /// queue produce End; a pipe EOF/crash must instead call transport_eof.
    pub fn read(&mut self, output: &mut [u8]) -> Result<ReadStatus, ParentError> {
        self.healthy()?;
        if output.is_empty() {
            return Err(ParentError::Order);
        }
        let mut count = 0;
        while let Some(chunk) = self.chunks.front_mut() {
            let take = (chunk.bytes.len() - chunk.offset).min(output.len() - count);
            output[count..count + take]
                .copy_from_slice(&chunk.bytes[chunk.offset..chunk.offset + take]);
            count += take;
            chunk.offset += take;
            self.queued_bytes -= take as u64;
            if chunk.offset == chunk.bytes.len() {
                self.chunks.pop_front();
            }
            if count == output.len() {
                break;
            }
        }
        if count != 0 {
            return Ok(ReadStatus::Data(count));
        }
        if self.exchange.status() == BodyStatus::Ended {
            Ok(ReadStatus::End)
        } else {
            Ok(ReadStatus::Pending)
        }
    }
    pub fn transport_eof(&mut self) -> Result<(), ParentError> {
        self.healthy()?;
        if self.exchange.status() == BodyStatus::Ended {
            return Ok(());
        }
        let error = ParentError::Protocol(ProtocolError::Truncated);
        self.first = Some(error);
        Err(error)
    }
}
