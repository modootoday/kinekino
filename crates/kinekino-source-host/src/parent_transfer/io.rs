use super::*;
use kinekino_process::infrastructure::ipc::{IoPoll, MAX_CONTROL_SEGMENT, MAX_DATA_SEGMENT};
impl<G: Send> ParentTransfer<G> {
    /// Original current/attempt/control check surrounds each bounded IO turn.
    /// Control writes and data reads progress independently; no reader thread.
    pub fn poll_io(
        &mut self,
        mut check: impl FnMut() -> Result<(), TransferError>,
    ) -> Result<TransferProgress, TransferError> {
        self.healthy()?;
        if !self.started {
            return Err(TransferError::Order);
        }
        let result = (|| {
            check()?;
            self.write_step()?;
            let received = self.read_step()?;
            check()?;
            Ok(TransferProgress {
                bootstrap_sent: self.bootstrap_sent,
                open_sent: self.open_sent,
                frame_received: received,
                metadata_ready: self.metadata().is_some(),
            })
        })();
        self.record(result)
    }
    fn write_step(&mut self) -> Result<(), TransferError> {
        if self.tx.is_none() {
            return Ok(());
        }
        if !self.writing {
            let tx = self.tx.as_mut().expect("retained control bytes");
            tx.offered = (tx.bytes.len() - tx.offset).min(MAX_CONTROL_SEGMENT);
            let result = self
                .launch
                .host_mut()
                .ok_or(TransferError::Order)?
                .ipc_mut()
                .ok_or(TransferError::Closing)?
                .control()
                .start_write(&tx.bytes[tx.offset..tx.offset + tx.offered]);
            self.record_io(result)?;
            self.writing = true;
        }
        let result = self
            .launch
            .host_mut()
            .ok_or(TransferError::Order)?
            .ipc_mut()
            .ok_or(TransferError::Closing)?
            .control()
            .poll_write();
        match self.record_io(result)? {
            IoPoll::Pending => Ok(()),
            IoPoll::Written(count) => {
                self.writing = false;
                let tx = self.tx.as_mut().expect("original pending control");
                if count == 0 || count > tx.offered {
                    return Err(TransferError::Protocol(ProtocolError::Io));
                }
                tx.offset += count;
                tx.offered = 0;
                if tx.offset == tx.bytes.len() {
                    self.bootstrap_sent |= tx.bootstrap;
                    self.open_sent |= tx.open;
                    self.tx = None;
                }
                Ok(())
            }
            _ => Err(TransferError::Protocol(ProtocolError::Io)),
        }
    }
    fn read_step(&mut self) -> Result<bool, TransferError> {
        if !self.stream.as_ref().is_some_and(ParentStream::can_receive) {
            return Ok(false);
        }
        if self.input_offset == self.input.len() {
            self.input.clear();
            self.input_offset = 0;
            if !self.reading {
                let result = self
                    .launch
                    .host_mut()
                    .ok_or(TransferError::Order)?
                    .ipc_mut()
                    .ok_or(TransferError::Closing)?
                    .data()
                    .start_read(MAX_DATA_SEGMENT);
                self.record_io(result)?;
                self.reading = true;
            }
            let result = self
                .launch
                .host_mut()
                .ok_or(TransferError::Order)?
                .ipc_mut()
                .ok_or(TransferError::Closing)?
                .data()
                .poll_read();
            match self.record_io(result)? {
                IoPoll::Pending => return Ok(false),
                IoPoll::Read(bytes) => {
                    self.reading = false;
                    self.input = bytes;
                }
                IoPoll::Eof => {
                    self.reading = false;
                    self.receiver.eof().map_err(TransferError::Protocol)?;
                    self.stream
                        .as_mut()
                        .ok_or(TransferError::Order)?
                        .transport_eof()
                        .map_err(TransferError::Parent)?;
                    return Ok(false);
                }
                _ => return Err(TransferError::Protocol(ProtocolError::Io)),
            }
        }
        let (used, frame) = self
            .receiver
            .push(&self.input[self.input_offset..])
            .map_err(TransferError::Protocol)?;
        self.input_offset += used;
        if let Some(frame) = frame {
            self.stream
                .as_mut()
                .ok_or(TransferError::Order)?
                .receive(frame, uptime_millis())
                .map_err(TransferError::Parent)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
