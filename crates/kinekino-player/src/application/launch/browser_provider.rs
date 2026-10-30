//! Trusted desktop provider roots; live navigation remains in the shared browser jobs.
use crate::application::api::{ApiError, ErrorCode};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct BrowserProviderConfiguration {
    pub job_root: PathBuf,
    pub catalog_root: PathBuf,
    pub profile_root: PathBuf,
}
impl BrowserProviderConfiguration {
    pub fn from_explicit(
        job_root: PathBuf,
        catalog_root: PathBuf,
        profile_root: PathBuf,
    ) -> Result<Self, ApiError> {
        if [&job_root, &catalog_root, &profile_root]
            .iter()
            .any(|root| !root.is_absolute())
        {
            return Err(invalid());
        }
        // The provider constructor checks canonical root separation and profile custody.
        Ok(Self {
            job_root,
            catalog_root,
            profile_root,
        })
    }
}

/// Other launch option owners have already selected job/catalog roots.
/// Absence selects no provider; the live viewer caller must reject missing required configuration.
pub fn desktop_configuration(
    args: &[String],
    job_root: Option<&Path>,
    catalog_root: Option<&Path>,
) -> Result<Option<BrowserProviderConfiguration>, ApiError> {
    let mut selected = None;
    let mut index = 0;
    while index < args.len() {
        let value = if args[index] == "--browser-profile-root" {
            index += 1;
            Some(
                args.get(index)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(invalid)?
                    .as_str(),
            )
        } else {
            args[index].strip_prefix("--browser-profile-root=")
        };
        if let Some(value) = value
            && (value.is_empty() || selected.replace(PathBuf::from(value)).is_some())
        {
            return Err(invalid());
        }
        index += 1;
    }
    let Some(profile_root) = selected else {
        return Ok(None);
    };
    if super::desktop_launch_plan(args)?.mode != super::DesktopMode::NativeViewer {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Browser provider configuration requires Native Viewer",
        ));
    }
    let (Some(job_root), Some(catalog_root)) = (job_root, catalog_root) else {
        return Err(invalid());
    };
    BrowserProviderConfiguration::from_explicit(
        job_root.to_owned(),
        catalog_root.to_owned(),
        profile_root,
    )
    .map(Some)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Live browser provider requires explicit absolute job/catalog/profile roots; duplicate, missing or empty profile options are invalid",
    )
}
