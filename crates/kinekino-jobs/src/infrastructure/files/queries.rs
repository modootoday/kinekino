use super::*;
use crate::{
    application::{EventPage, JobListCursor, JobPage},
    domain::JobId,
};
fn limit(value: usize) -> Result<(), ApiError> {
    if !(1..=200).contains(&value) {
        Err(invalid("Job page limit must be1..200"))
    } else {
        Ok(())
    }
}
impl JobFiles {
    pub(super) fn query_job(&self, id: &JobId) -> Result<JobRecord, ApiError> {
        let _local = self.local_gate()?;
        let _writer = self.writer_lock()?;
        let headers = self.headers()?;
        if let Some(tail) = headers.last() {
            self.read_event(tail)?;
        }
        let header = headers
            .iter()
            .rev()
            .find(|header| &header.job_id == id)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Job not found"))?;
        Ok(self.read_event(header)?.job)
    }
    pub(super) fn query_events(&self, after: u64, count: usize) -> Result<EventPage, ApiError> {
        let _local = self.local_gate()?;
        limit(count)?;
        let _writer = self.writer_lock()?;
        let headers = self.headers()?;
        let latest = headers.len() as u64;
        if after > latest {
            return Err(invalid("Event cursor is beyond this journal"));
        }
        if let Some(tail) = headers.last() {
            self.read_event(tail)?;
        }
        let mut events = Vec::new();
        let mut used = 512usize;
        for header in headers.iter().skip(after as usize).take(count) {
            let event = self.read_event(header)?;
            let bytes = serde_json::to_vec(&event).map_err(|_| corrupt())?.len() + 1;
            if used + bytes > MAX_PAGE_BYTES {
                break;
            }
            used += bytes;
            events.push(event);
        }
        let next = events.last().map_or(after, |event| event.sequence.0);
        Ok(EventPage {
            events,
            next_sequence: DecimalU64(next),
            latest_sequence: DecimalU64(latest),
            earliest_sequence: DecimalU64(if latest == 0 { 0 } else { 1 }),
            gap: false,
        })
    }
    pub(super) fn query_jobs(
        &self,
        cursor: Option<&JobListCursor>,
        count: usize,
    ) -> Result<JobPage, ApiError> {
        let _local = self.local_gate()?;
        limit(count)?;
        let _writer = self.writer_lock()?;
        let headers = self.headers()?;
        let latest = headers.len() as u64;
        if let Some(tail) = headers.last() {
            self.read_event(tail)?;
        }
        let through = cursor.map_or(latest, |cursor| cursor.through_sequence.0);
        let after = cursor.map_or(0, |cursor| cursor.after_created_sequence.0);
        if through > latest || after > through {
            return Err(invalid("Invalid job list cursor"));
        }
        let mut index = std::collections::BTreeMap::new();
        for header in headers.iter().filter(|header| header.sequence <= through) {
            let entry = index
                .entry(header.job_id.clone())
                .or_insert((header.sequence, header));
            entry.1 = header;
        }
        let mut items: Vec<_> = index
            .values()
            .filter(|(created, _)| *created > after)
            .collect();
        items.sort_by_key(|(created, _)| *created);
        let mut jobs = Vec::new();
        let mut used = 512usize;
        let mut last = after;
        for (created, header) in &items {
            if jobs.len() == count {
                break;
            }
            let job = self.read_event(header)?.job;
            let bytes = serde_json::to_vec(&job).map_err(|_| corrupt())?.len() + 1;
            if used + bytes > MAX_PAGE_BYTES {
                break;
            }
            used += bytes;
            last = *created;
            jobs.push(job);
        }
        let next_cursor = (jobs.len() < items.len()).then_some(JobListCursor {
            through_sequence: DecimalU64(through),
            after_created_sequence: DecimalU64(last),
        });
        Ok(JobPage { jobs, next_cursor })
    }
}
