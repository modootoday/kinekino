//! Parent wire/host composition, not SDK/replay admission or publication policy.
mod io;
use crate::{
    codec,
    domain::{Cutoffs, Frame, ProtocolError},
    parent_host::{ParentHost, ParentReceiptSink},
    parent_launch::{LaunchCloseFacts, OpenOnceError, ParentLaunch},
    parent_stream::{ParentError, ParentStream, ReadStatus},
    request::{BootstrapError, HttpBootstrap},
    transport::FrameReceiver,
};
use kinekino_process::{domain::deadline::Deadline, infrastructure::ipc::uptime_millis};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferError {
    Order,
    Closing,
    Current,
    Launch(OpenOnceError),
    Bootstrap(BootstrapError),
    Protocol(ProtocolError),
    Parent(ParentError),
    Io,
}
struct ControlWrite {
    bytes: Vec<u8>,
    offset: usize,
    offered: usize,
    bootstrap: bool,
    open: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct TransferProgress {
    pub bootstrap_sent: bool,
    pub open_sent: bool,
    pub frame_received: bool,
    pub metadata_ready: bool,
}
/// Owns URL, original launch/guards, bounded pending IO and parent queue. No Debug.
pub struct ParentTransfer<G: Send> {
    launch: ParentLaunch<G>,
    request: HttpBootstrap,
    stream: Option<ParentStream>,
    receiver: FrameReceiver,
    input: Vec<u8>,
    input_offset: usize,
    tx: Option<ControlWrite>,
    started: bool,
    closing: bool,
    writing: bool,
    reading: bool,
    bootstrap_sent: bool,
    open_sent: bool,
    first: Option<TransferError>,
    io_errors: Vec<std::io::Error>,
    io_overflow: bool,
}
impl<G: Send> ParentTransfer<G> {
    /// Original prepared IPC connection only; no child or bootstrap transmission.
    pub fn poll_prepared(&mut self) -> Result<bool, TransferError> {
        self.healthy()?;
        let result = self.launch.poll_prepared();
        self.record_io(result)
    }
    /// Original immutable bootstrap; sensitive values must not be logged.
    pub fn request(&self) -> &HttpBootstrap {
        &self.request
    }
    pub fn control_pending(&self) -> bool {
        self.tx.is_some()
    }
    pub fn cutoffs(&self) -> Option<Cutoffs> {
        self.stream.as_ref().map(ParentStream::cutoffs)
    }
    /// Infallible custody; core owns this before start. Request data grants no GET.
    pub fn new(launch: ParentLaunch<G>, request: HttpBootstrap) -> Self {
        Self {
            launch,
            request,
            stream: None,
            receiver: FrameReceiver::default(),
            input: Vec::new(),
            input_offset: 0,
            tx: None,
            started: false,
            closing: false,
            writing: false,
            reading: false,
            bootstrap_sent: false,
            open_sent: false,
            first: None,
            io_errors: Vec::new(),
            io_overflow: false,
        }
    }
    pub fn first_error(&self) -> Option<TransferError> {
        self.first
    }
    pub fn io_errors(&self) -> &[std::io::Error] {
        &self.io_errors
    }
    pub fn io_overflow(&self) -> bool {
        self.io_overflow
    }
    pub fn host(&self) -> Option<&ParentHost> {
        self.launch.host()
    }
    /// Trusted adapter diagnostic access only; do not mutate lane IO externally.
    pub fn host_mut(&mut self) -> Option<&mut ParentHost> {
        self.launch.host_mut()
    }
    fn healthy(&self) -> Result<(), TransferError> {
        if self.closing {
            return Err(TransferError::Closing);
        }
        self.first.map_or(Ok(()), Err)
    }
    fn record<T>(&mut self, result: Result<T, TransferError>) -> Result<T, TransferError> {
        if let Err(error) = result {
            self.first.get_or_insert(error);
        }
        result
    }
    fn record_io<T>(&mut self, result: std::io::Result<T>) -> Result<T, TransferError> {
        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                if self.io_errors.len() < 8 {
                    self.io_errors.push(error);
                } else {
                    self.io_overflow = true;
                }
                Err(TransferError::Io)
            }
        }
    }
    /// Launch at most once and queue only bootstrap. Open remains explicit and
    /// requires original external replay admission; start itself sends no GET.
    pub fn start(
        &mut self,
        deadline: Deadline,
        mut check: impl FnMut() -> Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        self.healthy()?;
        if self.started {
            return Err(TransferError::Order);
        }
        self.started = true;
        let mut rejected = None;
        let launched = self.launch.try_launch(deadline, || match check() {
            Ok(()) => Ok(()),
            Err(error) => {
                rejected.get_or_insert(error);
                Err(OpenOnceError::Current)
            }
        });
        if let Err(error) = launched {
            return self.record(Err(rejected.unwrap_or(TransferError::Launch(error))));
        }
        let result = (|| {
            let ticks = self
                .launch
                .host()
                .ok_or(TransferError::Order)?
                .source_cutoffs();
            let cutoffs =
                Cutoffs::new(ticks.operation(), ticks.end()).map_err(TransferError::Protocol)?;
            self.stream =
                Some(ParentStream::new(&self.request, cutoffs).map_err(TransferError::Parent)?);
            let bytes = self.request.encode().map_err(TransferError::Bootstrap)?;
            self.tx = Some(ControlWrite {
                bytes,
                offset: 0,
                offered: 0,
                bootstrap: true,
                open: false,
            });
            Ok(())
        })();
        self.record(result)
    }
    fn queue(&mut self, frame: Frame, open: bool) -> Result<(), TransferError> {
        let bytes = codec::encode(&frame).map_err(TransferError::Protocol)?;
        self.tx = Some(ControlWrite {
            bytes,
            offset: 0,
            offered: 0,
            bootstrap: false,
            open,
        });
        Ok(())
    }
    pub fn queue_open(
        &mut self,
        mut check: impl FnMut() -> Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        self.healthy()?;
        if !self.bootstrap_sent || self.open_sent || self.tx.is_some() {
            return Err(TransferError::Order);
        }
        let result = (|| {
            check()?;
            let frame = self
                .stream
                .as_mut()
                .ok_or(TransferError::Order)?
                .open_frame(uptime_millis())
                .map_err(TransferError::Parent)?;
            self.queue(frame, true)
        })();
        self.record(result)
    }
    pub fn queue_credit(
        &mut self,
        amount: u64,
        mut check: impl FnMut() -> Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        self.healthy()?;
        if !self.open_sent || self.tx.is_some() {
            return Err(TransferError::Order);
        }
        let result = (|| {
            check()?;
            let frame = self
                .stream
                .as_mut()
                .ok_or(TransferError::Order)?
                .credit_frame(amount, uptime_millis())
                .map_err(TransferError::Parent)?;
            self.queue(frame, false)
        })();
        self.record(result)
    }
    pub fn available_credit(&self) -> u64 {
        self.stream
            .as_ref()
            .map_or(0, ParentStream::available_credit)
    }
    pub fn metadata(&self) -> Option<&crate::http_metadata::HttpMetadata> {
        self.stream.as_ref().and_then(ParentStream::metadata)
    }
    pub fn read(
        &mut self,
        output: &mut [u8],
        mut check: impl FnMut() -> Result<(), TransferError>,
    ) -> Result<ReadStatus, TransferError> {
        self.healthy()?;
        let result = (|| {
            check()?;
            let read = self
                .stream
                .as_mut()
                .ok_or(TransferError::Order)?
                .read(output)
                .map_err(TransferError::Parent)?;
            check()?;
            Ok(read)
        })();
        self.record(result)
    }
    pub fn poll_close(
        &mut self,
        deadline: Deadline,
        sink: &mut impl ParentReceiptSink,
    ) -> LaunchCloseFacts {
        self.closing = true;
        self.launch.poll_close(deadline, sink)
    }
}
