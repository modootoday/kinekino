use super::{Pin, changed};
use kinekino_media_contracts::errors::ApiError;
use kinekino_vad::domain::{ArtifactPin, MAX_RUNTIME_NOTICE_FILES, validate_runtime_notice_sizes};
use serde::{Deserialize, Serialize};
#[derive(Clone)]
pub struct ChildPinnedFile {
    pub path: String,
    pub pin: ArtifactPin,
}
#[derive(Clone)]
pub struct ChildNativeModule {
    pub file: ChildPinnedFile,
    pub required_at_startup: bool,
}
#[derive(Clone)]
pub struct ChildNativeDeployment {
    pub worker_path: String,
    pub modules: Vec<ChildNativeModule>,
    pub notices: Vec<ChildPinnedFile>,
    pub system_module_names: Vec<String>,
    pub minimum_windows_build: u32,
}
impl ChildNativeDeployment {
    pub(super) fn validate(&self) -> Result<(), ApiError> {
        let valid_path =
            |p: &str| !p.is_empty() && p.len() <= 4096 && !p.chars().any(char::is_control);
        if !valid_path(&self.worker_path)
            || self.modules.is_empty()
            || self.modules.len() > 16
            || self.notices.is_empty()
            || self.notices.len() > MAX_RUNTIME_NOTICE_FILES
            || self.system_module_names.is_empty()
            || self.system_module_names.len() > 128
            || self.minimum_windows_build == 0
            || self.modules.iter().any(|m| !valid_path(&m.file.path))
            || self.notices.iter().any(|n| !valid_path(&n.path))
            || self.system_module_names.iter().any(|n| {
                n.is_empty()
                    || n.len() > 128
                    || !n
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
            })
        {
            return Err(changed("Incomplete or oversized parent native deployment"));
        }
        validate_runtime_notice_sizes(self.notices.iter().map(|n| n.pin.bytes()))
            .map_err(|_| changed("Parent native notice sizes exceed fixed bounds"))?;
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileWire {
    path: String,
    pin: Pin,
}
impl From<&ChildPinnedFile> for FileWire {
    fn from(f: &ChildPinnedFile) -> Self {
        Self {
            path: f.path.clone(),
            pin: Pin::from(&f.pin),
        }
    }
}
impl FileWire {
    fn checked(self) -> Result<ChildPinnedFile, ApiError> {
        Ok(ChildPinnedFile {
            path: self.path,
            pin: self.pin.checked()?,
        })
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModuleWire {
    file: FileWire,
    required_at_startup: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Wire {
    worker_path: String,
    modules: Vec<ModuleWire>,
    notices: Vec<FileWire>,
    system_module_names: Vec<String>,
    minimum_windows_build: u32,
}
impl From<&ChildNativeDeployment> for Wire {
    fn from(d: &ChildNativeDeployment) -> Self {
        Self {
            worker_path: d.worker_path.clone(),
            modules: d
                .modules
                .iter()
                .map(|m| ModuleWire {
                    file: FileWire::from(&m.file),
                    required_at_startup: m.required_at_startup,
                })
                .collect(),
            notices: d.notices.iter().map(FileWire::from).collect(),
            system_module_names: d.system_module_names.clone(),
            minimum_windows_build: d.minimum_windows_build,
        }
    }
}
impl Wire {
    pub(super) fn checked(self) -> Result<ChildNativeDeployment, ApiError> {
        let d = ChildNativeDeployment {
            worker_path: self.worker_path,
            modules: self
                .modules
                .into_iter()
                .map(|m| {
                    Ok(ChildNativeModule {
                        file: m.file.checked()?,
                        required_at_startup: m.required_at_startup,
                    })
                })
                .collect::<Result<_, ApiError>>()?,
            notices: self
                .notices
                .into_iter()
                .map(FileWire::checked)
                .collect::<Result<_, _>>()?,
            system_module_names: self.system_module_names,
            minimum_windows_build: self.minimum_windows_build,
        };
        d.validate()?;
        Ok(d)
    }
}
