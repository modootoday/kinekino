//! Owned source lanes over the central `SourcePipe` port (ADR 0006). The lane state machine
//! (cutoffs, single in-flight read/write, explicit cancellation and reap) is OS-neutral; the
//! transport is a Windows named pipe or a POSIX socketpair chosen by the OS adapter.
use super::ipc::{IoPoll, MAX_CONTROL_SEGMENT, MAX_DATA_SEGMENT, check_segment};
use super::os_policy::process;
use crate::domain::{deadline::Deadline, source_clock::SourceCutoffs};
use kinekino_core::platform::process::{PipeOperation, SourceChildEndpoint, SourcePipe};
use std::{io, sync::Arc, time::Instant};

/// Operations whose buffers the OS adapter intentionally leaked (Windows overlapped I/O only).
pub fn emergency_retained_operations() -> u64 {
    process().emergency_retained_operations()
}
/// Host-wide monotonic milliseconds shared with owned children.
pub fn uptime_millis() -> u64 {
    process().uptime_millis()
}
pub fn source_cutoffs(deadline: Deadline) -> io::Result<SourceCutoffs> {
    // Sample the shared tick FIRST, then remaining from the original Instant.
    // Flooring and sampling latency can only shorten the transmitted budget.
    let tick = uptime_millis();
    let now = Instant::now();
    SourceCutoffs::from_remaining(
        tick,
        deadline.operation_remaining(now),
        deadline.remaining(now),
    )
    .map_err(|error| io::Error::new(io::ErrorKind::TimedOut, format!("source cutoff: {error:?}")))
}

pub(crate) struct ChildEndpoints {
    pub control: Box<dyn SourceChildEndpoint>,
    pub data: Box<dyn SourceChildEndpoint>,
}
#[must_use = "Retain original endpoints through explicit cancellation completion"]
pub struct PreparedSourceIpc {
    pub(crate) parent: SourceIpc,
    pub(crate) child: Option<ChildEndpoints>,
    original_deadline: Deadline,
}
#[must_use = "Pipe completion/EOF is not family closure or a persisted receipt"]
pub struct SourceIpc {
    control: SourceLane,
    data: SourceLane,
}
pub struct SourceLane {
    pipe: Option<Arc<dyn SourcePipe>>,
    needs_connect: bool,
    connect: Option<Box<dyn PipeOperation>>,
    connected: bool,
    read: Option<Box<dyn PipeOperation>>,
    write: Option<Box<dyn PipeOperation>>,
    maximum: usize,
    eof: bool,
    closing: bool,
    first_error: Option<io::Error>,
    secondary_errors: Vec<io::Error>,
    secondary_overflow: bool,
    cutoffs: SourceCutoffs,
    parent_deadline: Option<Deadline>,
    fixture_name: Option<String>,
}
#[derive(Debug, Clone, Copy)]
pub struct ClosePoll {
    pub io_reaped: bool,
    pub local_endpoints_closed: bool,
    pub control_eof: bool,
    pub data_eof: bool,
    pub had_error: bool,
}

