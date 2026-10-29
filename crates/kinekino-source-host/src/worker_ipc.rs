//! Original B endpoints only. No alternate launcher, handle duplication or HTTP.
use crate::{
    domain::{Frame, Kind, ProtocolError},
    request::{BootstrapError, BootstrapReceiver, HttpBootstrap},
    transport::FrameReceiver,
    worker_session::{WorkerError, WorkerSession},
};
use kinekino_process::{
    domain::source_clock::SourceCutoffs,
    infrastructure::ipc::{ClosePoll, IoPoll, MAX_CONTROL_SEGMENT, MAX_DATA_SEGMENT, SourceIpc},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcError {
    Io,
    Order,
    Eof,
    Protocol(ProtocolError),
    Bootstrap(BootstrapError),
    Worker(WorkerError),
}
pub enum ControlEvent {
    Pending,
    Bootstrap(HttpBootstrap),
    Frame(Frame),
    Eof,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteProgress {
    Idle,
    Pending,
    CompletedSegment,
}
/// Holds original endpoints/operations throughout close. No Debug or body logs.
pub struct WorkerIpc {
    ipc: SourceIpc,
    request: BootstrapReceiver,
    request_done: bool,
    frames: FrameReceiver,
    control: Vec<u8>,
    offset: usize,
    reading: bool,
    writing: bool,
    closing: bool,
    eof: bool,
    first: Option<IpcError>,
}
impl WorkerIpc {
    /// # Safety
    /// Exactly the two distinct original handles inherited from trusted
    /// spawn_source, transferred once. Never call on arbitrary external argv.
    pub unsafe fn from_inherited(
        control: usize,
        data: usize,
        cutoffs: SourceCutoffs,
    ) -> Result<Self, IpcError> {
        let ipc = unsafe { SourceIpc::from_inherited(control, data, cutoffs) }
            .map_err(|_| IpcError::Io)?;
        Ok(Self {
            ipc,
            request: BootstrapReceiver::new(),
            request_done: false,
            frames: FrameReceiver::control_only(),
            control: Vec::new(),
            offset: 0,
            reading: false,
            writing: false,
            closing: false,
            eof: false,
            first: None,
        })
    }
    pub fn first_error(&self) -> Option<IpcError> {
        self.first
    }
    fn available(&self) -> Result<(), IpcError> {
        if let Some(error) = self.first {
            return Err(error);
        }
        if self.closing {
            return Err(IpcError::Order);
        }
        Ok(())
    }
    fn remember<T>(&mut self, result: Result<T, IpcError>) -> Result<T, IpcError> {
        if let Err(error) = result {
            self.first.get_or_insert(error);
        }
        result
    }
    /// Parse one event, retaining any coalesced following bytes (at most8KiB).
    pub fn poll_control(&mut self) -> Result<ControlEvent, IpcError> {
        self.available()?;
        let result = self.control_step();
        self.remember(result)
    }
    fn control_step(&mut self) -> Result<ControlEvent, IpcError> {
        if self.eof {
            return Ok(ControlEvent::Eof);
        }
        if self.offset == self.control.len() {
            self.control.clear();
            self.offset = 0;
            if !self.reading {
                self.ipc
                    .control()
                    .start_read(MAX_CONTROL_SEGMENT)
                    .map_err(|_| IpcError::Io)?;
                self.reading = true;
            }
            match self.ipc.control().poll_read().map_err(|_| IpcError::Io)? {
                IoPoll::Pending => return Ok(ControlEvent::Pending),
                IoPoll::Read(bytes) => {
                    self.reading = false;
                    self.control = bytes;
                }
                IoPoll::Eof => {
                    self.reading = false;
                    if !self.request_done {
                        self.request.eof().map_err(IpcError::Bootstrap)?;
                    }
                    self.frames.eof().map_err(IpcError::Protocol)?;
                    self.eof = true;
                    return Ok(ControlEvent::Eof);
                }
                _ => return Err(IpcError::Eof),
            }
        }
        if !self.request_done {
            let (used, request) = self
                .request
                .push(&self.control[self.offset..])
                .map_err(IpcError::Bootstrap)?;
            self.offset += used;
            return match request {
                Some(request) => {
                    self.request_done = true;
                    Ok(ControlEvent::Bootstrap(request))
                }
                None => Ok(ControlEvent::Pending),
            };
        }
        let (used, frame) = self
            .frames
            .push(&self.control[self.offset..])
            .map_err(IpcError::Protocol)?;
        self.offset += used;
        match frame {
            Some(frame) if matches!(frame.kind, Kind::Open | Kind::Credit | Kind::Abort) => {
                Ok(ControlEvent::Frame(frame))
            }
            Some(_) => Err(IpcError::Protocol(ProtocolError::Order)),
            None => Ok(ControlEvent::Pending),
        }
    }
    /// B copies offered bytes into its original OVERLAPPED allocation before
    /// start_write returns. Do not advance the session before poll_write reaps it.
    pub fn poll_data_write(
        &mut self,
        session: &mut WorkerSession,
    ) -> Result<WriteProgress, IpcError> {
        self.available()?;
        let result = self.write_step(session);
        self.remember(result)
    }
    fn write_step(&mut self, session: &mut WorkerSession) -> Result<WriteProgress, IpcError> {
        if !self.writing {
            let Some(bytes) = session
                .offer_write(MAX_DATA_SEGMENT)
                .map_err(IpcError::Worker)?
            else {
                return Ok(WriteProgress::Idle);
            };
            self.ipc
                .data()
                .start_write(bytes)
                .map_err(|_| IpcError::Io)?;
            self.writing = true;
        }
        match self.ipc.data().poll_write().map_err(|_| IpcError::Io)? {
            IoPoll::Pending => Ok(WriteProgress::Pending),
            IoPoll::Written(count) => {
                self.writing = false;
                session.complete_write(count).map_err(IpcError::Worker)?;
                Ok(WriteProgress::CompletedSegment)
            }
            _ => Err(IpcError::Eof),
        }
    }
    /// Always callable after failure; original B owner cancels and reaps pending
    /// IO before closing endpoints. These facts are not process-family/receipt ACK.
    pub fn poll_close(&mut self) -> ClosePoll {
        self.closing = true;
        self.ipc.poll_close()
    }
}
