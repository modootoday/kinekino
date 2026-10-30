//! Bounded saved values. Lookup metadata does not mint native or skip authority.
use super::codec::NativeExportValueError;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_session::NativeGuardIdentity,
    },
    domain::{
        api_values::DecimalU64,
        identifiers::{valid_id, valid_sha256},
        playback_control::PlaybackControl,
        skip_segments::SegmentTimebase,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExportOriginInputV1 {
    pub registry_generation: String,
    pub session: String,
    pub pass: DecimalU64,
    pub segment: String,
    pub request_revision: DecimalU64,
    pub engine_instance_id: DecimalU64,
    pub resolution_artifact_id: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "NativeExportOriginInputV1")]
pub struct NativeExportOriginV1 {
    registry_generation: String,
    session: String,
    pass: DecimalU64,
    segment: String,
    request_revision: DecimalU64,
    engine_instance_id: DecimalU64,
    resolution_artifact_id: String,
}
impl TryFrom<NativeExportOriginInputV1> for NativeExportOriginV1 {
    type Error = NativeExportValueError;
    fn try_from(value: NativeExportOriginInputV1) -> Result<Self, Self::Error> {
        if value.registry_generation.len() > 40
            || !valid_id(&value.registry_generation)
            || !valid_id(&value.session)
            || !valid_id(&value.segment)
            || value.pass.0 == 0
            || value.request_revision.0 == 0
            || value.engine_instance_id.0 == 0
            || !valid_sha256(&value.resolution_artifact_id)
        {
            return Err(invalid());
        }
        Ok(Self {
            registry_generation: compact(value.registry_generation),
            session: compact(value.session),
            pass: value.pass,
            segment: compact(value.segment),
            request_revision: value.request_revision,
            engine_instance_id: value.engine_instance_id,
            resolution_artifact_id: compact(value.resolution_artifact_id),
        })
    }
}
impl NativeExportOriginV1 {
    pub fn registry_generation(&self) -> &str {
        &self.registry_generation
    }
    pub fn session(&self) -> &str {
        &self.session
    }
    pub fn pass(&self) -> DecimalU64 {
        self.pass
    }
    pub fn segment(&self) -> &str {
        &self.segment
    }
    pub fn request_revision(&self) -> DecimalU64 {
        self.request_revision
    }
    pub fn engine_instance_id(&self) -> DecimalU64 {
        self.engine_instance_id
    }
    pub fn resolution_artifact_id(&self) -> &str {
        &self.resolution_artifact_id
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExportAnnotationInputV1 {
    pub annotation_revision: DecimalU64,
    pub verified_revision: DecimalU64,
    pub timebase: SegmentTimebase,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "NativeExportAnnotationInputV1")]
pub struct NativeExportAnnotationV1 {
    annotation_revision: DecimalU64,
    verified_revision: DecimalU64,
    timebase: SegmentTimebase,
}
impl TryFrom<NativeExportAnnotationInputV1> for NativeExportAnnotationV1 {
    type Error = NativeExportValueError;
    fn try_from(value: NativeExportAnnotationInputV1) -> Result<Self, Self::Error> {
        if value.annotation_revision.0 == 0 || value.verified_revision != value.annotation_revision
        {
            return Err(invalid());
        }
        Ok(Self {
            annotation_revision: value.annotation_revision,
            verified_revision: value.verified_revision,
            timebase: value.timebase,
        })
    }
}
impl NativeExportAnnotationV1 {
    pub fn annotation_revision(&self) -> DecimalU64 {
        self.annotation_revision
    }
    pub fn verified_revision(&self) -> DecimalU64 {
        self.verified_revision
    }
    pub fn timebase(&self) -> SegmentTimebase {
        self.timebase
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExportMetadataInputV1 {
    pub origin: NativeExportOriginV1,
    pub annotation: NativeExportAnnotationV1,
    pub actual_target_us: DecimalU64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "NativeExportMetadataInputV1")]
pub struct NativeExportMetadataV1 {
    origin: NativeExportOriginV1,
    annotation: NativeExportAnnotationV1,
    actual_target_us: DecimalU64,
}
impl TryFrom<NativeExportMetadataInputV1> for NativeExportMetadataV1 {
    type Error = NativeExportValueError;
    fn try_from(value: NativeExportMetadataInputV1) -> Result<Self, Self::Error> {
        PlaybackControl::Seek {
            position_us: value.actual_target_us,
        }
        .validate()
        .map_err(|_| invalid())?;
        Ok(Self {
            origin: value.origin,
            annotation: value.annotation,
            actual_target_us: value.actual_target_us,
        })
    }
}
impl NativeExportMetadataV1 {
    pub fn origin(&self) -> &NativeExportOriginV1 {
        &self.origin
    }
    pub fn annotation(&self) -> NativeExportAnnotationV1 {
        self.annotation
    }
    pub fn actual_target_us(&self) -> DecimalU64 {
        self.actual_target_us
    }
    #[allow(
        dead_code,
        reason = "P1 stamps the target from the actual original SubmitInput"
    )]
    pub(crate) fn stamp(
        origin: NativeExportOriginV1,
        annotation: NativeExportAnnotationV1,
        target: DecimalU64,
        identity: &NativeGuardIdentity,
    ) -> Result<Self, ApiError> {
        if origin.engine_instance_id() != identity.engine_instance_id()
            || origin.resolution_artifact_id() != identity.resolution_artifact_id()
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Export origin differs from original native issuer",
            ));
        }
        Self::try_from(NativeExportMetadataInputV1 {
            origin,
            annotation,
            actual_target_us: target,
        })
        .map_err(Into::into)
    }
}
fn compact(value: String) -> String {
    value.into_boxed_str().into_string()
}
fn invalid() -> NativeExportValueError {
    NativeExportValueError {
        code: ErrorCode::InvalidArgument,
    }
}