type Pair = (SourceLane, Box<dyn SourceChildEndpoint>);
fn pair(maximum: usize, cutoffs: SourceCutoffs, deadline: Deadline) -> io::Result<Pair> {
    let made = process().source_pipe_pair(maximum)?;
    Ok((
        SourceLane {
            pipe: Some(made.parent),
            needs_connect: made.needs_connect,
            // Do not start an operation before both lanes can be returned to their original
            // explicit-close owner.
            connect: None,
            connected: false,
            read: None,
            write: None,
            maximum,
            eof: false,
            closing: false,
            first_error: None,
            secondary_errors: Vec::new(),
            secondary_overflow: false,
            cutoffs,
            parent_deadline: Some(deadline),
            fixture_name: made.name,
        },
        made.child,
    ))
}
impl PreparedSourceIpc {
    pub fn new(deadline: Deadline) -> io::Result<Self> {
        Self::with_pair(deadline, pair)
    }
    fn with_pair(
        deadline: Deadline,
        mut prepare: impl FnMut(usize, SourceCutoffs, Deadline) -> io::Result<Pair>,
    ) -> io::Result<Self> {
        let cutoffs = source_cutoffs(deadline)?;
        let (control, control_child) = prepare(MAX_CONTROL_SEGMENT, cutoffs, deadline)?;
        let (data, data_child) = prepare(MAX_DATA_SEGMENT, cutoffs, deadline)?;
        Ok(Self {
            parent: SourceIpc { control, data },
            child: Some(ChildEndpoints {
                control: control_child,
                data: data_child,
            }),
            original_deadline: deadline,
        })
    }
    pub fn poll_connected(&mut self) -> io::Result<bool> {
        let control = self.parent.control.poll_connected()?;
        let data = self.parent.data.poll_connected()?;
        Ok(control && data)
    }
    pub fn cutoffs(&self) -> SourceCutoffs {
        self.parent.control.cutoffs
    }
    pub fn parent_handles_inheritable(&self) -> io::Result<bool> {
        for lane in [&self.parent.control, &self.parent.data] {
            if lane
                .pipe
                .as_ref()
                .expect("prepared endpoint")
                .is_inheritable()?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Diagnostic transport names (Windows pipe names); None for anonymous socket pairs.
    pub fn fixture_names(&self) -> Option<[String; 2]> {
        Some([
            self.parent.control.fixture_name.clone()?,
            self.parent.data.fixture_name.clone()?,
        ])
    }
    pub(crate) fn constrain_deadline(&mut self, requested: Deadline) -> io::Result<Deadline> {
        let deadline = self.original_deadline.constrained_by(requested);
        let ticks = source_cutoffs(deadline)?;
        for lane in [&mut self.parent.control, &mut self.parent.data] {
            lane.cutoffs = lane.cutoffs.constrained_by(ticks);
            lane.parent_deadline = Some(deadline);
        }
        self.original_deadline = deadline;
        Ok(deadline)
    }
    pub fn poll_close(&mut self) -> ClosePoll {
        drop(self.child.take());
        self.parent.poll_close()
    }
}

#[cfg(test)]
#[path = "source_channel_tests.rs"]
mod custody_tests;
impl SourceLane {
    pub fn diagnostic_pipe_name(&self) -> Option<&str> {
        self.fixture_name.as_deref()
    }
    fn check_cutoff(&self) -> io::Result<()> {
        if self.cutoffs.operation_remaining(uptime_millis()).is_zero()
            || self
                .parent_deadline
                .is_some_and(|deadline| deadline.operation_remaining(Instant::now()).is_zero())
        {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "original source operation cutoff",
            ))
        } else {
            Ok(())
        }
    }
    fn remember(&mut self, error: io::Error) {
        if self.first_error.is_none() {
            self.first_error = Some(error);
        } else if self.secondary_errors.len() < 8 {
            self.secondary_errors.push(error);
        } else {
            self.secondary_overflow = true;
        }
    }
    fn poll_connected(&mut self) -> io::Result<bool> {
        if let Some(error) = &self.first_error {
            return Err(io::Error::new(error.kind(), error.to_string()));
        }
        let result = self.connect_step();
        if let Err(error) = &result {
            self.remember(io::Error::new(error.kind(), error.to_string()));
        }
        result
    }
    fn pipe(&self) -> io::Result<Arc<dyn SourcePipe>> {
        self.pipe
            .clone()
            .ok_or_else(|| io::Error::other("source pipe closed"))
    }
    fn connect_step(&mut self) -> io::Result<bool> {
        self.check_cutoff()?;
        if self.closing {
            return Err(io::Error::other("source pipe is closing"));
        }
        if self.connected {
            return Ok(true);
        }
        if !self.needs_connect {
            // Pre-connected transport (socketpair): only the peer check remains.
            self.pipe()?.verify_peer()?;
            self.connected = true;
            return Ok(true);
        }
        if self.connect.is_none() {
            self.connect = Some(self.pipe()?.connect()?);
        }
        if let Some(operation) = self.connect.as_mut() {
            match operation.poll() {
                None => return Ok(false),
                Some(Ok(done)) if !done.cancelled && !done.eof => {
                    self.connect.take();
                }
                Some(Ok(_)) => return Err(io::Error::other("source pipe connection cancelled")),
                Some(Err(error)) => return Err(error),
            }
            self.pipe()?.verify_peer()?;
            self.connected = true;
        }
        Ok(self.connected)
    }
    pub fn first_error(&self) -> Option<&io::Error> {
        self.first_error.as_ref()
    }
    pub fn secondary_errors(&self) -> &[io::Error] {
        &self.secondary_errors
    }
    pub fn secondary_overflow(&self) -> bool {
        self.secondary_overflow
    }
    pub fn start_read(&mut self, maximum: usize) -> io::Result<()> {
        self.check_cutoff()?;
        check_segment(maximum, self.maximum)?;
        if self.closing
            || self.first_error.is_some()
            || !self.connected
            || self.connect.is_some()
            || self.read.is_some()
            || self.eof
        {
            return Err(io::Error::other("source read unavailable"));
        }
        self.read = Some(self.pipe()?.read(maximum)?);
        Ok(())
    }
    pub fn start_write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.check_cutoff()?;
        check_segment(bytes.len(), self.maximum)?;
        if self.closing
            || self.first_error.is_some()
            || !self.connected
            || self.connect.is_some()
            || self.write.is_some()
        {
            return Err(io::Error::other("source write unavailable"));
        }
        self.write = Some(self.pipe()?.write(bytes)?);
        Ok(())
    }
    pub fn poll_read(&mut self) -> io::Result<IoPoll> {
        self.poll(false)
    }
    pub fn poll_write(&mut self) -> io::Result<IoPoll> {
        self.poll(true)
    }
    fn poll(&mut self, writing: bool) -> io::Result<IoPoll> {
        let expired = self.check_cutoff().err();
        if let Some(error) = &expired {
            self.remember(io::Error::new(error.kind(), error.to_string()));
        }
        let slot = if writing {
            &mut self.write
        } else {
            &mut self.read
        };
        let operation = slot
            .as_mut()
            .ok_or_else(|| io::Error::other("source operation absent"))?;
        let cancel_error = if expired.is_some() {
            operation.cancel().err()
        } else {
            None
        };
        let polled = operation.poll();
        if operation.reaped() {
            slot.take();
        }
        if let Some(error) = cancel_error {
            self.remember(error);
        }
        let result = match polled {
            None => return Ok(IoPoll::Pending),
            Some(result) => result,
        };
        if let Some(error) = expired {
            return Err(error);
        }
        match result {
            Ok(done) if done.cancelled => Ok(IoPoll::Cancelled),
            Ok(done) if done.eof => {
                self.eof = true;
                Ok(IoPoll::Eof)
            }
            Ok(done) if writing => Ok(IoPoll::Written(done.count)),
            Ok(done) => Ok(IoPoll::Read(done.bytes)),
            Err(error) => {
                self.remember(io::Error::new(error.kind(), error.to_string()));
                Err(error)
            }
        }
    }
    fn close_step(&mut self) -> bool {
        if !self.closing {
            self.closing = true;
            let mut errors = Vec::new();
            for operation in [&mut self.connect, &mut self.read, &mut self.write]
                .into_iter()
                .flatten()
            {
                if let Err(error) = operation.cancel() {
                    errors.push(error);
                }
            }
            for error in errors {
                self.remember(error);
            }
        }
        let mut errors = Vec::new();
        for slot in [&mut self.connect, &mut self.read, &mut self.write] {
            if let Some(operation) = slot.as_mut()
                && let Some(result) = operation.poll()
            {
                match result {
                    Ok(done) => self.eof |= done.eof,
                    Err(error) => errors.push(error),
                }
                if operation.reaped() {
                    slot.take();
                }
            }
        }
        for error in errors {
            self.remember(error);
        }
        let reaped = self.connect.is_none() && self.read.is_none() && self.write.is_none();
        if reaped {
            self.pipe.take();
        }
        reaped
    }
}
impl SourceIpc {
    /// Exact inherited clock values, not recomputed timeouts or admission.
    pub fn source_cutoffs(&self) -> SourceCutoffs {
        self.control.cutoffs
    }
    pub fn control(&mut self) -> &mut SourceLane {
        &mut self.control
    }
    pub fn data(&mut self) -> &mut SourceLane {
        &mut self.data
    }
    pub fn poll_close(&mut self) -> ClosePoll {
        let control = self.control.close_step();
        let data = self.data.close_step();
        ClosePoll {
            io_reaped: control && data,
            local_endpoints_closed: self.control.pipe.is_none() && self.data.pipe.is_none(),
            control_eof: self.control.eof,
            data_eof: self.data.eof,
            had_error: self.control.first_error.is_some() || self.data.first_error.is_some(),
        }
    }
    /// # Safety
    /// Values must be the two distinct ORIGINAL endpoints inherited by this trusted
    /// child from spawn_source, transferred exactly once, never arbitrary argv.
    pub unsafe fn from_inherited(
        control: usize,
        data: usize,
        cutoffs: SourceCutoffs,
    ) -> io::Result<Self> {
        if control == 0
            || data == 0
            || control == usize::MAX
            || data == usize::MAX
            || control == data
        {
            return Err(io::Error::other("invalid source endpoints"));
        }
        fn adopt(value: usize, maximum: usize, cutoffs: SourceCutoffs) -> io::Result<SourceLane> {
            // SAFETY: delegated original-inherited-endpoint contract.
            let pipe = unsafe { process().adopt_inherited_source(value)? };
            Ok(SourceLane {
                pipe: Some(pipe),
                needs_connect: false,
                connect: None,
                // These are the original already-connected inherited endpoints.
                connected: true,
                read: None,
                write: None,
                maximum,
                eof: false,
                closing: false,
                first_error: None,
                secondary_errors: Vec::new(),
                secondary_overflow: false,
                cutoffs,
                parent_deadline: None,
                fixture_name: None,
            })
        }
        Ok(Self {
            control: adopt(control, MAX_CONTROL_SEGMENT, cutoffs)?,
            data: adopt(data, MAX_DATA_SEGMENT, cutoffs)?,
        })
    }
}
