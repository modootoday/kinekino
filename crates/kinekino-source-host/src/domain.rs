pub const HEADER_BYTES: usize = 60;
pub const MAX_DATA_BYTES: usize = 64 * 1024;
pub const MAX_METADATA_BYTES: usize = 32 * 1024;
pub const MAX_FAILURE_BYTES: usize = 4096;
pub const MAX_CREDIT_BYTES: u64 = 256 * 1024;
pub const MAGIC: [u8; 4] = *b"MSH1";
pub const VERSION: u16 = 1;
pub mod state;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    Framing,
    Limit,
    Binding,
    Order,
    Credit,
    Cutoff,
    Truncated,
    Io,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Open = 1,
    Metadata = 2,
    Data = 3,
    Credit = 4,
    Abort = 5,
    BodyEnd = 6,
    Failure = 7,
}
impl Kind {
    pub fn from_byte(value: u8) -> Result<Self, ProtocolError> {
        match value {
            1 => Ok(Self::Open),
            2 => Ok(Self::Metadata),
            3 => Ok(Self::Data),
            4 => Ok(Self::Credit),
            5 => Ok(Self::Abort),
            6 => Ok(Self::BodyEnd),
            7 => Ok(Self::Failure),
            _ => Err(ProtocolError::Framing),
        }
    }
    pub fn check_length(self, length: usize) -> Result<(), ProtocolError> {
        let valid = match self {
            Self::Open => length == 16,
            Self::Credit => length == 8,
            Self::Abort | Self::BodyEnd => length == 0,
            Self::Metadata => (1..=MAX_METADATA_BYTES).contains(&length),
            Self::Data => (1..=MAX_DATA_BYTES).contains(&length),
            Self::Failure => (1..=MAX_FAILURE_BYTES).contains(&length),
        };
        if valid {
            Ok(())
        } else {
            Err(ProtocolError::Limit)
        }
    }
}

/// This value identifies a stream exchange; possession grants no request authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub binding: [u8; 32],
    pub sequence: u64,
    pub offset: u64,
    pub kind: Kind,
    pub payload: Vec<u8>,
}

/// Integer values in an externally established common monotonic clock domain.
/// This module neither reads a clock nor translates a process-local Instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cutoffs {
    pub operation: u64,
    pub end: u64,
}
impl Cutoffs {
    pub fn new(operation: u64, end: u64) -> Result<Self, ProtocolError> {
        if operation == 0 || operation >= end {
            Err(ProtocolError::Cutoff)
        } else {
            Ok(Self { operation, end })
        }
    }
    pub fn constrained_by(self, requested: Self) -> Result<Self, ProtocolError> {
        Self::new(
            self.operation.min(requested.operation),
            self.end.min(requested.end),
        )
    }
}