#[cfg(test)]
pub(super) fn maximum_metadata() -> NativeExportMetadataV1 {
    let origin = NativeExportOriginV1::try_from(NativeExportOriginInputV1 {
        registry_generation: "g".repeat(40),
        session: "s".repeat(80),
        pass: DecimalU64(u64::MAX),
        segment: "t".repeat(80),
        request_revision: DecimalU64(u64::MAX),
        engine_instance_id: DecimalU64(u64::MAX),
        resolution_artifact_id: "f".repeat(64),
    })
    .unwrap();
    let annotation = NativeExportAnnotationV1::try_from(NativeExportAnnotationInputV1 {
        annotation_revision: DecimalU64(u64::MAX),
        verified_revision: DecimalU64(u64::MAX),
        timebase: SegmentTimebase::EnginePlayback,
    })
    .unwrap();
    NativeExportMetadataV1::try_from(NativeExportMetadataInputV1 {
        origin,
        annotation,
        actual_target_us: DecimalU64(crate::domain::playback_control::MAX_SEEK_US),
    })
    .unwrap()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn origin_annotation_and_actual_target_remain_independent_saved_values() {
        let mut value = maximum_metadata();
        value.origin.request_revision = DecimalU64(9);
        value.annotation.annotation_revision = DecimalU64(3);
        value.annotation.verified_revision = DecimalU64(3);
        value.actual_target_us = DecimalU64(17);
        let saved = value.clone();
        value.origin.request_revision = DecimalU64(10);
        assert_eq!(saved.origin().request_revision(), DecimalU64(9));
        assert_eq!(saved.annotation().annotation_revision(), DecimalU64(3));
        assert_eq!(saved.actual_target_us(), DecimalU64(17));
        assert_eq!(
            serde_json::from_slice::<NativeExportMetadataV1>(&serde_json::to_vec(&saved).unwrap())
                .unwrap(),
            saved
        );
    }
    #[test]
    fn metadata_rejects_unknown_fields_timebase_and_unverified_annotation() {
        let value = maximum_metadata();
        let mut json = serde_json::to_value(value).unwrap();
        json["annotation"]["verified_revision"] = serde_json::json!("1");
        assert!(serde_json::from_value::<NativeExportMetadataV1>(json.clone()).is_err());
        json["annotation"]["verified_revision"] = serde_json::json!(u64::MAX.to_string());
        json["annotation"]["timebase"] = serde_json::json!("source_pts");
        assert!(serde_json::from_value::<NativeExportMetadataV1>(json).is_err());
        let mut json = serde_json::to_value(maximum_metadata()).unwrap();
        json["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<NativeExportMetadataV1>(json).is_err());
    }
    #[test]
    fn origin_rejects_oversize_identity_and_removes_excess_capacity() {
        let origin = maximum_metadata().origin;
        let data = NativeExportOriginInputV1 {
            registry_generation: origin.registry_generation,
            session: origin.session,
            pass: origin.pass,
            segment: origin.segment,
            request_revision: origin.request_revision,
            engine_instance_id: origin.engine_instance_id,
            resolution_artifact_id: origin.resolution_artifact_id,
        };
        let mut bad = data.clone();
        bad.session.push('x');
        assert!(NativeExportOriginV1::try_from(bad).is_err());
        let mut data = data;
        data.session = String::with_capacity(1_048_576);
        data.session.push_str("session");
        let saved = NativeExportOriginV1::try_from(data).unwrap();
        assert_eq!(saved.session.capacity(), saved.session.len());
    }
    #[test]
    fn target_stamp_requires_the_actual_original_native_identity() {
        let identity = crate::application::playback_session::guarded::test_submission(
            crate::application::playback_control::ControlReceipt::enqueued(7),
        )
        .ticket
        .identity()
        .clone();
        let maximum = maximum_metadata();
        assert_eq!(
            NativeExportMetadataV1::stamp(
                maximum.origin,
                maximum.annotation,
                DecimalU64(1),
                &identity
            )
            .unwrap_err()
            .code,
            ErrorCode::RevisionConflict
        );
        let mut value = maximum_metadata();
        value.origin.engine_instance_id = identity.engine_instance_id();
        value.origin.resolution_artifact_id = identity.resolution_artifact_id().into();
        let saved = NativeExportMetadataV1::stamp(
            value.origin,
            value.annotation,
            DecimalU64(17),
            &identity,
        )
        .unwrap();
        assert_eq!(saved.actual_target_us(), DecimalU64(17));
    }
}
