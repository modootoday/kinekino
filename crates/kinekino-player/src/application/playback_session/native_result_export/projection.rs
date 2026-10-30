//! Finite v1 read DTOs and compact encoding. Transport/CLI integration is a later stage.
use super::codec::NativeExportValueError;
use super::{
    codec::{NativeIssuerScopeV1, NativeResultRefV1},
    metadata::NativeExportMetadataV1,
};
use crate::application::{
    api::{ApiError, ErrorCode},
    playback_session::GuardedSeekState,
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, SeqAccess, Visitor},
};
use sha2::{Digest, Sha256};

pub const NATIVE_EXPORT_WIRE_BYTES: usize = 98_304;
pub const NATIVE_EXPORT_JSON_BYTES: usize = NATIVE_EXPORT_WIRE_BYTES - 5;
pub const NATIVE_EXPORT_ITEM_BYTES: usize = 1024;
pub const NATIVE_EXPORT_MAX_ROWS: usize = 64;
pub const NATIVE_EXPORT_MESSAGE_BYTES: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeExportStateV1 {
    Queued,
    WriteCommitted,
    WrittenAwaitingAck,
    CommandAccepted,
    RejectedBeforeWrite,
    Failed,
    OutcomeUnobserved,
}
impl From<GuardedSeekState> for NativeExportStateV1 {
    fn from(value: GuardedSeekState) -> Self {
        match value {
            GuardedSeekState::Queued => Self::Queued,
            GuardedSeekState::WriteCommitted => Self::WriteCommitted,
            GuardedSeekState::WrittenAwaitingAck => Self::WrittenAwaitingAck,
            GuardedSeekState::CommandAccepted => Self::CommandAccepted,
            GuardedSeekState::RejectedBeforeWrite => Self::RejectedBeforeWrite,
            GuardedSeekState::Failed => Self::Failed,
            GuardedSeekState::OutcomeUnobserved => Self::OutcomeUnobserved,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeExportReadModeV1 {
    Standalone,
    Routed,
    Unavailable,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExportItemV1 {
    reference: NativeResultRefV1,
    metadata: NativeExportMetadataV1,
    state: NativeExportStateV1,
    read_mode: NativeExportReadModeV1,
}
impl NativeExportItemV1 {
    pub fn new(
        reference: NativeResultRefV1,
        metadata: NativeExportMetadataV1,
        state: NativeExportStateV1,
        read_mode: NativeExportReadModeV1,
    ) -> Result<Self, ApiError> {
        let value = Self {
            reference,
            metadata,
            state,
            read_mode,
        };
        bounded(&value, NATIVE_EXPORT_ITEM_BYTES)?;
        Ok(value)
    }
    pub fn reference(&self) -> NativeResultRefV1 {
        self.reference
    }
    pub fn metadata(&self) -> &NativeExportMetadataV1 {
        &self.metadata
    }
    pub fn state(&self) -> NativeExportStateV1 {
        self.state
    }
    pub fn read_mode(&self) -> NativeExportReadModeV1 {
        self.read_mode
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListInput {
    scope: NativeIssuerScopeV1,
    #[serde(deserialize_with = "bounded_items")]
    items: Vec<NativeExportItemV1>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "ListInput")]
pub struct NativeExportListV1 {
    scope: NativeIssuerScopeV1,
    items: Vec<NativeExportItemV1>,
}
impl NativeExportListV1 {
    pub fn new(
        scope: NativeIssuerScopeV1,
        items: Vec<NativeExportItemV1>,
    ) -> Result<Self, ApiError> {
        if items.len() > NATIVE_EXPORT_MAX_ROWS {
            return Err(exhausted());
        }
        for (index, item) in items.iter().enumerate() {
            if item.reference.scope() != scope
                || items[..index]
                    .iter()
                    .any(|prior| prior.reference == item.reference)
            {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Mismatched or duplicate original native export reference",
                ));
            }
            bounded(item, NATIVE_EXPORT_ITEM_BYTES)?;
        }
        Ok(Self {
            scope,
            items: items.into_boxed_slice().into_vec(),
        })
    }
    pub fn scope(&self) -> NativeIssuerScopeV1 {
        self.scope
    }
    pub fn items(&self) -> &[NativeExportItemV1] {
        &self.items
    }
}
impl TryFrom<ListInput> for NativeExportListV1 {
    type Error = NativeExportValueError;
    fn try_from(value: ListInput) -> Result<Self, Self::Error> {
        Self::new(value.scope, value.items).map_err(Into::into)
    }
}
fn bounded_items<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<NativeExportItemV1>, D::Error> {
    struct Items;
    impl<'de> Visitor<'de> for Items {
        type Value = Vec<NativeExportItemV1>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("at most 64 native result items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut items = Vec::new();
            while items.len() < NATIVE_EXPORT_MAX_ROWS {
                match seq.next_element()? {
                    Some(item) => items.push(item),
                    None => return Ok(items),
                }
            }
            if seq.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom("native export row budget exceeded"));
            }
            Ok(items)
        }
    }
    deserializer.deserialize_seq(Items)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeExportFailureStageV1 {
    BeforeWrite,
    Write,
    Acknowledgement,
    Connection,
    Deadline,
}
pub struct NativeExportFailureParts<'a> {
    pub code: ErrorCode,
    pub stage: NativeExportFailureStageV1,
    pub message: &'a str,
    pub original_message_bytes: u64,
    pub sha256: [u8; 32],
    pub truncated: bool,
    pub raw_os_code: Option<i32>,
    pub kind: Option<&'a str>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FailureInput {
    code: ErrorCode,
    stage: NativeExportFailureStageV1,
    message_prefix: String,
    original_message_bytes: crate::domain::api_values::DecimalU64,
    sha256: String,
    truncated: bool,
    raw_os_code: Option<i32>,
    kind: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "FailureInput")]
