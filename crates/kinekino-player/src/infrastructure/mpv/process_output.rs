use std::{
    io::Read,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
const MAX_STREAM_BYTES: usize = 512 * 1024;
#[derive(Clone, Default)]
pub(super) struct OutputSnapshot {
    pub bytes: Vec<u8>,
    pub collector_started: bool,
    pub eof_observed: bool,
    pub truncated: bool,
    pub read_failed: bool,
}
pub(super) type LatestOutput = Arc<Mutex<OutputSnapshot>>;
pub(super) fn collect(
    mut reader: impl Read + Send + 'static,
    latest: LatestOutput,
    name: &'static str,
) -> std::io::Result<JoinHandle<()>> {
    std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            if let Ok(mut state) = latest.lock() {
                state.collector_started = true;
            }
            let mut buffer = [0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        if let Ok(mut state) = latest.lock() {
                            state.eof_observed = true;
                        }
                        break;
                    }
                    Ok(count) => {
                        if let Ok(mut state) = latest.lock() {
                            let take =
                                count.min(MAX_STREAM_BYTES.saturating_sub(state.bytes.len()));
                            state.bytes.extend_from_slice(&buffer[..take]);
                            state.truncated |= take != count;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        if let Ok(mut state) = latest.lock() {
                            state.read_failed = true;
                        }
                        break;
                    }
                }
            }
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_prefix_preserves_original_bytes_and_drains_after_limit() {
        let bytes = vec![0xff; MAX_STREAM_BYTES + 17];
        let latest = Arc::new(Mutex::new(OutputSnapshot::default()));
        collect(
            std::io::Cursor::new(bytes),
            latest.clone(),
            "synthetic-output",
        )
        .unwrap()
        .join()
        .unwrap();
        let state = latest.lock().unwrap();
        assert_eq!(state.bytes, vec![0xff; MAX_STREAM_BYTES]);
        assert!(state.truncated);
        assert!(!state.read_failed);
        assert!(state.collector_started && state.eof_observed);
    }
}

#[cfg(test)]
mod empty_observation_tests {
    use super::*;
    #[test]
    fn empty_default_is_not_observed_empty_stream() {
        let initial = OutputSnapshot::default();
        assert!(!initial.collector_started && !initial.eof_observed);
        let latest = Arc::new(Mutex::new(OutputSnapshot::default()));
        collect(
            std::io::Cursor::new(Vec::<u8>::new()),
            latest.clone(),
            "empty-observed-output",
        )
        .unwrap()
        .join()
        .unwrap();
        let observed = latest.lock().unwrap();
        assert!(observed.bytes.is_empty());
        assert!(observed.collector_started && observed.eof_observed);
        assert!(!observed.read_failed);
    }
}
