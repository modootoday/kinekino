use super::{BootstrapError, HttpBootstrap, MAX_REQUEST_BYTES, REQUEST_HEADER_BYTES, codec};

/// Bounded incremental framing only. The caller retains unread bytes and owns
/// deadlines, cancellation, binding admission and the physical IPC lifetime.
/// No Debug: the backing storage can contain sensitive request query values.
pub struct BootstrapReceiver {
    bytes: [u8; MAX_REQUEST_BYTES],
    used: usize,
    expected: usize,
    terminal: bool,
    failure: Option<BootstrapError>,
}
impl Default for BootstrapReceiver {
    fn default() -> Self {
        Self::new()
    }
}
impl BootstrapReceiver {
    pub fn new() -> Self {
        Self {
            bytes: [0; MAX_REQUEST_BYTES],
            used: 0,
            expected: REQUEST_HEADER_BYTES,
            terminal: false,
            failure: None,
        }
    }
    /// Consume at most one preamble. A coalesced following MSH1 frame remains
    /// in input[consumed..]. A completed or failed receiver cannot be reused.
    pub fn push(&mut self, input: &[u8]) -> Result<(usize, Option<HttpBootstrap>), BootstrapError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.terminal {
            return Err(BootstrapError::Framing);
        }
        match self.receive(input) {
            Ok(result) => Ok(result),
            Err(error) => {
                self.failure = Some(error);
                self.terminal = true;
                Err(error)
            }
        }
    }
    fn receive(&mut self, input: &[u8]) -> Result<(usize, Option<HttpBootstrap>), BootstrapError> {
        let mut consumed = 0;
        loop {
            let count = (self.expected - self.used).min(input.len() - consumed);
            self.bytes[self.used..self.used + count]
                .copy_from_slice(&input[consumed..consumed + count]);
            self.used += count;
            consumed += count;
            if self.used < self.expected {
                return Ok((consumed, None));
            }
            if self.expected == REQUEST_HEADER_BYTES {
                // No body is copied until its declared size has been admitted.
                self.expected = codec::declared_size(&self.bytes[..self.used])?;
                continue;
            }
            let request = HttpBootstrap::decode_exact(&self.bytes[..self.used])?;
            self.terminal = true;
            return Ok((consumed, Some(request)));
        }
    }
    /// Transport EOF before a full preamble is truncation, never BodyEnd.
    pub fn eof(&mut self) -> Result<(), BootstrapError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.terminal {
            return Ok(());
        }
        self.failure = Some(BootstrapError::Truncated);
        self.terminal = true;
        Err(BootstrapError::Truncated)
    }
}
