//! Delegation from an existing trusted Rust host, separate from caller requests.
use super::{MAX_RELEASE_MANIFEST_BYTES, bind_static_worker};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::{
    application::ProfileAdmission,
    domain::{ArtifactPin, VadProfile},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_CHILD_CONFIGURATION_BYTES: u64 = 8192;
pub const CHILD_CONFIGURATION_FILE: &str = "parent-release.v1.json";
#[path = "child_configuration/deployment.rs"]
mod deployment;
pub use deployment::{ChildNativeDeployment, ChildNativeModule, ChildPinnedFile};

/// Only host composition calls this constructor with its existing issuer.
/// This is not Deserialize and never reads a request or environment variable.
pub struct ParentSelectedRelease {
    profile: VadProfile,
    manifest: ArtifactPin,
    deployment: Option<ChildNativeDeployment>,
}
impl ParentSelectedRelease {
    pub fn authorize(
        profile: VadProfile,
        manifest: ArtifactPin,
        issuer: &dyn ProfileAdmission,
    ) -> Result<Self, ApiError> {
        issuer.authorize(&profile)?;
        if manifest.bytes() > MAX_RELEASE_MANIFEST_BYTES {
            return Err(changed("Selected manifest exceeds bound"));
        }
        Ok(Self {
            profile,
            manifest,
            deployment: None,
        })
    }
    /// Product-owned Rust deployment data. No value is selected from requests.
    pub fn with_native_deployment(
        mut self,
        deployment: ChildNativeDeployment,
    ) -> Result<Self, ApiError> {
        deployment.validate()?;
        self.deployment = Some(deployment);
        Ok(self)
    }
    pub fn check_before_spawn(
        &self,
        profile: &VadProfile,
        manifest: &ArtifactPin,
        worker: &ArtifactPin,
    ) -> Result<(), ApiError> {
        bind_static_worker(&self.profile, profile, worker)?;
        if manifest != &self.manifest {
            return Err(changed(
                "Attempt belongs to another selected release manifest",
            ));
        }
        Ok(())
    }
    pub fn profile(&self) -> &VadProfile {
        &self.profile
    }
    pub fn native_deployment(&self) -> Option<&ChildNativeDeployment> {
        self.deployment.as_ref()
    }
    pub fn manifest(&self) -> &ArtifactPin {
        &self.manifest
    }
    pub fn encode_for_attempt(
        &self,
        request: &ArtifactPin,
        profile: &VadProfile,
        manifest: &ArtifactPin,
        worker: &ArtifactPin,
    ) -> Result<(Vec<u8>, ArtifactPin), ApiError> {
        self.check_before_spawn(profile, manifest, worker)?;
        self.encode_for_request(request)
    }
    pub fn encode_for_request(
        &self,
        request: &ArtifactPin,
    ) -> Result<(Vec<u8>, ArtifactPin), ApiError> {
        let p = &self.profile;
        let wire = Wire {
            schema_version: "parent-vad-release/1".into(),
            request: Pin::from(request),
            manifest: Pin::from(&self.manifest),
            profile_id: p.id().into(),
            repo: p.repo().into(),
            revision: p.revision().into(),
            weights: Pin::from(p.weights()),
            license: Pin::from(p.license_document()),
            worker: Pin::from(p.runtime()),
            deployment: self.deployment.as_ref().map(deployment::Wire::from),
        };
        let bytes =
            serde_json::to_vec(&wire).map_err(|_| changed("Cannot encode parent selection"))?;
        if bytes.len() as u64 > MAX_CHILD_CONFIGURATION_BYTES {
            return Err(changed("Parent selection exceeds bound"));
        }
        let pin = ArtifactPin::new(
            "parent-release-configuration".into(),
            format!("{:x}", Sha256::digest(&bytes)),
            DecimalU64(bytes.len() as u64),
        )?;
        Ok((bytes, pin))
    }
}

/// Decoding establishes syntax/integrity only. This type is not ProfileAdmission.
/// The child adapter must read it through the original trusted parent's guards.
pub struct DelegatedReleaseConfiguration {
    profile: VadProfile,
    manifest: ArtifactPin,
    deployment: Option<ChildNativeDeployment>,
}
impl DelegatedReleaseConfiguration {
    pub fn native_deployment(&self) -> Option<&ChildNativeDeployment> {
        self.deployment.as_ref()
    }
    pub fn profile(&self) -> &VadProfile {
        &self.profile
    }
    pub fn manifest(&self) -> &ArtifactPin {
        &self.manifest
    }
    pub fn check_request(
        &self,
        requested: &VadProfile,
        manifest: &ArtifactPin,
    ) -> Result<(), ApiError> {
        bind_static_worker(&self.profile, requested, self.profile.runtime())?;
        if manifest != &self.manifest {
            return Err(changed(
                "Child launch differs from parent-selected manifest",
            ));
        }
        Ok(())
    }
}
pub fn decode_parent_configuration(
    bytes: &[u8],
    expected: &ArtifactPin,
    original_request: &ArtifactPin,
) -> Result<DelegatedReleaseConfiguration, ApiError> {
    if expected.id() != "parent-release-configuration"
        || bytes.len() as u64 > MAX_CHILD_CONFIGURATION_BYTES
        || bytes.len() as u64 != expected.bytes()
        || format!("{:x}", Sha256::digest(bytes)) != expected.sha256()
    {
        return Err(changed("Original parent configuration bytes/SHA differ"));
    }
    let w: Wire =
        serde_json::from_slice(bytes).map_err(|_| changed("Invalid parent selection syntax"))?;
    if w.schema_version != "parent-vad-release/1" || w.request.checked()? != *original_request {
        return Err(changed(
            "Parent selection belongs to another original request",
        ));
    }
    let manifest = w.manifest.checked()?;
    if manifest.bytes() > MAX_RELEASE_MANIFEST_BYTES {
        return Err(changed("Parent manifest exceeds bound"));
    }
    let profile = VadProfile::new(
        w.profile_id,
        w.repo,
        w.revision,
        w.weights.checked()?,
        w.license.checked()?,
        w.worker.checked()?,
    )?;
    let deployment = w.deployment.map(deployment::Wire::checked).transpose()?;
    Ok(DelegatedReleaseConfiguration {
        profile,
        manifest,
        deployment,
    })
}
fn changed(message: &str) -> ApiError {
    ApiError::new(ErrorCode::SourceExpired, message)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    id: String,
    sha256: String,
    bytes: DecimalU64,
}
impl From<&ArtifactPin> for Pin {
    fn from(p: &ArtifactPin) -> Self {
        Self {
            id: p.id().into(),
            sha256: p.sha256().into(),
            bytes: DecimalU64(p.bytes()),
        }
    }
}
impl Pin {
    fn checked(self) -> Result<ArtifactPin, ApiError> {
        ArtifactPin::new(self.id, self.sha256, self.bytes)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: String,
    request: Pin,
    manifest: Pin,
    profile_id: String,
    repo: String,
    revision: String,
    weights: Pin,
    license: Pin,
    worker: Pin,
    deployment: Option<deployment::Wire>,
}

#[cfg(test)]
#[path = "child_configuration/tests.rs"]
mod tests;
