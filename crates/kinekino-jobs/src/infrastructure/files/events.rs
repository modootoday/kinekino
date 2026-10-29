use super::*;
use crate::domain::{JobId, valid_sha256};
use std::io::Read;
pub(super) struct Header {
    pub sequence: u64,
    pub job_id: JobId,
    pub path: PathBuf,
}
impl JobFiles {
    pub(super) fn find_submission_in(
        &self,
        headers: &[Header],
        key_sha256: &str,
    ) -> Result<Option<JobRecord>, ApiError> {
        let mut latest = std::collections::BTreeMap::new();
        for header in headers {
            latest.insert(&header.job_id, header);
        }
        if latest.len() > super::MAX_JOBS {
            return Err(corrupt());
        }
        for header in latest.values() {
            let job = self.read_event(header)?.job;
            if job
                .submission
                .as_ref()
                .is_some_and(|submission| submission.key_sha256 == key_sha256)
            {
                return Ok(Some(job));
            }
        }
        Ok(None)
    }
    pub(super) fn headers(&self) -> Result<Vec<Header>, ApiError> {
        let mut headers = Vec::new();
        for entry in fs::read_dir(&self.events).map_err(io_error)? {
            if headers.len() >= MAX_EVENTS {
                return Err(exhausted("Job journal metadata exceeds its read bound"));
            }
            let entry = entry.map_err(io_error)?;
            let name = entry.file_name();
            let name = name.to_str().ok_or_else(corrupt)?;
            let (number, id) = name
                .strip_prefix("e-")
                .and_then(|name| name.split_once('-'))
                .ok_or_else(corrupt)?;
            let sequence = number
                .parse::<u64>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or_else(corrupt)?;
            let job_id: JobId = id.to_owned().try_into().map_err(|_| corrupt())?;
            if name != format!("e-{sequence}-{}", job_id.as_str()) {
                return Err(corrupt());
            }
            headers.push(Header {
                sequence,
                job_id,
                path: entry.path(),
            });
        }
        headers.sort_by_key(|header| header.sequence);
        if headers
            .iter()
            .enumerate()
            .any(|(index, header)| header.sequence != index as u64 + 1)
        {
            return Err(corrupt());
        }
        Ok(headers)
    }
    pub(super) fn read_event(&self, header: &Header) -> Result<JobEvent, ApiError> {
        let _guard = self.paths.directory(&header.path)?;
        let files = fs::read_dir(&header.path)
            .map_err(io_error)?
            .take(2)
            .collect::<Result<Vec<_>, _>>()
            .map_err(io_error)?;
        if files.len() != 1 {
            return Err(corrupt());
        }
        let path = files[0].path();
        let sha = path
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|sha| valid_sha256(sha))
            .ok_or_else(corrupt)?;
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            return Err(corrupt());
        }
        let mut file = self.paths.file(&path)?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(MAX_EVENT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() > MAX_EVENT_BYTES || sources::digest(&bytes) != sha {
            return Err(corrupt());
        }
        let event: JobEvent = serde_json::from_slice(&bytes).map_err(|_| corrupt())?;
        event.job.validate().map_err(|_| corrupt())?;
        if event.sequence.0 != header.sequence
            || event.job.job_id != header.job_id
            || serde_json::to_vec(&event).map_err(|_| corrupt())? != bytes
        {
            return Err(corrupt());
        }
        Ok(event)
    }
}