pub struct NativeExportFailureV1 {
    code: ErrorCode,
    stage: NativeExportFailureStageV1,
    message_prefix: String,
    original_message_bytes: crate::domain::api_values::DecimalU64,
    sha256: String,
    truncated: bool,
    raw_os_code: Option<i32>,
    kind: Option<String>,
}
impl TryFrom<FailureInput> for NativeExportFailureV1 {
    type Error = NativeExportValueError;
    fn try_from(value: FailureInput) -> Result<Self, Self::Error> {
        if value.message_prefix.len() > NATIVE_EXPORT_MESSAGE_BYTES
            || value.original_message_bytes.0 < value.message_prefix.len() as u64
            || !crate::domain::identifiers::valid_sha256(&value.sha256)
            || (value.original_message_bytes.0 > value.message_prefix.len() as u64
                && !value.truncated)
            || value.kind.as_ref().is_some_and(|kind| {
                kind.len() > 48
                    || kind.is_empty()
                    || !kind.bytes().all(|byte| byte.is_ascii_alphabetic())
            })
        {
            return Err(invalid());
        }
        Ok(Self {
            code: value.code,
            stage: value.stage,
            message_prefix: compact(value.message_prefix),
            original_message_bytes: value.original_message_bytes,
            sha256: compact(value.sha256),
            truncated: value.truncated,
            raw_os_code: value.raw_os_code,
            kind: value.kind.map(compact),
        })
    }
}
impl NativeExportFailureV1 {
    pub fn from_native(parts: NativeExportFailureParts<'_>) -> Result<Self, ApiError> {
        if parts.original_message_bytes < parts.message.len() as u64
            || parts.kind.is_some_and(|kind| {
                kind.len() > 48
                    || kind.is_empty()
                    || !kind.bytes().all(|byte| byte.is_ascii_alphabetic())
            })
        {
            return Err(invalid().into());
        }
        let message_prefix = prefix(parts.message);
        Self::try_from(FailureInput {
            code: parts.code,
            stage: parts.stage,
            original_message_bytes: crate::domain::api_values::DecimalU64(
                parts.original_message_bytes,
            ),
            sha256: hex(parts.sha256),
            truncated: parts.truncated
                || parts.original_message_bytes > message_prefix.len() as u64,
            message_prefix,
            raw_os_code: parts.raw_os_code,
            kind: parts.kind.map(str::to_owned),
        })
        .map_err(Into::into)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoreErrorInput {
    code: ErrorCode,
    retryable: bool,
    message_prefix: String,
    original_message_bytes: crate::domain::api_values::DecimalU64,
    sha256: String,
    truncated: bool,
    details_omitted: bool,
    remediation_omitted: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "CoreErrorInput")]
pub struct NativeExportCoreErrorV1 {
    code: ErrorCode,
    retryable: bool,
    message_prefix: String,
    original_message_bytes: crate::domain::api_values::DecimalU64,
    sha256: String,
    truncated: bool,
    details_omitted: bool,
    remediation_omitted: bool,
}
impl TryFrom<CoreErrorInput> for NativeExportCoreErrorV1 {
    type Error = NativeExportValueError;
    fn try_from(value: CoreErrorInput) -> Result<Self, Self::Error> {
        if value.message_prefix.len() > NATIVE_EXPORT_MESSAGE_BYTES
            || value.original_message_bytes.0 < value.message_prefix.len() as u64
            || !crate::domain::identifiers::valid_sha256(&value.sha256)
            || (value.original_message_bytes.0 > value.message_prefix.len() as u64
                && !value.truncated)
            || !value.details_omitted
            || !value.remediation_omitted
        {
            return Err(invalid());
        }
        Ok(Self {
            code: value.code,
            retryable: value.retryable,
            message_prefix: compact(value.message_prefix),
            original_message_bytes: value.original_message_bytes,
            sha256: compact(value.sha256),
            truncated: value.truncated,
            details_omitted: true,
            remediation_omitted: true,
        })
    }
}
impl NativeExportCoreErrorV1 {
    pub fn from_api(error: &ApiError) -> Self {
        let message_prefix = prefix(&error.message);
        Self {
            code: error.code,
            retryable: error.retryable,
            original_message_bytes: crate::domain::api_values::DecimalU64(
                error.message.len() as u64
            ),
            sha256: hex(Sha256::digest(error.message.as_bytes()).into()),
            truncated: error.message.len() > message_prefix.len(),
            message_prefix,
            details_omitted: true,
            remediation_omitted: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExportResultV1 {
    pub item: NativeExportItemV1,
    pub write_committed: bool,
    pub failure: Option<NativeExportFailureV1>,
}

pub fn encode_native_export_v1(value: &impl Serialize) -> Result<Vec<u8>, ApiError> {
    bounded(value, NATIVE_EXPORT_JSON_BYTES)
}
fn bounded(value: &impl Serialize, maximum: usize) -> Result<Vec<u8>, ApiError> {
    crate::application::local_host::encode_bounded(value, maximum).map_err(|_| exhausted())
}
fn prefix(value: &str) -> String {
    let mut end = value.len().min(NATIVE_EXPORT_MESSAGE_BYTES);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}
fn compact(value: String) -> String {
    value.into_boxed_str().into_string()
}
fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(64);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing into String cannot fail");
    }
    output
}
fn invalid() -> NativeExportValueError {
    NativeExportValueError {
        code: ErrorCode::InvalidArgument,
    }
}
fn exhausted() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Native export projection exceeds its byte or row budget",
    )
}

