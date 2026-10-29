use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Parent,
    Child,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyStatus {
    AwaitingOpen,
    AwaitingMetadata,
    Streaming,
    Ended,
    Failed,
    Aborted,
}

/// Wire state only. No variant asserts process closure or issues browser authority.
#[derive(Clone)]
pub struct Exchange {
    binding: [u8; 32],
    cutoffs: Cutoffs,
    next_parent: u64,
    next_child: u64,
    offset: u64,
    credit: u64,
    last_tick: u64,
    status: BodyStatus,
    poisoned: bool,
}
impl Exchange {
    pub fn new(binding: [u8; 32], cutoffs: Cutoffs) -> Result<Self, ProtocolError> {
        Cutoffs::new(cutoffs.operation, cutoffs.end)?;
        Ok(Self {
            binding,
            cutoffs,
            next_parent: 0,
            next_child: 0,
            offset: 0,
            credit: 0,
            last_tick: 0,
            status: BodyStatus::AwaitingOpen,
            poisoned: false,
        })
    }
    pub fn status(&self) -> BodyStatus {
        self.status
    }
    pub fn cutoffs(&self) -> Cutoffs {
        self.cutoffs
    }
    pub fn credit(&self) -> u64 {
        self.credit
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn accept(
        &mut self,
        direction: Direction,
        frame: &Frame,
        now: u64,
    ) -> Result<(), ProtocolError> {
        if self.poisoned {
            return Err(ProtocolError::Order);
        }
        let mut candidate = self.clone();
        match candidate.apply(direction, frame, now) {
            Ok(()) => {
                *self = candidate;
                Ok(())
            }
            Err(error) => {
                self.poisoned = true;
                Err(error)
            }
        }
    }
    fn apply(
        &mut self,
        direction: Direction,
        frame: &Frame,
        now: u64,
    ) -> Result<(), ProtocolError> {
        crate::codec::validate(frame)?;
        if frame.binding != self.binding {
            return Err(ProtocolError::Binding);
        }
        if now < self.last_tick {
            return Err(ProtocolError::Cutoff);
        }
        self.last_tick = now;
        if matches!(
            self.status,
            BodyStatus::Ended | BodyStatus::Failed | BodyStatus::Aborted
        ) {
            return Err(ProtocolError::Order);
        }
        let next = match direction {
            Direction::Parent => &mut self.next_parent,
            Direction::Child => &mut self.next_child,
        };
        if frame.sequence != *next {
            return Err(ProtocolError::Order);
        }
        *next = next.checked_add(1).ok_or(ProtocolError::Order)?;
        if !matches!(frame.kind, Kind::Abort | Kind::Failure) && now >= self.cutoffs.operation {
            return Err(ProtocolError::Cutoff);
        }
        if !matches!(frame.kind, Kind::Data | Kind::BodyEnd) && frame.offset != 0 {
            return Err(ProtocolError::Order);
        }
        match (direction, frame.kind) {
            (Direction::Parent, Kind::Open) if self.status == BodyStatus::AwaitingOpen => {
                let operation = u64::from_le_bytes(
                    frame.payload[..8]
                        .try_into()
                        .map_err(|_| ProtocolError::Framing)?,
                );
                let end = u64::from_le_bytes(
                    frame.payload[8..]
                        .try_into()
                        .map_err(|_| ProtocolError::Framing)?,
                );
                let requested = Cutoffs::new(operation, end)?;
                if operation > self.cutoffs.operation || end > self.cutoffs.end || now >= operation
                {
                    return Err(ProtocolError::Cutoff);
                }
                self.cutoffs = requested;
                self.status = BodyStatus::AwaitingMetadata;
            }
            (Direction::Child, Kind::Metadata) if self.status == BodyStatus::AwaitingMetadata => {
                self.status = BodyStatus::Streaming
            }
            (Direction::Parent, Kind::Credit) if self.status == BodyStatus::Streaming => {
                let amount = u64::from_le_bytes(
                    frame
                        .payload
                        .as_slice()
                        .try_into()
                        .map_err(|_| ProtocolError::Framing)?,
                );
                if amount == 0 {
                    return Err(ProtocolError::Credit);
                }
                self.credit = self
                    .credit
                    .checked_add(amount)
                    .filter(|n| *n <= MAX_CREDIT_BYTES)
                    .ok_or(ProtocolError::Credit)?;
            }
            (Direction::Child, Kind::Data) if self.status == BodyStatus::Streaming => {
                if frame.offset != self.offset {
                    return Err(ProtocolError::Order);
                }
                self.credit = self
                    .credit
                    .checked_sub(frame.payload.len() as u64)
                    .ok_or(ProtocolError::Credit)?;
                self.offset = self
                    .offset
                    .checked_add(frame.payload.len() as u64)
                    .ok_or(ProtocolError::Limit)?;
            }
            (Direction::Child, Kind::BodyEnd) if self.status == BodyStatus::Streaming => {
                if frame.offset != self.offset {
                    return Err(ProtocolError::Order);
                }
                self.status = BodyStatus::Ended;
            }
            (Direction::Child, Kind::Failure) if self.status != BodyStatus::AwaitingOpen => {
                self.status = BodyStatus::Failed
            }
            (Direction::Parent, Kind::Abort) if self.status != BodyStatus::AwaitingOpen => {
                self.status = BodyStatus::Aborted
            }
            _ => return Err(ProtocolError::Order),
        }
        Ok(())
    }
}
