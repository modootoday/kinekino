//! Actual regular-file EOF after the original family and receipt have settled.
//! Each poll reads at most one fixed chunk per stream; errors retain both readers.
use super::*;
use std::fs::File;

#[derive(Default)]
struct StreamDrain {
    reader: Option<File>,
    eof: bool,
}
impl StreamDrain {
    fn poll(&mut self, raw: &RawFile) -> Result<bool, ApiError> {
        if self.eof {
            return Ok(true);
        }
        if self.reader.is_none() {
            self.reader = Some(raw.open_read().map_err(|_| failed("output_drain_open"))?);
        }
        let mut chunk = [0_u8; 65536];
        let count = self
            .reader
            .as_mut()
            .expect("Original drain reader")
            .read(&mut chunk)
            .map_err(|_| failed("output_drain_read"))?;
        self.eof = count == 0;
        Ok(self.eof)
    }
}

#[derive(Default)]
pub(super) struct OutputDrain {
    stdout: StreamDrain,
    stderr: StreamDrain,
}
impl OutputDrain {
    pub(super) fn complete(&self) -> bool {
        self.stdout.eof && self.stderr.eof
    }
    pub(super) fn poll(
        &mut self,
        evidence: &kinekino_process::infrastructure::evidence_files::EvidencePaths,
    ) -> Result<bool, ApiError> {
        self.stdout.poll(&evidence.stdout)?;
        self.stderr.poll(&evidence.stderr)?;
        Ok(self.complete())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_regular_readers_require_actual_zero_read_after_bounded_chunks() {
        // kinekino-process reaches the installed OS port; unit tests install it explicitly.
        let _ = crate::infrastructure::os_policy::host();
        let root = std::env::temp_dir().join(format!(
            "native-drain-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let evidence = kinekino_process::infrastructure::evidence_files::PreparedEvidence::create(
            &root, None, 0,
        )
        .unwrap();
        std::fs::write(evidence.paths.stdout.path(), vec![7_u8; 65537]).unwrap();
        std::fs::write(evidence.paths.stderr.path(), b"stderr").unwrap();
        let mut drain = OutputDrain::default();
        assert!(!drain.complete());
        assert!(!drain.poll(&evidence.paths).unwrap());
        assert!(!drain.poll(&evidence.paths).unwrap());
        assert!(drain.poll(&evidence.paths).unwrap());
        assert!(drain.complete());
        assert!(drain.poll(&evidence.paths).unwrap());
        // Evidence is retained, and observing EOF does not consume another reader.
        assert_eq!(read(&evidence.paths.stdout, 65537).unwrap(), vec![7; 65537]);
        assert_eq!(read(&evidence.paths.stderr, 6).unwrap(), b"stderr");
    }
}
