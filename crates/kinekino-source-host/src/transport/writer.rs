use crate::{
    codec,
    domain::{Frame, MAX_DATA_BYTES, ProtocolError},
};

/// Serialized bytes remain owned until every byte is reported written. The OS
/// adapter must separately retain its pending operation's buffer/OVERLAPPED.
pub struct FrameWriter {
    bytes: Vec<u8>,
    offset: usize,
    offered: usize,
    failure: Option<ProtocolError>,
}
impl FrameWriter {
    pub fn new(frame: &Frame) -> Result<Self, ProtocolError> {
        Ok(Self {
            bytes: codec::encode(frame)?,
            offset: 0,
            offered: 0,
            failure: None,
        })
    }
    /// Adapter copies this slice into its own pending-IO allocation before return.
    /// Each offer is single-use until the original write completion is reaped.
    pub fn offer(&mut self, maximum: usize) -> Result<Option<&[u8]>, ProtocolError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if maximum == 0 || maximum > MAX_DATA_BYTES {
            return Err(ProtocolError::Limit);
        }
        if self.offered != 0 {
            return Err(ProtocolError::Order);
        }
        if self.offset == self.bytes.len() {
            return Ok(None);
        }
        self.offered = (self.bytes.len() - self.offset).min(maximum);
        Ok(Some(&self.bytes[self.offset..self.offset + self.offered]))
    }
    /// Actual OS bytes written can be less than offered. A zero/oversized or
    /// unpaired completion poisons the writer without discarding owned bytes.
    pub fn complete(&mut self, count: usize) -> Result<(), ProtocolError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if count == 0 || count > self.offered {
            self.failure = Some(ProtocolError::Io);
            return Err(ProtocolError::Io);
        }
        self.offset += count;
        self.offered = 0;
        Ok(())
    }
    pub fn done(&self) -> bool {
        self.failure.is_none() && self.offset == self.bytes.len() && self.offered == 0
    }
}
