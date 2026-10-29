use crate::{
    codec,
    domain::{Frame, HEADER_BYTES, ProtocolError},
};

/// No Debug: backing bytes may contain private source data.
pub struct FrameReceiver {
    bytes: Vec<u8>,
    expected: usize,
    header_done: bool,
    closed: bool,
    failure: Option<ProtocolError>,
    control_only: bool,
}
impl Default for FrameReceiver {
    fn default() -> Self {
        Self {
            bytes: Vec::new(),
            expected: HEADER_BYTES,
            header_done: false,
            closed: false,
            failure: None,
            control_only: false,
        }
    }
}
impl FrameReceiver {
    /// Parent control lane admits only Open/Credit/Abort, checked in the header
    /// before any body allocation or copy. General data receivers stay unchanged.
    pub fn control_only() -> Self {
        Self {
            control_only: true,
            ..Self::default()
        }
    }
    /// Consume at most one frame. Caller retains input[consumed..]. A header
    /// is checked before body reservation or copying. Failed receivers stay failed.
    pub fn push(&mut self, input: &[u8]) -> Result<(usize, Option<Frame>), ProtocolError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.closed {
            return Err(ProtocolError::Order);
        }
        let result = self.receive(input);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
    fn receive(&mut self, input: &[u8]) -> Result<(usize, Option<Frame>), ProtocolError> {
        let mut consumed = 0;
        loop {
            let count = (self.expected - self.bytes.len()).min(input.len() - consumed);
            self.bytes
                .try_reserve_exact(count)
                .map_err(|_| ProtocolError::Limit)?;
            self.bytes
                .extend_from_slice(&input[consumed..consumed + count]);
            consumed += count;
            if self.bytes.len() < self.expected {
                return Ok((consumed, None));
            }
            if !self.header_done {
                let (kind, length) = codec::header_fields(&self.bytes)?;
                if self.control_only
                    && !matches!(
                        kind,
                        crate::domain::Kind::Open
                            | crate::domain::Kind::Credit
                            | crate::domain::Kind::Abort
                    )
                {
                    return Err(ProtocolError::Order);
                }
                self.expected = HEADER_BYTES
                    .checked_add(length)
                    .ok_or(ProtocolError::Limit)?;
                self.header_done = true;
                continue;
            }
            let frame = codec::decode_exact(&self.bytes)?;
            self.bytes.clear();
            self.expected = HEADER_BYTES;
            self.header_done = false;
            return Ok((consumed, Some(frame)));
        }
    }
    /// Boundary EOF is a transport fact only. Partial header/body is truncation.
    pub fn eof(&mut self) -> Result<(), ProtocolError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        self.closed = true;
        if !self.bytes.is_empty() {
            self.failure = Some(ProtocolError::Truncated);
            return Err(ProtocolError::Truncated);
        }
        Ok(())
    }
}
