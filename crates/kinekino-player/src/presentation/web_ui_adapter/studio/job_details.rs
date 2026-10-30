//! Read-only JOB detail projection. Job policy and record validation stay in media-jobs.
use crate::application::jobs::{JobId, JobRecord};
use meari::domain::{
    Value,
    host_callbacks::{CallbackReference, CallbackSignature},
};
use std::collections::BTreeMap;

pub struct JobDetailsProjection {
    selected: JobId,
    record: Option<JobRecord>,
    pub status: String,
    pub diagnostic: String,
    pub loading: bool,
    pub retry_result: bool,
    pub closing: bool,
}
impl JobDetailsProjection {
    pub fn new(selected: JobId) -> Self {
        Self {
            selected,
            record: None,
            status: "아직 조회하지 않았습니다.".into(),
            diagnostic: String::new(),
            loading: false,
            retry_result: false,
            closing: false,
        }
    }
    pub fn selected(&self) -> &JobId {
        &self.selected
    }
    pub fn record(&self) -> Option<&JobRecord> {
        self.record.as_ref()
    }
    pub fn received(&mut self, record: JobRecord) -> Result<(), String> {
        record.validate().map_err(str::to_owned)?;
        if record.job_id != self.selected {
            return Err("조회한 작업 ID가 선택한 작업과 다릅니다.".into());
        }
        if self
            .record
            .as_ref()
            .is_some_and(|previous| previous.revision.0 > record.revision.0)
        {
            return Err("이전 revision의 응답입니다. 마지막 조회값을 유지합니다.".into());
        }
        self.record = Some(record);
        self.status = "작업 정보를 조회했습니다.".into();
        self.diagnostic.clear();
        Ok(())
    }
    pub fn props(&self) -> BTreeMap<String, Value> {
        let record = self.record.as_ref();
        let state = record
            .map(|record| super::jobs::labels::state_label(record.state))
            .unwrap_or("—");
        let mut props = BTreeMap::new();
        for (name, value) in [
            ("title", "선택한 작업 정보".into()),
            ("jobId", self.selected.as_str().into()),
            ("statusLabel", self.status.clone()),
            ("diagnostic", self.diagnostic.clone()),
            (
                "snapshotLabel",
                if self.record.is_none() {
                    "아직 조회한 작업 기록이 없습니다."
                } else if self.loading || !self.diagnostic.is_empty() {
                    "이전 조회값은 최신 상태를 보장하지 않습니다."
                } else {
                    "명시적으로 조회한 작업 기록입니다. 실시간 재생 상태가 아닙니다."
                }
                .into(),
            ),
            ("stateLabel", state.into()),
            (
                "revisionLabel",
                record
                    .map(|r| r.revision.0.to_string())
                    .unwrap_or("—".into()),
            ),
            (
                "attemptLabel",
                record
                    .map(|r| r.attempt.0.to_string())
                    .unwrap_or("—".into()),
            ),
            (
                "operationLabel",
                record
                    .map(|r| r.input.operation.clone())
                    .unwrap_or("—".into()),
            ),
            (
                "sourceSha256",
                record
                    .map(|r| r.input.source.sha256.clone())
                    .unwrap_or("—".into()),
            ),
            (
                "sourceBytes",
                record
                    .map(|r| r.input.source.bytes.0.to_string())
                    .unwrap_or("—".into()),
            ),
            (
                "parameters",
                record
                    .map(|r| {
                        serde_json::to_string_pretty(&r.input.parameters)
                            .unwrap_or_else(|_| "입력 표시 오류".into())
                    })
                    .unwrap_or("—".into()),
            ),
            (
                "outputPath",
                record
                    .and_then(|r| r.output.as_ref())
                    .map(|o| o.path.clone())
                    .unwrap_or("—".into()),
            ),
            (
                "outputSha256",
                record
                    .and_then(|r| r.output.as_ref())
                    .map(|o| o.sha256.clone())
                    .unwrap_or("—".into()),
            ),
            (
                "outputBytes",
                record
                    .and_then(|r| r.output.as_ref())
                    .map(|o| o.bytes.0.to_string())
                    .unwrap_or("—".into()),
            ),
            (
                "failureLabel",
                record
                    .and_then(|r| r.failure.as_ref())
                    .map(|f| f.message.clone())
                    .unwrap_or_default(),
            ),
            (
                "remediation",
                record
                    .and_then(|r| r.failure.as_ref())
                    .map(|f| f.remediation.clone())
                    .unwrap_or_default(),
            ),
            ("refreshLabel", "작업 정보 새로고침".into()),
            ("retryLabel", "조회 결과 다시 확인".into()),
            (
                "unsupportedLabel",
                "취소·재시도·폴더 열기는 이 패널에서 아직 연결하지 않았습니다.".into(),
            ),
        ] {
            props.insert(name.into(), Value::String(value));
        }
        props.insert(
            "refreshDisabled".into(),
            Value::Bool(self.loading || self.retry_result || self.closing),
        );
        props.insert(
            "retryDisabled".into(),
            Value::Bool(!self.retry_result || self.closing),
        );
        for name in ["onRefreshJob", "onRetryJobResult"] {
            props.insert(
                name.into(),
                Value::CallbackReference(CallbackReference {
                    binding: name.into(),
                    signature: CallbackSignature::Action,
                }),
            );
        }
        props
    }
}

#[cfg(test)]
#[path = "job_details/tests.rs"]
mod tests;
