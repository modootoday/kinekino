//! One product-owned fixed selection. No file/JSON/CLI grants model authority.
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::{
    application::ProfileAdmission,
    domain::{ArtifactPin, VadProfile},
};

struct PinLiteral {
    id: &'static str,
    sha256: &'static str,
    bytes: u64,
}
impl PinLiteral {
    fn value(&self) -> Result<ArtifactPin, ApiError> {
        ArtifactPin::new(self.id.into(), self.sha256.into(), DecimalU64(self.bytes))
    }
}
struct ProductReleasePins {
    public_profile_id: &'static str,
    license: PinLiteral,
    worker: PinLiteral,
    manifest: PinLiteral,
    deployment: Option<&'static NativePins>,
}
struct NativeFilePin {
    name: &'static str,
    pin: PinLiteral,
    startup: bool,
}
struct NativePins {
    modules: &'static [NativeFilePin],
    notices: &'static [NativeFilePin],
    system_module_names: &'static [&'static str],
    minimum_windows_build: u32,
}
pub enum ProductArtifactRole {
    Model,
    License,
    Worker,
    Dependency { startup: bool },
    Notice,
}
pub struct ProductArtifact {
    pub name: &'static str,
    pub pin: ArtifactPin,
    pub role: ProductArtifactRole,
}
pub struct ProductNativeDeployment {
    pub artifacts: Vec<ProductArtifact>,
    pub system_module_names: Vec<String>,
    pub minimum_windows_build: u32,
}
// Deliberately unset: existing cache/review samples are not a product release.
// Parent packaging may fill this ONE selection only after explicit product pin
// acceptance. Do not embed a worker's own hash into that same worker binary.
static PRODUCT_RELEASE: Option<ProductReleasePins> = None;
pub struct ConfiguredVadRelease {
    public_profile_id: &'static str,
    profile: VadProfile,
    manifest: ArtifactPin,
    native: Option<ProductNativeDeployment>,
}
impl ConfiguredVadRelease {
    pub fn configured() -> Result<Self, ApiError> {
        let pins=PRODUCT_RELEASE.as_ref().ok_or_else(||ApiError::new(ErrorCode::NeedsInput,"Product-owned license, dedicated worker and full release manifest pins are not configured"))?;
        Self::from_product_pins(pins)
    }
    fn from_product_pins(pins: &ProductReleasePins) -> Result<Self, ApiError> {
        let native = pins
            .deployment
            .map(|d| {
                let mut artifacts = vec![
                    ProductArtifact {
                        name: "model.onnx",
                        pin: VadProfile::cached_silero(
                            pins.license.value()?,
                            pins.worker.value()?,
                        )?
                        .weights()
                        .clone(),
                        role: ProductArtifactRole::Model,
                    },
                    ProductArtifact {
                        name: "model-license.txt",
                        pin: pins.license.value()?,
                        role: ProductArtifactRole::License,
                    },
                    ProductArtifact {
                        name: "media-vad-worker.exe",
                        pin: pins.worker.value()?,
                        role: ProductArtifactRole::Worker,
                    },
                ];
                for m in d.modules {
                    artifacts.push(ProductArtifact {
                        name: m.name,
                        pin: m.pin.value()?,
                        role: ProductArtifactRole::Dependency { startup: m.startup },
                    });
                }
                for n in d.notices {
                    artifacts.push(ProductArtifact {
                        name: n.name,
                        pin: n.pin.value()?,
                        role: ProductArtifactRole::Notice,
                    });
                }
                Ok::<_, ApiError>(ProductNativeDeployment {
                    artifacts,
                    system_module_names: d
                        .system_module_names
                        .iter()
                        .map(|s| (*s).into())
                        .collect(),
                    minimum_windows_build: d.minimum_windows_build,
                })
            })
            .transpose()?;
        Ok(Self {
            public_profile_id: pins.public_profile_id,
            profile: VadProfile::cached_silero(pins.license.value()?, pins.worker.value()?)?,
            manifest: pins.manifest.value()?,
            native,
        })
    }
    pub fn public_profile_id(&self) -> &str {
        self.public_profile_id
    }
    pub fn profile(&self) -> &VadProfile {
        &self.profile
    }
    pub fn manifest(&self) -> &ArtifactPin {
        &self.manifest
    }
    pub fn native_deployment(&self) -> Result<&ProductNativeDeployment, ApiError> {
        self.native.as_ref().ok_or_else(||ApiError::new(ErrorCode::NeedsInput,"Product-owned native files/notices, supported Windows baseline and load phases are not configured"))
    }
}
impl ProfileAdmission for ConfiguredVadRelease {
    fn authorize(&self, profile: &VadProfile) -> Result<(), ApiError> {
        if profile == &self.profile {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Requested VAD profile is not the product-owned fixed release",
            ))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn pins() -> ProductReleasePins {
        ProductReleasePins {
            deployment: None,
            public_profile_id: "fixture-only",
            license: PinLiteral {
                id: "unit-license",
                sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                bytes: 16,
            },
            worker: PinLiteral {
                id: "unit-worker",
                sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                bytes: 16,
            },
            manifest: PinLiteral {
                id: "unit-release",
                sha256: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                bytes: 16,
            },
        }
    }
    #[test]
    fn unconfigured_product_does_not_infer_admission_from_cached_model() {
        assert_eq!(
            ConfiguredVadRelease::configured().err().unwrap().code,
            ErrorCode::NeedsInput
        );
    }
    #[test]
    fn fixed_product_selection_compares_full_profile_and_runtime_identity() {
        let release = ConfiguredVadRelease::from_product_pins(&pins()).unwrap();
        assert!(release.authorize(release.profile()).is_ok());
        let other = VadProfile::cached_silero(
            release.profile().license_document().clone(),
            ArtifactPin::new(
                "unit-worker-alias".into(),
                release.profile().runtime().sha256().into(),
                DecimalU64(16),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            release.authorize(&other).unwrap_err().code,
            ErrorCode::PermissionDenied
        );
    }
    #[test]
    fn other_revision_or_license_cannot_reuse_fixed_product_permission() {
        let release = ConfiguredVadRelease::from_product_pins(&pins()).unwrap();
        let p = release.profile();
        let other = VadProfile::new(
            p.id().into(),
            p.repo().into(),
            "2".repeat(40),
            p.weights().clone(),
            p.license_document().clone(),
            p.runtime().clone(),
        )
        .unwrap();
        assert_eq!(
            release.authorize(&other).unwrap_err().code,
            ErrorCode::PermissionDenied
        );
        let other = VadProfile::cached_silero(
            ArtifactPin::new("other-license".into(), "d".repeat(64), DecimalU64(16)).unwrap(),
            p.runtime().clone(),
        )
        .unwrap();
        assert_eq!(
            release.authorize(&other).unwrap_err().code,
            ErrorCode::PermissionDenied
        );
    }
}
