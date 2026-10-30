//! Offline private IPC fixture. No HTTP client, browser admission, or media engine.
#[cfg(windows)]
mod fixture {
    use kinekino_process::{domain::source_clock::SourceCutoffs, infrastructure::ipc::*};
    use kinekino_source_host::{
        codec,
        domain::{
            Cutoffs, Frame, Kind,
            state::{Direction, Exchange},
        },
        http_metadata::HttpMetadata,
        request::{BootstrapReceiver, HttpBootstrap},
        transport::FrameReceiver,
    };
    use std::time::Duration;
    const BODY: &[u8] = b"offline-sdk-body";
    struct Reader {
        bytes: Vec<u8>,
        offset: usize,
    }
    fn tick(cutoffs: SourceCutoffs) -> Result<(), ()> {
        if uptime_millis() >= cutoffs.operation() {
            Err(())
        } else {
            Ok(())
        }
    }
    impl Reader {
        fn fill(&mut self, ipc: &mut SourceIpc, cutoffs: SourceCutoffs) -> Result<(), ()> {
            if self.offset < self.bytes.len() {
                return Ok(());
            }
            tick(cutoffs)?;
            ipc.control()
                .start_read(MAX_CONTROL_SEGMENT)
                .map_err(|_| ())?;
            loop {
                tick(cutoffs)?;
                match ipc.control().poll_read().map_err(|_| ())? {
                    IoPoll::Pending => std::thread::sleep(Duration::from_millis(1)),
                    IoPoll::Read(bytes) if !bytes.is_empty() => {
                        self.bytes = bytes;
                        self.offset = 0;
                        return Ok(());
                    }
                    _ => return Err(()),
                }
            }
        }
        fn bootstrap(
            &mut self,
            ipc: &mut SourceIpc,
            cutoffs: SourceCutoffs,
        ) -> Result<HttpBootstrap, ()> {
            let mut receiver = BootstrapReceiver::new();
            loop {
                self.fill(ipc, cutoffs)?;
                let (used, request) = receiver.push(&self.bytes[self.offset..]).map_err(|_| ())?;
                self.offset += used;
                if let Some(request) = request {
                    return Ok(request);
                }
            }
        }
        fn frame(&mut self, ipc: &mut SourceIpc, cutoffs: SourceCutoffs) -> Result<Frame, ()> {
            let mut receiver = FrameReceiver::control_only();
            loop {
                self.fill(ipc, cutoffs)?;
                let (used, frame) = receiver.push(&self.bytes[self.offset..]).map_err(|_| ())?;
                self.offset += used;
                if let Some(frame) = frame {
                    return Ok(frame);
                }
            }
        }
    }
    fn send(
        ipc: &mut SourceIpc,
        cutoffs: SourceCutoffs,
        exchange: &mut Exchange,
        frame: Frame,
    ) -> Result<(), ()> {
        exchange
            .accept(Direction::Child, &frame, uptime_millis())
            .map_err(|_| ())?;
        let bytes = codec::encode(&frame).map_err(|_| ())?;
        let mut offset = 0;
        while offset < bytes.len() {
            tick(cutoffs)?;
            let offered = (bytes.len() - offset).min(MAX_DATA_SEGMENT);
            ipc.data()
                .start_write(&bytes[offset..offset + offered])
                .map_err(|_| ())?;
            loop {
                tick(cutoffs)?;
                match ipc.data().poll_write().map_err(|_| ())? {
                    IoPoll::Pending => std::thread::sleep(Duration::from_millis(1)),
                    IoPoll::Written(n) if n > 0 && n <= offered => {
                        offset += n;
                        break;
                    }
                    _ => return Err(()),
                }
            }
        }
        Ok(())
    }
    fn run(ipc: &mut SourceIpc, cutoffs: SourceCutoffs) -> Result<(), ()> {
        let mut reader = Reader {
            bytes: Vec::new(),
            offset: 0,
        };
        let request = reader.bootstrap(ipc, cutoffs)?;
        if request.maximum_bytes() < (BODY.len() as u64) {
            return Err(());
        }
        let mut exchange = Exchange::new(
            *request.binding(),
            Cutoffs::new(cutoffs.operation(), cutoffs.end()).map_err(|_| ())?,
        )
        .map_err(|_| ())?;
        let open = reader.frame(ipc, cutoffs)?;
        if open.kind != Kind::Open {
            return Err(());
        }
        exchange
            .accept(Direction::Parent, &open, uptime_millis())
            .map_err(|_| ())?;
        let metadata = HttpMetadata::new(
            request.target().as_str(),
            Some(BODY.len() as u64),
            Some("application/octet-stream"),
        )
        .map_err(|_| ())?;
        send(
            ipc,
            cutoffs,
            &mut exchange,
            Frame {
                binding: *request.binding(),
                sequence: 0,
                offset: 0,
                kind: Kind::Metadata,
                payload: metadata.encode().map_err(|_| ())?,
            },
        )?;
        while exchange.credit() < (BODY.len() as u64) {
            let frame = reader.frame(ipc, cutoffs)?;
            if frame.kind != Kind::Credit {
                return Err(());
            }
            exchange
                .accept(Direction::Parent, &frame, uptime_millis())
                .map_err(|_| ())?;
        }
        send(
            ipc,
            cutoffs,
            &mut exchange,
            Frame {
                binding: *request.binding(),
                sequence: 1,
                offset: 0,
                kind: Kind::Data,
                payload: BODY.to_vec(),
            },
        )?;
        if request.target().path() == "/offline-sdk-without-body-end" {
            return Ok(());
        }
        send(
            ipc,
            cutoffs,
            &mut exchange,
            Frame {
                binding: *request.binding(),
                sequence: 2,
                offset: BODY.len() as u64,
                kind: Kind::BodyEnd,
                payload: Vec::new(),
            },
        )?;
        if request.target().path() == "/offline-sdk-high-exit" {
            // Controlled DWORD exit, not an actual memory fault or HTTP failure.
            std::process::exit(i32::MIN + 1);
        }
        if request.target().path() == "/offline-sdk-hold-after-body" {
            ipc.control()
                .start_read(MAX_CONTROL_SEGMENT)
                .map_err(|_| ())?;
            loop {
                tick(cutoffs)?;
                match ipc.control().poll_read().map_err(|_| ())? {
                    IoPoll::Pending => std::thread::sleep(Duration::from_millis(1)),
                    IoPoll::Eof => break,
                    _ => return Err(()),
                }
            }
        }
        Ok(())
    }
    pub fn main() -> i32 {
        let args: Vec<_> = std::env::args().skip(1).collect();
        let read = |prefix: &str| -> Option<u64> {
            let mut values = args.iter().filter_map(|arg| arg.strip_prefix(prefix));
            let value = values.next()?.parse().ok()?;
            if values.next().is_some() {
                None
            } else {
                Some(value)
            }
        };
        let parsed = (|| {
            Some((
                usize::try_from(read(CONTROL_ARGUMENT)?).ok()?,
                usize::try_from(read(DATA_ARGUMENT)?).ok()?,
                SourceCutoffs::new(read(OPERATION_ARGUMENT)?, read(END_ARGUMENT)?).ok()?,
            ))
        })();
        let Some((control, data, cutoffs)) = parsed else {
            return 20;
        };
        if args.len() != 4 {
            return 20;
        }
        // Trusted fixed fixture artifact receives only spawn_source's original handles.
        let Ok(mut ipc) = (unsafe { SourceIpc::from_inherited(control, data, cutoffs) }) else {
            return 21;
        };
        std::panic::set_hook(Box::new(|_| eprintln!("source-sdk-fixture-panic")));
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&mut ipc, cutoffs)));
        loop {
            let facts = ipc.poll_close();
            if facts.io_reaped && facts.local_endpoints_closed {
                return if matches!(result, Ok(Ok(()))) && !facts.had_error {
                    0
                } else {
                    23
                };
            }
            if uptime_millis() >= cutoffs.end() {
                return 124;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
#[cfg(windows)]
pub fn main() {
    std::process::exit(fixture::main());
}
#[cfg(not(windows))]
pub fn main() {
    std::process::exit(24);
}
