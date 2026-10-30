use super::labels;
use crate::application::{
    api::ApiError,
    jobs::{JobId, JobPage},
};
use meari::domain::{
    Prop, PropType, Value,
    array_value::ArrayValue,
    host_callbacks::{CallbackReference, CallbackSignature},
    record_value::RecordValue,
};
use std::collections::BTreeMap;
use std::{collections::BTreeSet, num::NonZeroU64};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListStamp {
    route: NonZeroU64,
    request: NonZeroU64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListPresentationError {
    Closing,
    Busy,
    CounterExhausted,
    Stale,
    InvalidRecord(String),
    DuplicateIdentity,
}

#[derive(Clone)]
pub struct JobsListProjection {
    route: NonZeroU64,
    sequence: u64,
    pending: Option<ListStamp>,
    page: Option<JobPage>,
    diagnostic: String,
    closing: bool,
}
impl JobsListProjection {
    pub fn new(route: NonZeroU64) -> Self {
        Self {
            route,
            sequence: 0,
            pending: None,
            page: None,
            diagnostic: String::new(),
            closing: false,
        }
    }
    pub fn begin(&mut self) -> Result<ListStamp, ListPresentationError> {
        if self.closing {
            return Err(ListPresentationError::Closing);
        }
        if self.pending.is_some() {
            return Err(ListPresentationError::Busy);
        }
        let next = self
            .sequence
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or(ListPresentationError::CounterExhausted)?;
        let stamp = ListStamp {
            route: self.route,
            request: next,
        };
        self.sequence = next.get();
        self.pending = Some(stamp);
        self.diagnostic.clear();
        Ok(stamp)
    }
    fn current(&self, stamp: ListStamp) -> Result<(), ListPresentationError> {
        if self.closing || self.pending != Some(stamp) || stamp.route != self.route {
            return Err(ListPresentationError::Stale);
        }
        Ok(())
    }
    /// The caller retains the original terminal/DTO if validation or projection fails.
    pub fn accept(
        &mut self,
        stamp: ListStamp,
        page: &JobPage,
    ) -> Result<(), ListPresentationError> {
        self.current(stamp)?;
        let mut identities = BTreeSet::new();
        for job in &page.jobs {
            job.validate()
                .map_err(|error| ListPresentationError::InvalidRecord(error.into()))?;
            if !identities.insert(job.job_id.as_str()) {
                return Err(ListPresentationError::DuplicateIdentity);
            }
        }
        self.page = Some(page.clone());
        self.pending = None;
        self.diagnostic.clear();
        Ok(())
    }
    pub fn fail(
        &mut self,
        stamp: ListStamp,
        error: &ApiError,
    ) -> Result<(), ListPresentationError> {
        self.reject(stamp, format!("{:?}: {}", error.code, error.message))
    }
    pub fn reject(
        &mut self,
        stamp: ListStamp,
        diagnostic: String,
    ) -> Result<(), ListPresentationError> {
        self.current(stamp)?;
        self.diagnostic = diagnostic;
        self.pending = None;
        Ok(())
    }
    pub fn page(&self) -> Option<&JobPage> {
        self.page.as_ref()
    }
    pub fn route(&self) -> NonZeroU64 {
        self.route
    }
    pub fn contains(&self, job: &JobId) -> bool {
        self.page
            .as_ref()
            .is_some_and(|page| page.jobs.iter().any(|record| &record.job_id == job))
    }
    pub fn diagnostic(&self) -> &str {
        &self.diagnostic
    }
    pub fn status(&self) -> &'static str {
        if self.closing {
            labels::CLOSING
        } else if self.pending.is_some() {
            labels::LOADING
        } else if !self.diagnostic.is_empty() {
            labels::FAILED
        } else if self.page.as_ref().is_some_and(|page| page.jobs.is_empty()) {
            labels::EMPTY
        } else if self.page.is_some() {
            labels::RECEIVED
        } else {
            labels::NOT_REQUESTED
        }
    }
    pub fn previous_snapshot(&self) -> bool {
        self.page.is_some() && (self.pending.is_some() || !self.diagnostic.is_empty())
    }
    pub fn disabled(&self) -> bool {
        self.closing || self.pending.is_some()
    }
    pub fn begin_close(&mut self) {
        // SDK terminal drain/ACK/release belongs to the original consumer, not this view.
        self.closing = true;
    }

    pub fn props(
        &self,
        selection: &super::selection::JobsSelection,
    ) -> Result<BTreeMap<String, Value>, String> {
        let schema: Vec<_> = ["key", "operation", "source", "state", "output"]
            .into_iter()
            .map(|name| Prop {
                name: name.into(),
                kind: PropType::String,
            })
            .chain(["selected", "disabled"].into_iter().map(|name| Prop {
                name: name.into(),
                kind: PropType::Bool,
            }))
            .collect();
        let mut rows = Vec::new();
        if let Some(page) = &self.page {
            for record in &page.jobs {
                let mut fields = BTreeMap::new();
                for (key, text) in [
                    ("key", record.job_id.as_str().to_owned()),
                    ("operation", record.input.operation.clone()),
                    (
                        "source",
                        format!(
                            "{} · {} bytes",
                            record.input.source.sha256, record.input.source.bytes.0
                        ),
                    ),
                    ("state", labels::state_label(record.state).into()),
                    (
                        "output",
                        record
                            .output
                            .as_ref()
                            .map(|output| output.path.clone())
                            .unwrap_or_else(|| "—".into()),
                    ),
                ] {
                    fields.insert(key.into(), Value::String(text));
                }
                fields.insert(
                    "selected".into(),
                    Value::Bool(
                        selection.active().and_then(|value| value.job()) == Some(&record.job_id),
                    ),
                );
                fields.insert(
                    "disabled".into(),
                    Value::Bool(self.disabled() || selection.pending().is_some()),
                );
                rows.push(Value::Record(RecordValue::new(schema.clone(), fields)?));
            }
        }
        let mut result = BTreeMap::from([
            (
                "rows".into(),
                Value::Array(ArrayValue {
                    element: PropType::Record(schema),
                    items: rows,
                }),
            ),
            ("status".into(), Value::String(self.status().into())),
            ("diagnostic".into(), Value::String(self.diagnostic.clone())),
            (
                "snapshot".into(),
                Value::String(
                    if self.page.is_none() {
                        "조회한 목록이 없습니다."
                    } else if self.previous_snapshot() {
                        labels::PREVIOUS
                    } else if self
                        .page
                        .as_ref()
                        .is_some_and(|page| page.next_cursor.is_some())
                    {
                        "조회한 첫 페이지입니다. 다음 페이지 이동은 아직 제공하지 않습니다."
                    } else {
                        "명시적으로 조회한 목록입니다. 실시간 갱신이 아닙니다."
                    }
                    .into(),
                ),
            ),
            (
                "refreshDisabled".into(),
                Value::Bool(self.disabled() || selection.pending().is_some()),
            ),
        ]);
        for (name, signature) in [
            ("onRefreshJobs", CallbackSignature::Action),
            ("onSelectJob", CallbackSignature::Text),
        ] {
            result.insert(
                name.into(),
                Value::CallbackReference(CallbackReference {
                    binding: name.into(),
                    signature,
                }),
            );
        }
        Ok(result)
    }
}