#[cfg(test)]
mod tests {
    use super::super::metadata::maximum_metadata;
    use super::*;
    fn scope() -> NativeIssuerScopeV1 {
        "nsi1.abcdefabcdefabcdefabcdefabcdefab".parse().unwrap()
    }
    fn item(row: u64) -> NativeExportItemV1 {
        NativeExportItemV1::new(
            format!("nsr1.abcdefabcdefabcdefabcdefabcdefab.{row:016x}")
                .parse()
                .unwrap(),
            maximum_metadata(),
            NativeExportStateV1::RejectedBeforeWrite,
            NativeExportReadModeV1::Unavailable,
        )
        .unwrap()
    }
    #[test]
    fn maximum64_item_envelopes_fit_compact_pretty_and_framing_budgets() {
        let list = NativeExportListV1::new(scope(), (1..=64).map(item).collect()).unwrap();
        for value in list.items() {
            assert!(serde_json::to_vec(value).unwrap().len() <= NATIVE_EXPORT_ITEM_BYTES);
        }
        let envelope = serde_json::json!({"api_version":"1","request_id":"😀".repeat(128),"ok":true,"data":list,"error":null});
        let compact = encode_native_export_v1(&envelope).unwrap();
        let pretty = serde_json::to_vec_pretty(&envelope).unwrap();
        assert!(compact.len() + 5 <= NATIVE_EXPORT_WIRE_BYTES);
        assert!(pretty.len() + 5 <= NATIVE_EXPORT_WIRE_BYTES);
    }
    #[test]
    fn list_rejects_overflow_duplicates_and_foreign_scope() {
        assert_eq!(
            NativeExportListV1::new(scope(), (1..=65).map(item).collect())
                .unwrap_err()
                .code,
            ErrorCode::ResourceExhausted
        );
        assert_eq!(
            NativeExportListV1::new(scope(), vec![item(1), item(1)])
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        let foreign = "nsi1.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".parse().unwrap();
        assert!(NativeExportListV1::new(foreign, vec![item(1)]).is_err());
        let raw =
            serde_json::json!({"scope":scope(),"items":(1..=65).map(item).collect::<Vec<_>>()});
        assert!(serde_json::from_value::<NativeExportListV1>(raw).is_err());
    }
    #[test]
    fn native_failure_preserves_original_hash_and_utf8_with_escaped_display() {
        let original = "\u{0001}".repeat(4096);
        let failure = NativeExportFailureV1::from_native(NativeExportFailureParts {
            code: ErrorCode::EngineFailed,
            stage: NativeExportFailureStageV1::Acknowledgement,
            message: &original,
            original_message_bytes: 4096,
            sha256: Sha256::digest(original.as_bytes()).into(),
            truncated: false,
            raw_os_code: Some(5),
            kind: Some("PermissionDenied"),
        })
        .unwrap();
        assert_eq!(failure.message_prefix.len(), 1024);
        assert!(failure.truncated);
        assert_eq!(failure.original_message_bytes.0, 4096);
        assert_eq!(
            failure.sha256,
            hex(Sha256::digest(original.as_bytes()).into())
        );
        let result = NativeExportResultV1 {
            item: item(1),
            write_committed: true,
            failure: Some(failure),
        };
        assert!(encode_native_export_v1(&result).unwrap().len() < NATIVE_EXPORT_JSON_BYTES);
        let mut error = ApiError::new(ErrorCode::EngineFailed, "한".repeat(2048));
        error.details = serde_json::json!({"secret":"x".repeat(100_000)});
        let projected = NativeExportCoreErrorV1::from_api(&error);
        assert!(projected.message_prefix.len() <= 1024);
        assert!(
            projected
                .message_prefix
                .is_char_boundary(projected.message_prefix.len())
        );
        assert!(encode_native_export_v1(&projected).unwrap().len() < 8192);
        assert!(projected.details_omitted);
    }
    #[test]
    fn compact_encoder_refuses_overbudget_without_truncating() {
        assert_eq!(
            encode_native_export_v1(&"x".repeat(NATIVE_EXPORT_JSON_BYTES))
                .unwrap_err()
                .code,
            ErrorCode::ResourceExhausted
        );
        let value = serde_json::to_value(NativeExportCoreErrorV1::from_api(&ApiError::new(
            ErrorCode::EngineFailed,
            "original",
        )))
        .unwrap();
        let mut bad = value;
        bad["details_omitted"] = serde_json::json!(false);
        assert!(serde_json::from_value::<NativeExportCoreErrorV1>(bad).is_err());
    }
    #[test]
    fn bounded_error_and_fallback_envelopes_include_request_id_and_newline() {
        let original = ApiError::new(ErrorCode::EngineFailed, "\u{0001}".repeat(4096));
        let envelope = serde_json::json!({"api_version":"1","request_id":"😀".repeat(128),"ok":true,"data":{"core_error":NativeExportCoreErrorV1::from_api(&original)},"error":null});
        assert!(encode_native_export_v1(&envelope).unwrap().len() + 5 <= NATIVE_EXPORT_WIRE_BYTES);
        let fallback = crate::application::api::Response::failure("😀".repeat(128), exhausted());
        assert!(encode_native_export_v1(&fallback).unwrap().len() + 5 <= NATIVE_EXPORT_WIRE_BYTES);
    }
}
