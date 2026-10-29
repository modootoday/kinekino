use crate::infrastructure::evidence_files::RawFile;
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Default, Debug)]
pub struct RecordCursor {
    offset: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum RecordPoll {
    Pending,
    Record { offset: u64, bytes: Vec<u8> },
}
impl RecordCursor {
    pub fn offset(&self) -> u64 {
        self.offset
    }
    /// Regular files only. `terminal` is valid after owned child/family termination observation.
    pub fn poll(
        &mut self,
        raw: &RawFile,
        maximum: usize,
        terminal: bool,
    ) -> io::Result<RecordPoll> {
        if maximum == 0 || maximum > 16 * 1024 * 1024 {
            return Err(io::Error::other("invalid record bound"));
        }
        let mut file = raw.open_read()?;
        let length = file.metadata()?.len();
        if length < self.offset {
            return Err(io::Error::other("raw file truncated"));
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let mut bytes = Vec::new();
        file.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
        let end = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|index| index + 1);
        let count = match end {
            Some(count) if count <= maximum => count,
            _ if bytes.len() > maximum => {
                return Err(io::Error::other("record prefix exceeds bound"));
            }
            None if terminal && !bytes.is_empty() => bytes.len(),
            _ => return Ok(RecordPoll::Pending),
        };
        bytes.truncate(count);
        let offset = self.offset;
        self.offset = self
            .offset
            .checked_add(count as u64)
            .ok_or_else(|| io::Error::other("record offset overflow"))?;
        Ok(RecordPoll::Record { offset, bytes })
    }
}
